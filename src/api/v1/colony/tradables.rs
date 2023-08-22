use crate::request_helpers::*;
use crate::traits::from::FromWithColonyUuid;
use actix_web::{HttpRequest, HttpResponse, Result};
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::models::bind::ClientBind;
use crate::db::{get_pg_connection, Ppc};

use crate::structs::colony::validate_ownership_and_fetch;

use crate::packets::colony::ColonyTradableSetRequest;
use crate::traits::item::{HasItemCode, HasThingDef};

use crate::db::models::inventory_staging::ColonyInventoryStaging;
use diesel::dsl::sql;
use diesel::expression::functions::date_and_time::now as sql_now;
use diesel::sql_query;
use std::collections::HashSet;
use took::Timer;

pub async fn action_post(
    _req: HttpRequest,
    bind: ClientBind,
    packet: ProtoBuf<ColonyTradableSetRequest>,
) -> Result<HttpResponse> {
    if let Some(colony) = validate_ownership_and_fetch(None, Some(&packet.0.colony_id), &bind) {
        let mut incoming = packet.0;

        // TODO: Load blacklisted ThingDefs

        // Map incoming data to rows
        let timer = Timer::new();
        let new_inventory: Vec<ColonyInventoryStaging> = incoming
            .item
            .drain(..)
            .map(|item| ColonyInventoryStaging::from_with_uuid(item, colony.colony_id))
            .filter(|item| filter_bad_things(item))
            .collect();
        debug!("Filter/Transform to inventory done, took {}", timer.took());

        // Now we're finished getting a list of ones we need to create
        let merge_tradables: bool = incoming.final_packet;
        let client_bind_id = colony.client_bind_fk.clone();
        let colony_id = colony.colony_id.clone();

        if upsert_new_inventory(new_inventory, client_bind_id, colony_id, merge_tradables).is_ok() {
            Ok(HttpResponse::Ok().finish())
        } else {
            // Deal with the fact a big insert might be blocked by running transactions
            // Official mod will respect this header and try again in a few seconds.
            Ok(HttpResponse::ServiceUnavailable()
                .insert_header(("retry-after", "5"))
                .finish())
        }
    } else {
        error!(
            "Couldn't find Colony ID {} or it doesn't belong to {}",
            &packet.0.colony_id, &bind.client_bind_id
        );
        Ok(HttpResponse::BadRequest().finish())
    }
}

// TODO: Add Database-backed blacklist
fn filter_bad_things<T>(ct: T) -> bool
    where
        T: HasItemCode + HasThingDef,
{
    use crate::structs::inventory::SILVER_ITEM;
    *ct.get_item_code() != SILVER_ITEM.item_code
}

