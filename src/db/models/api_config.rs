use crate::db::schema::api_config;
use crate::impl_from_sql;
use crate::impl_to_sql;
use crate::jtd::api_config::structure::ApiConfigData;

use diesel::pg::Pg;
use diesel::serialize::Output;
use diesel::sql_types::Jsonb;
use diesel::types::FromSql;
use diesel::types::ToSql;
use std::io::Write;

#[derive(Queryable, Identifiable, Debug, Default, Clone)]
#[primary_key(version)]
#[table_name = "api_config"]
pub struct ApiConfig {
    pub version: i32,
    pub config_data: ApiConfigData,
}

impl_to_sql!(for ApiConfigData);
impl_from_sql!(for ApiConfigData);
