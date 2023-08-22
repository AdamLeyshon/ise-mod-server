use chrono::NaiveDateTime;

use crate::db::schema::account_binds;
use crate::db::schema::client_binds;
use uuid::Uuid;

#[derive(Queryable, Insertable, Debug)]
#[table_name = "account_binds"]
pub struct AccountBind {
    pub bind_id: Uuid,
    pub steam_id: Option<String>,
    pub confirmed: bool,
    pub date_added: NaiveDateTime,
    pub date_expire: NaiveDateTime,
    pub account_fk: Option<i32>,
}

#[derive(Queryable, Insertable, Debug)]
#[table_name = "client_binds"]
pub struct ClientBind {
    pub client_bind_id: Uuid,
    pub account_fk: i32,
    pub confirmed: bool,
    pub date_added: NaiveDateTime,
}
