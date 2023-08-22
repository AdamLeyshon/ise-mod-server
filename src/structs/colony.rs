use chrono::NaiveDateTime;
use diesel::result::Error::RollbackTransaction;
use uuid::Uuid;

use crate::crypto::{generate_v4_uuid, parse_uuid};
use crate::db::get_pg_connection;
use crate::db::models::bind::ClientBind;
use crate::db::models::colony::Colony;
use crate::db::models::order::Order;
use crate::packets::colony::ColonyData;
use crate::packets::order::OrderStatusEnum;
use crate::structs::general::DbPkLoadable;
use crate::traits::item::Rollback;

impl From<Colony> for ColonyData {
    /// Convert from ColonyData to Protobuf message data
    fn from(c: Colony) -> Self {
        ColonyData {
            colony_id: c.colony_id.to_string(),
            name: c.name,
            faction_name: c.faction_name,
            map_id: c.map_id,
            tick: c.tick,
            used_dev_mode: c.used_dev_mode,
            game_version: c.game_version,
            platform: c.platform,
            create_date: c.create_date.timestamp(),
            seed: c.seed,
            location: c.location,
        }
    }
}

impl From<ColonyData> for Colony {
    /// Convert from Protobuf message data to ColonyData
    /// client_bind_fk is set to a random UUID since it's never used but must be populated
    fn from(c: ColonyData) -> Self {
        Colony {
            colony_id: Uuid::parse_str(&*c.colony_id).unwrap(),
            name: c.name,
            faction_name: c.faction_name,
            map_id: c.map_id,
            tick: c.tick,
            used_dev_mode: c.used_dev_mode,
            game_version: c.game_version,
            platform: c.platform,
            create_date: NaiveDateTime::from_timestamp(0, 0),
            client_bind_fk: generate_v4_uuid(),
            update_date: NaiveDateTime::from_timestamp(0, 0),
            seed: c.seed,
            location: c.location,
        }
    }
}

make_pk_loadable!(Colony, Uuid, crate::db::schema::colonies);

/// Check if the colony requested is equal to the packet and is owned by the current bind
pub fn validate_ownership_and_fetch(
    url_colony_id: Option<&String>,
    packet_colony_id: Option<&String>,
    bind: &ClientBind,
) -> Option<Colony> {
    // Check that at least one of the values is set and they're not empty
    if url_colony_id.map_or(false, |v| v.is_empty())
        || packet_colony_id.map_or(false, |v| v.is_empty())
    {
        return None;
    };

    // If both values are set, compare them to make sure they're the same
    if url_colony_id.is_some() && packet_colony_id.is_some() {
        if url_colony_id != packet_colony_id {
            return None;
        }
    }

    // Pick which ever one was set, prefer the packet ID if set.
    let pk = if let Some(id) = packet_colony_id {
        id
    } else {
        url_colony_id.unwrap()
    };

    // Parse the UUID from the URL or Protobuf packet
    let colony_id = match parse_uuid(&*pk) {
        Ok(uuid) => uuid,
        Err(_) => return None,
    };

    Colony::load_pk(&colony_id).map_or(None, |c| {
        if c.client_bind_fk == bind.client_bind_id {
            Some(c)
        } else {
            None
        }
    })
}

pub(crate) trait Anticheat {
    fn timewarp(&self, new_tick: i32) -> Result<(), ()>;
}

impl Anticheat for Colony {
    fn timewarp(&self, new_tick: i32) -> Result<(), ()> {
        use crate::db::schema::orders as schema;
        use diesel::prelude::*;
        use diesel::{ExpressionMethods, QueryDsl};

        let conn = &get_pg_connection();

        // Rollback any orders made after the "new tick" which is in the past
        let query = schema::table
            .filter(schema::colony_id.eq(&self.colony_id))
            .filter(schema::start_tick.ge(&new_tick))
            .filter(schema::status.ne(i32::from(OrderStatusEnum::Reversed)))
            .filter(schema::status.ne(i32::from(OrderStatusEnum::Failed)));

        let orders: Vec<Order> = query.get_results(conn).map_err(|_| ())?;

        conn.build_transaction()
            .read_committed()
            .run::<(), diesel::result::Error, _>(|| {
                for mut order in orders {
                    info!(
                        "Rolling back order {} placed @ {}",
                        &order.order_id, &order.start_tick
                    );
                    if order.rollback(conn).is_err() {
                        return Err(RollbackTransaction);
                    };
                }
                Ok(())
            })
            .map_err(|_| ())
    }
}
