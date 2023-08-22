use crate::request_helpers::*;
use actix_web::*;

use crate::packets::bind::bind_reply::BindErrorReason;
use crate::packets::bind::{BindReply, BindRequest};
use crate::steam::{get_steam_id_block_reason, parse_string_to_steam_id};
use crate::structs::binds::generate_account_bind_id;

pub async fn action_bind_request(proto_msg: ProtoBuf<BindRequest>) -> Result<HttpResponse> {
    let steam_id_string: String = proto_msg.0.steam_id;

    if !steam_id_string.is_empty() {
        //
        // STEAM Processing
        //
        match parse_string_to_steam_id(&steam_id_string) {
            Ok(value) => {
                // Check if Steam ID is blocked.
                let reason = get_steam_id_block_reason(value);
                if reason != BindErrorReason::None {
                    HttpResponse::Ok().protobuf(BindReply {
                        bind_id: "".to_string(),
                        valid: false,
                        reason: reason.into(),
                    })
                } else {
                    // Generate a Bind ID and send it.
                    if let Ok(bind) = generate_account_bind_id(Some(value)) {
                        HttpResponse::Ok().protobuf(BindReply {
                            bind_id: bind.bind_id.to_string(),
                            valid: true,
                            reason: reason.into(),
                        })
                    } else {
                        HttpResponse::Ok().protobuf(BindReply {
                            bind_id: "".to_string(),
                            valid: false,
                            reason: BindErrorReason::Unknown.into(),
                        })
                    }
                }
            }
            Err(_error) => HttpResponse::Ok().protobuf(BindReply {
                bind_id: "".to_string(),
                valid: false,
                reason: BindErrorReason::SteamIdInvalid.into(),
            }),
        }
    } else {
        //
        // DRM-Free Processing
        //
        // Generate a Bind ID and send it, the client will then kick them to the Web UI to complete
        // OAUTH or Steam sign in, create the account and verify the bind ID
        if let Ok(bind) = generate_account_bind_id(None) {
            HttpResponse::Ok().protobuf(BindReply {
                bind_id: bind.bind_id.to_string(),
                valid: true,
                reason: BindErrorReason::None.into(),
            })
        } else {
            HttpResponse::Ok().protobuf(BindReply {
                bind_id: "".to_string(),
                valid: false,
                reason: BindErrorReason::Unknown.into(),
            })
        }
    }
}
