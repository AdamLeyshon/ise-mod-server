use actix_web::{guard, web, Scope};

pub mod hello;

pub fn config() -> Scope {
    web::scope("/system").route(
        "/hello",
        web::route()
            .guard(guard::Post())
            .guard(guard::Header("content-type", "application/protobuf"))
            .to(hello::action_protobuf),
    )
}
