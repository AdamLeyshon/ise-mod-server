// use crate::request_helpers::*;
// use actix_web::*;
// use actix_web::{web, HttpResponse};
// use chrono::{Duration, Utc};
// use diesel::{sql_query, ExpressionMethods, RunQueryDsl};
// use itertools::Itertools;
// use uuid::Uuid;
//
// use crate::crypto::{generate_random_alphanum_string, generate_v4_uuid, parse_uuid};
// use crate::db::models::bind::ClientBind;
// use crate::db::models::colony_tradable::ColonyTradables;
// use crate::db::models::inventory::Inventory;
// use crate::db::models::inventory_promise::InventoryPromise;
// use crate::db::{get_pg_connection, Ppc};
// use crate::packets::inventory::{
//     ActivatePromiseReply, ActivatePromiseRequest, InventoryReply, InventoryRequest,
// };
// use crate::packets::tradable::Tradable;
// use crate::structs::api_config::LockedApiConfig;
// use crate::structs::bank_balance::get_bank_balance;
// use crate::structs::binds::ClientIdGuard;
// use crate::structs::colony::validate_ownership_and_fetch;
// use crate::structs::general::DbPkLoadable;
// use crate::traits::item::MakeTradable;
// use actix_web::web::Data;
//
// pub fn config() -> Scope {
//     web::scope("/contracts")
//         .guard(guard::Header("content-type", "application/protobuf"))
//         .guard(ClientIdGuard())
//         .route("/", web::post().to(action_post))
// }
//
// pub async fn action_post(
//     req: HttpRequest,
//     bind: ClientBind,
//     packet: ProtoBuf<InventoryRequest>,
// ) -> Result<HttpResponse> {
//     let colony = match validate_ownership_and_fetch(None, Some(&packet.colony_id), &bind) {
//         None => {
//             return Ok(HttpResponse::BadRequest().finish());
//         }
//         Some(value) => value,
//     };
//
//     // Load the relevant colony data
//     let colony_tradables = ColonyTradables::load_pk(&colony.colony_id);
//
//     // Do we have a tradables list for this Colony?
//     if colony_tradables.is_err() {
//         return Ok(HttpResponse::UnprocessableEntity().finish());
//     }
//
//     // Unwrap data and get current time
//     let colony_tradables = colony_tradables.unwrap();
//
//     let now = Utc::now().naive_utc();
//
//     // Make sure they updated the tradables recently.
//     if (colony_tradables.update_date - now).num_minutes() > 5 {
//         return Ok(HttpResponse::UnprocessableEntity().finish());
//     }
//
//     let conn = &get_pg_connection();
//
//     // Create new promise
//     // TODO: Configuration options for expiry times
//     let promise = generate_promise(colony.colony_id, conn);
//
//     // Load tradable data from Inventory table
//     let tradables = get_inventory_for_colony(colony.colony_id, conn, promise.private_key.clone());
//
//     let bank_balance = if let Ok(balance) = get_bank_balance(colony.colony_id, 0, conn) {
//         balance
//     } else {
//         return Ok(HttpResponse::InternalServerError().finish());
//     };
//
//     let lock = req.app_data::<Data<LockedApiConfig>>().unwrap().read();
//     let config = lock.as_ref().unwrap();
//
//     // TODO: Configuration options for charges
//     HttpResponse::Ok().protobuf(InventoryReply {
//         items: tradables,
//         inventory_promise_id: promise.promise_id.to_string(),
//         inventory_promise_expires: promise.expiry_date.timestamp(),
//         collection_charge_per_kg: config.config_data.delivery.collect_cost_per_kg as i32,
//         delivery_charge_per_kg: config.config_data.delivery.delivery_cost_per_kg as i32,
//         account_balance: bank_balance.balance,
//     })
// }
