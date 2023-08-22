use crate::request_helpers::*;

use actix_web::*;

use crate::db::models::bind::ClientBind;

use crate::structs::colony::validate_ownership_and_fetch;

use crate::packets::colony::{ColonyData, ColonyGetRequest};

pub async fn action_get(
    _req: HttpRequest,
    bind: ClientBind,
    packet: ProtoBuf<ColonyGetRequest>,
) -> Result<HttpResponse> {
    if let Some(colony) = validate_ownership_and_fetch(None, Some(&packet.colony_id), &bind) {
        HttpResponse::Ok().protobuf(ColonyData::from(colony))
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}