fn upsert_new_inventory(
    mut to_create: Vec<ColonyInventoryStaging>,
    client_id: Uuid,
    colony_id: Uuid,
    merge: bool,
) -> Result<(), ()> {
    use crate::db::schema::colony_inventory_staging as cis_schema;
    use crate::db::schema::inventory as inventory_schema;
    use crate::db::schema::new_inventory as new_inventory_schema;
    use crate::db::schema::new_inventory_vote_tracker as vote_schema;

    let conn = &get_pg_connection();

    match conn
        .build_transaction()
        .run::<_, diesel::result::Error, _>(|| {
            // Work out how many rows we can send in a batch
            let max_rows: usize = ColonyInventoryStaging::batch_size();
            let mut timer: Timer;

            while to_create.len() > 0 {
                let qty = max_rows.min(to_create.len());

                let mut unique_rows = HashSet::<String>::new();

                // Remove duplicate items from batch
                timer = Timer::new();
                let chunk: Vec<ColonyInventoryStaging> = to_create
                    .drain(0..qty)
                    .filter(|item| unique_rows.insert(item.version.clone()))
                    .collect();
                debug!(
                    "Dedupe of {} rows, now has {} rows, took {}",
                    qty,
                    chunk.len(),
                    timer.took()
                );
                debug!("Batch inserting {} rows", chunk.len());

                // Insert into staging table
                timer = Timer::new();
                if let Err(e) = diesel::insert_into(cis_schema::table)
                    .values(&chunk)
                    .on_conflict((cis_schema::colony_id, cis_schema::version))
                    .do_nothing()
                    .execute(conn)
                {
                    error!("Error staging new inventory for {}: {:?}", e, colony_id);
                    return Err(diesel::result::Error::RollbackTransaction);
                };
                debug!("Staging insert done, took {}", timer.took());
            }

            // Is this the final packet of data from the client?
            if !merge {
                return Ok(());
            }

            debug!("Final packet received, processing staged data.");

            // Upsert from Colony Inventory staging into new inventory
            // Ignore any items that the server already has.
            let staging_data = cis_schema::table
                .select((
                    cis_schema::version,
                    cis_schema::item_code,
                    cis_schema::thing_def,
                    cis_schema::quality,
                    cis_schema::minified,
                    cis_schema::base_value,
                    cis_schema::stuff,
                    cis_schema::weight,
                    sql_now,
                ))
                .filter(cis_schema::colony_id.eq(colony_id));

            // Delete existing votes
            timer = Timer::new();
            if let Err(e) = diesel::delete(vote_schema::table)
                .filter(vote_schema::colony_id.eq(&colony_id))
                .execute(conn)
            {
                error!("Failed to delete existing votes {}", e);
                return Err(diesel::result::Error::RollbackTransaction);
            };
            debug!("Vote deletion done, took {}", timer.took());

            // Update tradables
            timer = Timer::new();
            if !update_tradables(colony_id, &conn) {
                return Err(diesel::result::Error::RollbackTransaction);
            }
            debug!("Tradables update done, took {}", timer.took());

            timer = Timer::new();
            // Delete staged items that the server already has
            if let Err(e) = diesel::delete(cis_schema::table)
                .filter(cis_schema::colony_id.eq(colony_id))
                .filter(
                    cis_schema::version
                        .eq_any(inventory_schema::table.select(inventory_schema::version)),
                )
                .execute(conn)
            {
                error!("Failed to delete existing staged items {}", e);
                return Err(diesel::result::Error::RollbackTransaction);
            };
            debug!("Staging data cleanup, took {}", timer.took());

            if let Err(e) = diesel::insert_into(new_inventory_schema::table)
                .values(staging_data)
                .into_columns((
                    new_inventory_schema::version,
                    new_inventory_schema::item_code,
                    new_inventory_schema::thing_def,
                    new_inventory_schema::quality,
                    new_inventory_schema::minified,
                    new_inventory_schema::base_value,
                    new_inventory_schema::stuff,
                    new_inventory_schema::weight,
                    new_inventory_schema::date_added,
                ))
                .on_conflict(new_inventory_schema::version)
                .do_nothing()
                .execute(conn)
            {
                error!("Failed to insert new inventory {}", e);
                return Err(diesel::result::Error::RollbackTransaction);
            };
            debug!("New inventory insert done, took {}", timer.took());

            // Vote on new inventory
            timer = Timer::new();
            let staging_data = cis_schema::table
                .select((
                    sql("").bind::<diesel::sql_types::Uuid, _>(client_id),
                    cis_schema::version,
                    cis_schema::colony_id,
                ))
                .filter(cis_schema::colony_id.eq(colony_id));

            if let Err(e) = diesel::insert_into(vote_schema::table)
                .values(staging_data)
                .into_columns((
                    vote_schema::client_bind_id,
                    vote_schema::version,
                    vote_schema::colony_id,
                ))
                .on_conflict((
                    vote_schema::client_bind_id,
                    vote_schema::version,
                    vote_schema::colony_id,
                ))
                .do_nothing()
                .execute(conn)
            {
                error!("Failed to insert new votes {}", e);
                return Err(diesel::result::Error::RollbackTransaction);
            };

            debug!("Inventory votes done, took {}", timer.took());

            // Delete staging data
            timer = Timer::new();
            if let Err(e) = diesel::delete(cis_schema::table)
                .filter(cis_schema::colony_id.eq(&colony_id))
                .execute(conn)
            {
                error!("Failed to delete colony staging data! {}", e);
                return Err(diesel::result::Error::RollbackTransaction);
            };
            debug!("Delete staging data done, took {}", timer.took());

            Ok(())
        }) {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

fn update_tradables(colony_id: Uuid, conn: &Ppc) -> bool {
    let query = "WITH invt AS ( \
    SELECT cis.item_code \
    FROM colony_inventory_staging cis \
    WHERE cis.colony_id = $1 ) \
    INSERT INTO colony_tradables VALUES \
    ($1,  (SELECT coalesce(jsonb_agg(invt.item_code), '[]'::jsonb) FROM invt), now()) \
    ON CONFLICT (colony_id) DO UPDATE \
    SET tradables =  (SELECT coalesce(jsonb_agg(invt.item_code), '[]'::jsonb) FROM invt), \
    update_date = now();";

    if let Err(e) = sql_query(query)
        .bind::<diesel::sql_types::Uuid, _>(colony_id)
        .execute(conn)
    {
        error!("Error when saving tradables, {}", e);
        false
    } else {
        true
    }
}
