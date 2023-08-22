use actix_web::*;

pub async fn action(_req: HttpRequest) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
}
