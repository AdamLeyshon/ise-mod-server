use crate::db::schema::inventory_promises;
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Queryable, QueryableByName, Insertable, Identifiable, Debug, AsChangeset)]
#[primary_key(colony_id)]
#[table_name = "inventory_promises"]
pub struct InventoryPromise {
    pub colony_id: Uuid,
    pub promise_id: Uuid,
    pub private_key: String,
    pub expiry_date: NaiveDateTime,
    pub activated: bool,
}
