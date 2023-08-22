use crate::db::views::summary_inventory_votes;

#[derive(Queryable, QueryableByName, Identifiable, Debug)]
#[primary_key(version)]
#[table_name = "summary_inventory_votes"]
pub struct ThingVoteCount {
    pub item_code: String,
    pub thing_def: String,
    pub quality: Option<i32>,
    pub minified: bool,
    pub base_value: f32,
    pub stuff: Option<String>,
    pub weight: f32,
    pub version: String,
    pub votes: i32,
}
