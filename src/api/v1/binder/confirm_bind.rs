use std::convert::TryFrom;

use crate::request_helpers::*;
use actix_web::*;
use chrono::Utc;

use crate::db::models::bind::{AccountBind, ClientBind};
use crate::db::parse_and_load_uuid_pk;
use crate::packets::bind::BindTypeEnum;
use crate::packets::bind::{ConfirmBindReply, ConfirmBindRequest};
use crate::steam::ParsableSteamID;
use crate::structs::binds::{account_bind_valid, steam_autogenerate_client_bind_id};
use crate::structs::binds::{client_bind_valid, get_non_steam_client_bind_id};
use crate::traits::confirmable::Confirmable;

pub async fn action_confirm_bind(proto_msg: ProtoBuf<ConfirmBindRequest>) -> Result<HttpResponse> {
    let message = proto_msg.0;
    let current_time = Utc::now().naive_utc();
    match BindTypeEnum::try_from(message.bind_type)
        .map_err(|_x| BindTypeEnum::AccountBind)
        .unwrap()
    {
        BindTypeEnum::AccountBind => {
            let bad_bind_response = HttpResponse::Ok().protobuf(ConfirmBindReply {
                bind_type: BindTypeEnum::AccountBind.into(),
                is_valid: false,
                bind_complete: false,
                client_bind_id: String::from(""),
                ttl: 0,
            });

            if let Ok(mut bind_data) = parse_and_load_uuid_pk::<AccountBind>(&*message.bind_id) {
                if !account_bind_valid(&current_time, &bind_data) {
                    return bad_bind_response;
                };

                // Bind exists, hasn't expired and is not confirmed.
                // Calculate TTL of bind, ttl > 0 always at this point,
                // it's checked in account_bind_valid
                let ttl = (bind_data.date_expire - current_time).num_seconds() as i32;

                let mut client_bind_id = String::from("");

                // If a Steam user, automatically confirm the bind, link the account to the bind
                // and generate a client ID.
                if let Some(steam_id) = bind_data.steam_id.to_steam_id() {
                    if let Ok(client_bind) =
                        steam_autogenerate_client_bind_id(&steam_id, &bind_data)
                    {
                        client_bind_id = client_bind.client_bind_id.to_string();
                        bind_data.confirm();
                    }
                } else {
                    // Non-steam bind handling, has it been confirmed by the Web UI?
                    if bind_data.confirmed && bind_data.account_fk.is_some() {
                        let client_bind_data = get_non_steam_client_bind_id(&bind_data);
                        if client_bind_data.is_ok() {
                            client_bind_id = client_bind_data.unwrap().client_bind_id.to_string();
                        } else {
                            // Bind was marked as confirmed but there was no client bind ID set,
                            // Something went horribly wrong (most likely Web UI side)
                            return bad_bind_response;
                        }
                    }
                };

                HttpResponse::Ok().protobuf(ConfirmBindReply {
                    bind_type: BindTypeEnum::AccountBind.into(),
                    is_valid: true,
                    bind_complete: bind_data.confirmed,
                    client_bind_id,
                    ttl,
                })
            } else {
                bad_bind_response
            }
        }
        BindTypeEnum::ClientBind => {
            let bad_bind_response = HttpResponse::Ok().protobuf(ConfirmBindReply {
                bind_type: BindTypeEnum::ClientBind.into(),
                is_valid: false,
                bind_complete: false,
                client_bind_id: String::from(""),
                ttl: 0,
            });

            if let Ok(mut bind_data) = parse_and_load_uuid_pk::<ClientBind>(&*message.bind_id) {
                if !client_bind_valid(&current_time, &bind_data) {
                    return bad_bind_response;
                };
                bind_data.confirm();
                HttpResponse::Ok().protobuf(ConfirmBindReply {
                    bind_type: BindTypeEnum::ClientBind.into(),
                    is_valid: true,
                    bind_complete: bind_data.confirmed,
                    client_bind_id: bind_data.client_bind_id.to_string(),
                    ttl: 0,
                })
            } else {
                bad_bind_response
            }
        }
    }
}
