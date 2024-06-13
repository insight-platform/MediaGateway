use std::sync::Mutex;

use actix_web::{post, web, Responder};

use media_gateway_common::model::Media;

use crate::server::service::GatewayService;

#[post("/")]
async fn gateway(
    service: web::Data<Mutex<GatewayService>>,
    query: web::Json<Media>,
) -> impl Responder {
    let mut gateway_service = service.lock().unwrap();
    gateway_service.process(query)
}
