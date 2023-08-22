use chrono::NaiveDateTime;

use crate::db::schema::accounts;

#[derive(Queryable, Debug)]
pub struct Account {
    pub account_id: i32,
    pub date_added: NaiveDateTime,
    pub username: Option<String>,
    pub password: Option<String>,
    pub e_mail: Option<String>,
    pub mfa_code: Option<String>,
    pub active: bool,
    pub steam_id: Option<String>,
}

#[derive(Insertable, Debug)]
#[table_name = "accounts"]
pub struct NewAccount {
    pub date_added: NaiveDateTime,
    pub username: Option<String>,
    pub password: Option<String>,
    pub e_mail: Option<String>,
    pub mfa_code: Option<String>,
    pub active: bool,
    pub steam_id: Option<String>,
}
