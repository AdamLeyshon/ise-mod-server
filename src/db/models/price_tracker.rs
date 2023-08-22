use chrono::NaiveDateTime;

use crate::db::schema::price_tracker;
use bigdecimal::BigDecimal;

#[derive(Queryable, Insertable, Debug, AsChangeset)]
#[primary_key(item_code, value)]
#[table_name = "price_tracker"]
pub struct PriceTracker {
    pub item_code: String,
    pub value: BigDecimal,
    pub create_date: NaiveDateTime,
}
