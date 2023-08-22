use crate::db::schema::stock_config;
use crate::impl_from_sql;
use crate::impl_to_sql;

use crate::jtd::stock_config::StockConfig;

use diesel::pg::Pg;
use diesel::serialize::Output;
use diesel::sql_types::Jsonb;
use diesel::types::FromSql;
use diesel::types::ToSql;
use std::io::Write;

impl_to_sql!(for StockConfig);
impl_from_sql!(for StockConfig);

#[derive(Queryable, Identifiable, Debug, Default, Clone)]
#[primary_key(version)]
#[table_name = "stock_config"]
pub struct StockConfigRow {
    pub version: i32,
    pub config_data: StockConfig,
}
