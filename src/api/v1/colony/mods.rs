use std::ops::Deref;

use crate::request_helpers::*;
use actix_web::{HttpRequest, HttpResponse, Result};

use diesel::SaveChangesDsl;

use crate::db::models::bind::ClientBind;
use crate::db::models::colony_mod::ColonyMods;
use crate::db::models::StringVec;
use crate::db::schema::colony_mods as schema;
use crate::db::{get_pg_connection, insert_db_object, parse_and_load_uuid_pk};
use crate::structs::colony::validate_ownership_and_fetch;

use crate::packets::colony::ColonyModsSetRequest;

pub async fn action_post(
    _req: HttpRequest,
    bind: ClientBind,
    packet: ProtoBuf<ColonyModsSetRequest>,
) -> Result<HttpResponse> {
    let incoming = packet.0;
    if let Some(colony) = validate_ownership_and_fetch(None, Some(&incoming.colony_id), &bind) {
        let conn = &get_pg_connection();
        let mods = incoming.mod_name;
        if let Ok(mut cm) = parse_and_load_uuid_pk::<ColonyMods>(&*incoming.colony_id) {
            cm.mods = StringVec::from(mods);
            if let Err(_) = cm.save_changes::<ColonyMods>(conn.deref()) {
                return Ok(HttpResponse::InternalServerError().finish());
            }
        } else {
            let cm = ColonyMods {
                colony_id: colony.colony_id,
                mods: mods.into(),
            };
            if let Err(_) = insert_db_object(conn, cm, schema::table) {
                return Ok(HttpResponse::InternalServerError().finish());
            }
        };
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::BadRequest().finish())
    }
}
