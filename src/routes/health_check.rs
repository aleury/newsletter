use actix_web::HttpResponse;

#[tracing::instrument]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}
