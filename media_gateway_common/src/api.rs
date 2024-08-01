use actix_web::http::header::ContentType;
use actix_web::{web, HttpResponse, Responder};

use crate::health::HealthService;

pub async fn health(service: web::Data<HealthService>) -> impl Responder {
    let health_state = service.current_state();
    let body = serde_json::to_string(&health_state).unwrap();

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(body)
}
