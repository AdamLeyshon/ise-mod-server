use chrono::NaiveDateTime;

use crate::db::schema::blocked_steam_accounts;

#[derive(Queryable, Insertable, Debug)]
#[table_name = "blocked_steam_accounts"]
pub struct BlockedSteamAccount {
    pub steam_id: String,
    pub reason: i32,
    pub date_added: NaiveDateTime,
}
