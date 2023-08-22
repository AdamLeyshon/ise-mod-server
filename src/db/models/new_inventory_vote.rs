use crate::db::schema::new_inventory_vote_tracker;
use uuid::Uuid;

#[derive(Queryable, QueryableByName, Insertable, Identifiable, Debug)]
#[primary_key(client_bind_id, version)]
#[table_name = "new_inventory_vote_tracker"]
pub struct NewInventoryVote {
    pub client_bind_id: Uuid,
    pub version: String,
    pub colony_id: Uuid,
}
