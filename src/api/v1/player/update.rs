use actix_web::web;

pub async fn action(info: web::Path<String>) -> String {
    format!("Patch {}", info)
}
