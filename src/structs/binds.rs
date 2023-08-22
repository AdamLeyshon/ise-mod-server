use std::io;
use std::io::ErrorKind;

use actix_http::error::PayloadError;
use actix_http::{BoxedPayloadStream, Error, HttpMessage, Payload};

use actix_web::guard::{Guard, GuardContext};
use actix_web::{FromRequest, HttpRequest};
use chrono::{Duration, NaiveDateTime, Utc};
use diesel::prelude::*;
use futures::future::{err, ok, Ready};

use steamid_ng::SteamID;
use uuid::Uuid;

use crate::crypto::generate_v4_uuid;
use crate::db::models::account::Account;
use crate::db::models::bind::{AccountBind, ClientBind};
use crate::db::{get_pg_connection, insert_db_object, parse_and_load_uuid_pk};
use crate::structs::account::find_account_for_steam_id;
use crate::structs::general::DbPkLoadable;

make_pk_loadable!(AccountBind, Uuid, crate::db::schema::account_binds);
make_pk_loadable!(ClientBind, Uuid, crate::db::schema::client_binds);

pub fn generate_account_bind_id(steam_id: Option<SteamID>) -> Result<AccountBind, ()> {
    let steam_id3 = if let Some(steam) = steam_id {
        Some(steam.steam3())
    } else {
        None
    };

    let time_now = Utc::now().naive_utc();
    let ub = AccountBind {
        steam_id: steam_id3.clone(),
        bind_id: generate_v4_uuid(),
        confirmed: false,
        date_added: time_now.clone(),
        date_expire: time_now + Duration::minutes(5),
        account_fk: None,
    };

    let conn = get_pg_connection();
    use crate::db::schema::account_binds as schema;

    let result = insert_db_object(&conn, ub, schema::dsl::account_binds);

    result.map_or(Err(()), |v| Ok(v))
}

pub fn delete_bind_id(bind_id: Uuid) {
    use crate::db::schema::account_binds as schema;
    let conn = get_pg_connection();
    diesel::delete(schema::table.filter(schema::bind_id.eq(bind_id)))
        .execute(&conn)
        .expect("Failed to delete bind ID");
}

pub fn delete_client_bind_id(bind_id: Uuid) {
    use crate::db::schema::client_binds as schema;
    let conn = get_pg_connection();
    diesel::delete(schema::table.filter(schema::client_bind_id.eq(bind_id)))
        .execute(&conn)
        .expect("Failed to delete client bind ID");
}

pub fn account_bind_valid(current_time: &NaiveDateTime, bind: &AccountBind) -> bool {
    if (*current_time < bind.date_expire) && !bind.confirmed {
        true
    } else {
        // Do a bit of house keeping
        if *current_time > bind.date_expire {
            delete_bind_id(bind.bind_id);
        };
        false
    }
}

pub fn client_bind_valid(_current_time: &NaiveDateTime, bind: &ClientBind) -> bool {
    // It's a stub in case we decide to expand in future
    !bind.confirmed
}

pub fn steam_autogenerate_client_bind_id(
    steam_id: &SteamID,
    account_bind: &AccountBind,
) -> Result<ClientBind, ()> {
    let conn = get_pg_connection();
    use crate::db::schema::account_binds as ab_schema;
    use crate::db::schema::client_binds as cb_schema;

    let account = find_account_for_steam_id(steam_id);

    // Check if the Client ID has already been generated.
    let result: Result<ClientBind, diesel::result::Error> = cb_schema::table
        .filter(cb_schema::account_fk.eq(&account.account_id))
        .filter(cb_schema::confirmed.eq(false))
        .order(cb_schema::date_added.desc())
        .first(&conn);

    if result.is_err() {
        // If not generate one now.
        if result.unwrap_err().eq(&diesel::NotFound) {
            let time_now = Utc::now().naive_utc();
            let ub = ClientBind {
                client_bind_id: generate_v4_uuid(),
                account_fk: account.account_id,
                confirmed: false,
                date_added: time_now.clone(),
            };

            let result: Result<ClientBind, String> =
                insert_db_object(&conn, ub, cb_schema::dsl::client_binds);
            if result.is_ok() {
                // Link the Account to the Account Bind Request.
                diesel::update(
                    ab_schema::table.filter(ab_schema::bind_id.eq(&account_bind.bind_id)),
                )
                .set(ab_schema::account_fk.eq(account.account_id))
                .execute(&conn)
                .expect("Failed to link account bind with Account ID");
                return Ok(result.unwrap());
            };
            Err(())
        } else {
            Err(())
        }
    } else {
        // Return the existing Client Bind ID
        Ok(result.unwrap())
    }
}

pub fn get_non_steam_client_bind_id(account_bind: &AccountBind) -> Result<ClientBind, ()> {
    let conn = get_pg_connection();
    use crate::db::schema::client_binds as cb_schema;

    // Check if the Client ID has already been generated.
    let result: Result<ClientBind, diesel::result::Error> = cb_schema::table
        .filter(cb_schema::account_fk.eq(account_bind.account_fk.unwrap()))
        .filter(cb_schema::confirmed.eq(false))
        .order(cb_schema::date_added.desc())
        .first(&conn);

    result.map_or(Err(()), |v| Ok(v))
}

pub struct ClientIdGuard();

pub const ISE_CLIENT_HEADER_NAME: &'static str = "x-ise-client-id";

impl Guard for ClientIdGuard {
    fn check(&self, ctx: &GuardContext<'_>) -> bool {
        if let Some(val) = ctx.head().headers().get(ISE_CLIENT_HEADER_NAME) {
            if let Ok(bind) = parse_and_load_uuid_pk::<ClientBind>(val.to_str().unwrap_or("")) {
                if let Ok(account) = Account::load_pk(&bind.account_fk) {
                    if account.active && bind.confirmed {
                        ctx.req_data_mut().insert(bind);
                        return true;
                    };
                };
            };
        };
        false
    }
}

impl FromRequest for ClientBind {
    type Error = actix_http::error::Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload<BoxedPayloadStream>) -> Self::Future {
        if req.extensions().contains::<Self>() {
            ok(req.extensions_mut().remove::<Self>().unwrap())
        } else {
            err(Error::from(PayloadError::Incomplete(Some(io::Error::new(
                ErrorKind::Other,
                "No Client ID in request",
            )))))
        }
    }
}
