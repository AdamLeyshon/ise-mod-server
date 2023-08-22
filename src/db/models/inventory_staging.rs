use crate::db::schema::colony_inventory_staging;
use bigdecimal::BigDecimal;
use macros::FieldCount;
use uuid::Uuid;

#[derive(Queryable, QueryableByName, Insertable, Identifiable, Debug, AsChangeset, FieldCount)]
#[primary_key(colony_id, version)]
#[table_name = "colony_inventory_staging"]
pub struct ColonyInventoryStaging {
    pub colony_id: Uuid,
    pub item_code: String,
    pub thing_def: String,
    pub quality: Option<i32>,
    pub minified: bool,
    pub base_value: BigDecimal,
    pub stuff: Option<String>,
    pub weight: BigDecimal,
    pub version: String,
}
