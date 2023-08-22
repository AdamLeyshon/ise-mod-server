use uuid::Uuid;

use crate::db::models::colony_tradable::ColonyTradables;
use crate::structs::general::DbPkLoadable;

make_pk_loadable!(ColonyTradables, Uuid, crate::db::schema::colony_tradables);
