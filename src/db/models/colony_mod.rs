use crate::db::models::StringVec;
use crate::db::schema::colony_mods;
use uuid::Uuid;

#[derive(Queryable, Insertable, Identifiable, Debug, AsChangeset)]
#[primary_key(colony_id)]
#[table_name = "colony_mods"]
pub struct ColonyMods {
    pub colony_id: Uuid,
    pub mods: StringVec,
}
