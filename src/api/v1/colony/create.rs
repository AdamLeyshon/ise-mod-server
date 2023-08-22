use std::convert::TryFrom;

use crate::request_helpers::*;
use actix_web::*;
use chrono::Utc;

use crate::db::models::bind::ClientBind;
use crate::db::models::colony::Colony;
use crate::db::schema::colonies as schema;
use crate::db::{get_pg_connection, insert_db_object};

use crate::packets::colony::{ColonyCreateRequest, ColonyData, PlatformEnum};
use uuid::Uuid;

pub async fn action_create(
    _req: HttpRequest,
    bind: ClientBind,
    request: ProtoBuf<ColonyCreateRequest>,
) -> Result<HttpResponse> {
    if let Some(mut incoming) = request.0.data {
        if incoming.name.is_empty()
            || incoming.faction_name.is_empty()
            || incoming.game_version.is_empty()
            || incoming.tick <= 0
            || incoming.map_id < 0
            || PlatformEnum::try_from(incoming.platform).is_err()
        {
            error!("Unable to create colony using packet: {:?}", &incoming);
            return Ok(HttpResponse::BadRequest().finish());
        }

        let now = Utc::now().naive_utc();

        // We need to generate the UUID here since the ColonyData -> Colony conversion will fail
        // if the UUID string can't be parsed.
        incoming.colony_id = Uuid::new_v4().to_string();

        let mut new_colony: Colony = incoming.into();
        new_colony.create_date = now;
        new_colony.update_date = now;
        new_colony.client_bind_fk = bind.client_bind_id;

        let conn = get_pg_connection();
        let result = insert_db_object(&conn, new_colony, schema::table);
        if result.is_ok() {
            return HttpResponse::Ok().protobuf(ColonyData::from(result.unwrap()));
        };
    };
    error!("Unable to create colony, message payload was invalid");
    Ok(HttpResponse::BadRequest().finish())
}
