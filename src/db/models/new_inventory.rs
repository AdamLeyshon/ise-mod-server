use crate::db::schema::new_inventory;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use macros::FieldCount;

#[derive(Queryable, QueryableByName, Insertable, Identifiable, Debug, AsChangeset, FieldCount)]
#[primary_key(version)]
#[table_name = "new_inventory"]
pub struct NewInventory {
    pub item_code: String,
    pub thing_def: String,
    pub quality: Option<i32>,
    pub minified: bool,
    pub base_value: BigDecimal,
    pub stuff: Option<String>,
    pub weight: BigDecimal,
    pub version: String,
    pub date_added: NaiveDateTime,
}
