use chrono::NaiveDate;

use crate::db::schema::trade_statistics;

#[derive(Queryable, Insertable, Debug, AsChangeset)]
#[primary_key(item_code, direction, date)]
pub struct TradeStatistic {
    pub(crate) item_code: String,
    pub(crate) buy: bool,
    pub(crate) quantity: i64,
    pub(crate) date: NaiveDate,
}
