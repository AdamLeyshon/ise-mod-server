use std::ops::Deref;

use crate::request_helpers::*;
use actix_web::http::StatusCode;
use actix_web::*;
use chrono::Utc;
use diesel::SaveChangesDsl;

use crate::db::get_pg_connection;
use crate::db::models::bind::ClientBind;
use crate::db::models::colony::Colony;
use crate::packets::colony::{ColonyData, ColonyUpdateRequest};
use crate::structs::colony::{validate_ownership_and_fetch, Anticheat};
//use http_api_problem::*;

pub async fn action_update(
    _req: HttpRequest,
    bind: ClientBind,
    packet: ProtoBuf<ColonyUpdateRequest>,
) -> Result<HttpResponse> {
    if let Some(incoming) = packet.0.data {
        if let Some(mut colony) =
            validate_ownership_and_fetch(None, Some(&incoming.colony_id), &bind)
        {
            if incoming.tick > 0 {
                if incoming.tick < colony.tick {
                    warn!(
                        "Time-warped colony {}, tick value less than the last one ({} < {})",
                        &colony.colony_id, incoming.tick, colony.tick
                    );
                    if colony.timewarp(incoming.tick).is_err() {
                        error!(
                            "Couldn't rollback orders for Colony {}, ",
                            &colony.colony_id
                        );
                        // let res = HttpApiProblem::new(StatusCode::INTERNAL_SERVER_ERROR)
                        //     .title("Unable to update colony")
                        //     .detail("Error whilst undoing orders,")
                        //     .instance(format!("{}", &incoming.colony_id))
                        //     .to_actix_response().into();
                        // return Ok(res);
                        return Ok(
                            HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).finish()
                        );
                    }
                }
                colony.tick = incoming.tick
            } else {
                // Tick must always be positive
                error!(
                    "Colony {} provided a negative tick number",
                    &colony.colony_id
                );
                // return Ok(HttpApiProblem::new(StatusCode::BAD_REQUEST)
                //     .title("Unable to update colony")
                //     .detail("Tick must be positive integer")
                //     .instance(format!("{}", &incoming.colony_id))
                //     .to_actix_response());
                return Ok(HttpResponseBuilder::new(StatusCode::BAD_REQUEST).finish());
            }
            if incoming.used_dev_mode {
                // Blow the fuse, can never be unset.
                // TODO: Anti-cheat code
                warn!("Marked colony {} as a cheater", &colony.colony_id);
                colony.used_dev_mode = true;
            }
            if !incoming.game_version.is_empty() {
                colony.game_version = incoming.game_version
            };
            colony.update_date = Utc::now().naive_utc();
            let conn = get_pg_connection();
            if let Ok(result) = colony.save_changes::<Colony>(conn.deref()) {
                HttpResponse::Ok().protobuf(ColonyData::from(result))
            } else {
                error!("Couldn't save changes to colony {}", &bind.client_bind_id);
                Ok(HttpResponse::InternalServerError().finish())
            }
        } else {
            error!(
                "Couldn't find Colony ID {} or it doesn't belong to {}",
                &incoming.colony_id, &bind.client_bind_id
            );
            Ok(HttpResponse::BadRequest().finish())
        }
    } else {
        error!("Unable to update colony, message payload was invalid");
        Ok(HttpResponse::BadRequest().finish())
    }
}
