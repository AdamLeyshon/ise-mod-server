use crate::db::schema::inventory;
use crate::db::views::temporary_vote_data;
use bigdecimal::BigDecimal;
use macros::FieldCount;

#[derive(
    Queryable, QueryableByName, Insertable, Identifiable, Debug, AsChangeset, Clone, FieldCount,
)]
#[primary_key(item_code)]
#[table_name = "inventory"]
pub struct Inventory {
    pub item_code: String,
    pub thing_def: String,
    pub quality: Option<i32>,
    pub quantity: i32,
    pub minified: bool,
    pub base_value: BigDecimal,
    /// This is the cost that ISE buys from the colony at
    pub buy_at: BigDecimal,
    /// This is the cost that ISE sells to the colony at
    pub sell_at: BigDecimal,
    pub stuff: Option<String>,
    pub weight: BigDecimal,
    pub version: String,
}

#[derive(
    Queryable, QueryableByName, Insertable, Identifiable, Clone, Debug, AsChangeset, FieldCount,
)]
#[primary_key(item_code)]
#[table_name = "inventory"]
pub struct InventoryNoQuantity {
    pub item_code: String,
    pub thing_def: String,
    pub quality: Option<i32>,
    pub minified: bool,
    pub base_value: BigDecimal,
    pub buy_at: BigDecimal,
    pub sell_at: BigDecimal,
    pub stuff: Option<String>,
    pub weight: BigDecimal,
    pub version: String,
}

#[derive(Queryable, QueryableByName, Identifiable, Clone, Debug, FieldCount)]
#[primary_key(position)]
#[table_name = "temporary_vote_data"]
pub struct TempInventoryVote {
    pub position: i64,
    pub item_code: String,
    pub thing_def: String,
    pub quality: Option<i32>,
    pub minified: bool,
    pub base_value: BigDecimal,
    pub stuff: Option<String>,
    pub weight: BigDecimal,
    pub version: String,
}

#[derive(Queryable, QueryableByName, Insertable, Identifiable, Debug, AsChangeset)]
#[primary_key(item_code)]
#[table_name = "inventory"]
pub struct InventoryOnlyStats {
    pub item_code: String,
    pub thing_def: String,
    pub minified: bool,
    pub base_value: BigDecimal,
    pub weight: BigDecimal,
    pub version: String,
}
