use crate::db::schema::orders;
use crate::structs::order::{OrderManifest, OrderStats};
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Queryable, Identifiable, QueryableByName, Insertable, Debug, AsChangeset)]
#[primary_key(order_id)]
#[table_name = "orders"]
pub struct Order {
    pub order_id: Uuid,
    pub colony_id: Uuid,
    pub manifest: OrderManifest,
    pub status: i32,
    pub start_tick: i32,
    pub end_tick: i32,
    pub order_stats: OrderStats,
    pub create_date: NaiveDateTime,
    pub update_date: NaiveDateTime,
}

#[derive(Queryable, Identifiable, Debug, AsChangeset)]
#[primary_key(order_id)]
#[table_name = "orders"]
pub struct OrderNoManifest {
    pub order_id: Uuid,
    pub colony_id: Uuid,
    pub status: i32,
    pub start_tick: i32,
    pub end_tick: i32,
    pub order_stats: OrderStats,
    pub create_date: NaiveDateTime,
    pub update_date: NaiveDateTime,
}
