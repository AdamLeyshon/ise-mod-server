use std::collections::HashSet;
use std::io::Write;
use std::iter::FromIterator;

use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Jsonb;

use itertools::Itertools;
use uuid::Uuid;

use crate::db::models::colony::Colony;
use crate::db::models::inventory::Inventory;
use crate::db::models::order::{Order, OrderNoManifest};
use crate::db::{get_pg_connection, Ppc};

use crate::packets::common::CurrencyEnum;
use crate::packets::order::{
    DeliveryItem, OrderItem, OrderManifestReply, OrderStatusEnum, OrderStatusReply,
};

use crate::structs::general::DbPkLoadable;
use crate::structs::inventory::get_inventory;
use bigdecimal::BigDecimal;

#[derive(FromSqlRow, AsExpression, serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
#[sql_type = "Jsonb"]
pub struct OrderStats {
    /// Weight of stuff to collect from the colony
    pub total_sell_weight: BigDecimal,
    /// Weight of stuff to send to the colony
    pub total_buy_weight: BigDecimal,
    /// Total cash given to the colony
    pub total_sell_cost: BigDecimal,
    /// Total cash taken from the colony
    pub total_buy_cost: BigDecimal,
}

#[derive(FromSqlRow, AsExpression, serde::Serialize, serde::Deserialize, Debug, Default)]
#[sql_type = "Jsonb"]
pub struct OrderManifest {
    pub wts: Vec<OrderItem>,
    pub wtb: Vec<OrderItem>,
    #[serde(default)]
    pub balance_adjustment: i32,
    #[serde(default)]
    pub currency: CurrencyEnum,
}

impl_to_sql!(for OrderStats, OrderManifest);
impl_from_sql!(for OrderStats, OrderManifest);

impl From<Inventory> for DeliveryItem {
    fn from(inv: Inventory) -> Self {
        DeliveryItem {
            item_code: inv.item_code,
            thing_def: inv.thing_def,
            quantity: 0,
            quality: inv.quality.unwrap_or(0),
            stuff: inv.stuff.unwrap_or("".to_string()),
            minified: inv.minified,
        }
    }
}

impl From<&Inventory> for DeliveryItem {
    fn from(inv: &Inventory) -> Self {
        DeliveryItem {
            item_code: inv.item_code.clone(),
            thing_def: inv.thing_def.clone(),
            quantity: 0,
            quality: inv.quality.unwrap_or(0),
            stuff: (inv.stuff.as_ref().unwrap_or(&"".to_string())).clone(),
            minified: inv.minified,
        }
    }
}

impl From<Order> for OrderStatusReply {
    fn from(o: Order) -> Self {
        OrderStatusReply {
            order_id: o.order_id.to_string(),
            status: o.status,
            delivery_tick: o.end_tick,
            placed_tick: o.start_tick,
        }
    }
}

impl From<OrderNoManifest> for OrderStatusReply {
    fn from(o: OrderNoManifest) -> Self {
        OrderStatusReply {
            order_id: o.order_id.to_string(),
            status: o.status,
            delivery_tick: o.end_tick,
            placed_tick: o.start_tick,
        }
    }
}

make_pk_loadable!(Order, Uuid, crate::db::schema::orders);

impl From<Order> for OrderManifestReply {
    fn from(o: Order) -> Self {
        // Get the items from the shipping manifest
        let items: HashSet<&String> =
            HashSet::from_iter(o.manifest.wtb.iter().map(|item| &item.item_code));

        // Get inventory from DB
        let inv = get_inventory(items, &get_pg_connection());

        // Build Delivery item list using inventory data and fill in manifest quantities.
        let delivery_items: Vec<DeliveryItem> = o
            .manifest
            .wtb
            .iter()
            .map(|item| {
                let mut di = DeliveryItem::from(inv.get(&item.item_code).unwrap());
                di.quantity = item.quantity;
                di
            })
            .collect_vec();

        OrderManifestReply {
            items: delivery_items,
        }
    }
}

pub fn create_order(
    order_stats: &OrderStats,
    colony: &Colony,
    manifest: OrderManifest,
    tick: i32,
    delivery_tick: Option<i32>,
    conn: &Ppc,
) -> Result<Order, ()> {
    use crate::crypto::generate_v4_uuid;
    use crate::db::insert_db_object;
    use crate::db::schema::orders as schema;
    use crate::structs::general::ONE_DAY_TICKS;
    use chrono::Utc;
    let now = Utc::now().naive_utc();

    // If they're not buying and only selling mark as complete immediately.
    let (status, end_tick_value) = if manifest.wtb.is_empty() {
        (OrderStatusEnum::Delivered.into(), tick)
    } else {
        (
            OrderStatusEnum::Placed.into(),
            delivery_tick.unwrap_or(tick + (ONE_DAY_TICKS * 2)),
        )
    };

    let order = Order {
        order_id: generate_v4_uuid(),
        colony_id: colony.colony_id.clone(),
        manifest,
        status,
        start_tick: tick,
        end_tick: end_tick_value,
        order_stats: order_stats.clone(),
        create_date: now,
        update_date: now,
    };
    if let Ok(order) = insert_db_object(&conn, order, schema::table) {
        Ok(order)
    } else {
        Err(())
    }
}
