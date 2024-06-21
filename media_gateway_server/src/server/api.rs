use actix_protobuf::ProtoBuf;
use actix_web::http::header::ContentType;
use actix_web::{get, post, web, HttpResponse, Responder};
use tokio::sync::Mutex;

use media_gateway_common::model::Media;

use crate::server::service::gateway::GatewayService;
use crate::server::service::health::HealthService;

#[post("/")]
async fn gateway(
    service: web::Data<Mutex<GatewayService>>,
    media: ProtoBuf<Media>,
) -> impl Responder {
    let gateway_service = service.lock().await;
    gateway_service.process(media)
}

#[get("/health")]
async fn health(service: web::Data<HealthService>) -> impl Responder {
    let health_state = service.current_state();
    let body = serde_json::to_string(&health_state).unwrap();

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(body)
}
