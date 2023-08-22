use actix_web::{web, Scope};
pub mod inventory;

pub fn config() -> Scope {
    web::scope("/v2")
}
