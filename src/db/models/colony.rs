use chrono::NaiveDateTime;

use crate::db::schema::colonies;
use uuid::Uuid;

#[derive(Queryable, Insertable, Identifiable, Debug, AsChangeset)]
#[primary_key(colony_id)]
#[table_name = "colonies"]
pub struct Colony {
    pub colony_id: Uuid,
    pub name: String,
    pub faction_name: String,
    pub map_id: i32,
    pub tick: i32,
    /// This is a fuse, once blown, it cannot be unset.
    pub used_dev_mode: bool,
    pub game_version: String,
    pub platform: i32,
    pub create_date: NaiveDateTime,
    pub client_bind_fk: Uuid,
    pub update_date: NaiveDateTime,
    pub seed: String,
    pub location: String,
}
