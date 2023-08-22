use std::io::Write;
use std::ops::{Deref, DerefMut};

use crate::{impl_from_sql, impl_to_sql};
use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Jsonb;

pub mod account;
pub mod api_config;
pub mod bank;
pub mod bind;
pub mod blocked_steam_accounts;
pub mod colony;
pub mod colony_mod;
pub mod colony_tradable;
pub mod inventory;
pub mod inventory_promise;
pub mod inventory_staging;
pub mod maintenance;
pub mod new_inventory;
pub mod new_inventory_vote;
pub mod order;
pub mod price_tracker;
pub mod stock_config;
pub mod summary_inventory_votes;
pub mod trade_stats;

#[derive(FromSqlRow, AsExpression, serde::Serialize, serde::Deserialize, Debug, Default)]
#[sql_type = "Jsonb"]
pub struct StringVec(Vec<String>);

impl Deref for StringVec {
    type Target = Vec<String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StringVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<String>> for StringVec {
    fn from(v: Vec<String>) -> Self {
        Self { 0: v }
    }
}

impl_to_sql!(for StringVec);
impl_from_sql!(for StringVec);

// impl FromSql<Jsonb, Pg> for StringVec {
//     fn from_sql(bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
//         let value = <serde_json::Value as FromSql<Jsonb, Pg>>::from_sql(bytes)?;
//         Ok(serde_json::from_value(value)?)
//     }
// }
//
// impl ToSql<Jsonb, Pg> for StringVec {
//     fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> diesel::serialize::Result {
//         let value = serde_json::to_value(self)?;
//         <serde_json::Value as ToSql<Jsonb, Pg>>::to_sql(&value, out)
//     }
// }
