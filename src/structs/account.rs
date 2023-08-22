use chrono::Utc;
use diesel::prelude::*;
use steamid_ng::SteamID;

use crate::db::models::account::{Account, NewAccount};
use crate::db::schema::accounts as schema;
use crate::db::{get_pg_connection, insert_db_object_dyn};
use crate::structs::general::DbPkLoadable;

make_pk_loadable!(Account, i32, crate::db::schema::accounts);

pub fn find_account_for_steam_id(steam_id: &SteamID) -> Account {
    let conn = get_pg_connection();
    let result: Result<Option<Account>, diesel::result::Error> = schema::table
        .filter(schema::steam_id.eq(steam_id.steam3()))
        .order(schema::date_added.desc())
        .first(&conn)
        .optional();
    if let Ok(result) = result {
        if result.is_some() {
            result.unwrap()
        } else {
            create_account_for_steam_id(steam_id)
        }
    } else {
        panic!("Unable to query accounts")
    }
}

pub fn create_account_for_steam_id(steam_id: &SteamID) -> Account {
    let conn = get_pg_connection();
    let time_now = Utc::now().naive_utc();
    let account = NewAccount {
        date_added: time_now,
        username: None,
        password: None,
        e_mail: None,
        mfa_code: None,
        active: true,
        steam_id: Some(steam_id.steam3()),
    };
    let result: Result<Account, String> =
        insert_db_object_dyn(&conn, account, schema::dsl::accounts);
    result.expect("Failed to create new account for Steam ID")
}
