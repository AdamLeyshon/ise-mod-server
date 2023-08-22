use chrono::NaiveDateTime;

use crate::db::models::StringVec;
use crate::db::schema::colony_tradables;
use uuid::Uuid;

#[derive(Queryable, Insertable, Identifiable, Debug, AsChangeset)]
#[primary_key(colony_id)]
#[table_name = "colony_tradables"]
pub struct ColonyTradables {
    pub colony_id: Uuid,
    pub tradables: StringVec,
    pub update_date: NaiveDateTime,
}
