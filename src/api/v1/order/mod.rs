use actix_web::{web, Scope};

use crate::structs::binds::ClientIdGuard;

pub mod get;
pub mod get_list;
pub mod get_manifest;
pub mod place;
pub mod update;

pub fn config() -> Scope {
    web::scope("/order")
        .guard(ClientIdGuard())
        .route("/", web::post().to(get::action_get))
        .route("/place", web::post().to(place::action_post))
        .route("/list", web::post().to(get_list::action_get))
        .route("/update", web::post().to(update::action_update))
        .route("/manifest", web::post().to(get_manifest::action_get))
}
