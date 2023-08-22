use crate::request_helpers::*;
use actix_web::*;

use crate::db::models::account::Account;
use crate::db::models::bind::ClientBind;
use crate::db::parse_and_load_uuid_pk;
use crate::packets::bind::{ClientBindVerifyReply, ClientBindVerifyRequest};
use crate::steam::ParsableSteamID;
use crate::structs::general::DbPkLoadable;

pub async fn action_verify(proto_msg: ProtoBuf<ClientBindVerifyRequest>) -> Result<HttpResponse> {
    debug!("Verifying Bind ID {}", &proto_msg.client_bind_id);
    if let Ok(bind) = parse_and_load_uuid_pk::<ClientBind>(&*proto_msg.client_bind_id) {
        debug!("Found and parsed Bind ID {}", bind.client_bind_id);
        if bind.confirmed {
            if let Ok(account) = Account::load_pk(&bind.account_fk) {
                let ok_response =
                    HttpResponse::Ok().protobuf(ClientBindVerifyReply { valid: true });
                if account.steam_id.is_none() && proto_msg.steam_id.is_empty() {
                    return ok_response;
                } else if account
                    .steam_id
                    .to_steam_id()
                    .zip(proto_msg.steam_id.to_steam_id())
                    .map_or(false, |zipped| zipped.0 == zipped.1)
                {
                    /*
                    This is only true if account.steam_id and proto_msg.steam_id are both present,
                    both generate valid Steam ID's and are the same Steam ID.
                    Otherwise it's false if:
                    1. either of them are empty
                    2. either of them does not parse into a valid Steam ID
                    3. they both parsed, but they are not equal to each other.
                     */
                    return ok_response;
                } else {
                    debug!(
                        "Bind ID {} does not belong to '{}' or was not provided ",
                        bind.client_bind_id, &proto_msg.steam_id
                    );
                }
            }
        } else {
            debug!("Bind ID {} is not verified", bind.client_bind_id);
        }
    } else {
        debug!(
            "Bind ID {} was not found or could not be validated",
            &*proto_msg.client_bind_id
        );
    };
    HttpResponse::Ok().protobuf(ClientBindVerifyReply { valid: false })
}
