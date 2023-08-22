use actix_web::{web, Scope};

mod auth;
mod create;
mod get;
mod update;

pub fn config() -> Scope {
    web::scope("/player")
        .route("/{player_id}/", web::get().to(get::action))
        .route("/{player_id}/", web::patch().to(update::action))
        .route("/", web::post().to(create::action))
}
