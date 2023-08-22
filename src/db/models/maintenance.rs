use crate::db::schema::maintenance;
use chrono::NaiveDateTime;

#[derive(Queryable, Identifiable, Insertable, Debug, Default)]
#[primary_key(checksum)]
#[table_name = "maintenance"]
pub struct Maintenance {
    pub checksum: String,
    pub in_progress: bool,
    pub start_time: Option<NaiveDateTime>,
    pub execution_time: Option<NaiveDateTime>,
    pub node_name: Option<String>,
}
