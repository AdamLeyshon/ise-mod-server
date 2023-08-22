use actix_web::{HttpRequest, HttpResponse};

pub async fn action(_req: HttpRequest) -> HttpResponse {
    HttpResponse::InternalServerError().finish()
}
