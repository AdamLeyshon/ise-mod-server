use crate::request_helpers::ProtoBufConfig;
use actix_web::{guard, web, Scope};

use crate::structs::binds::ClientIdGuard;

mod create;
mod get;
mod mods;
mod tradables;
mod update;

pub fn config() -> Scope {
    let mut large_payload_size = ProtoBufConfig::default();
    large_payload_size.limit(10_000_000);

    web::scope("/colony")
        .guard(guard::Header("content-type", "application/protobuf"))
        .guard(ClientIdGuard())
        .route("/", web::post().to(create::action_create))
        .route("/get", web::post().to(get::action_get))
        .route("/", web::patch().to(update::action_update))
        .app_data(large_payload_size)
        .route("/mods", web::post().to(mods::action_post))
        .route("/tradables", web::post().to(tradables::action_post))
        .route("/tradables", web::patch().to(tradables::action_post))
}
