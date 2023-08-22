use actix_web::{guard, web, Scope};

pub mod bind;
pub mod bind_verify;
pub mod confirm_bind;

pub fn config() -> Scope {
    web::scope("/binder")
        .guard(guard::Header("content-type", "application/protobuf"))
        .route(
            "/bind",
            web::route()
                .guard(guard::Post())
                .to(bind::action_bind_request),
        )
        .route(
            "/bind_confirm",
            web::route()
                .guard(guard::Post())
                .to(confirm_bind::action_confirm_bind),
        )
        .route(
            "/bind_verify",
            web::route()
                .guard(guard::Post())
                .to(bind_verify::action_verify),
        )
}
