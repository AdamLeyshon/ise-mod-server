use actix_web::web;

pub mod v1;
pub mod v2;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(v1::config())
            .service(v2::config()),
    );
}
