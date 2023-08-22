use crate::db::models::colony_mod::ColonyMods;
use crate::structs::general::DbPkLoadable;
use uuid::Uuid;
make_pk_loadable!(ColonyMods, Uuid, crate::db::schema::colony_mods);
