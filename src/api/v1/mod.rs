use actix_web::{web, Scope};

pub mod backorder;
pub mod bank;
pub mod binder;
pub mod colony;
pub mod contracts;
pub mod inventory;
pub mod order;
pub mod player;
pub mod system;
pub mod utilities;

pub fn config() -> Scope {
    web::scope("/v1")
        .service(system::config())
        .service(colony::config())
        .service(order::config())
        .service(inventory::config())
        .service(player::config())
        .service(binder::config())
        .service(bank::config())
}
