use std::sync::Mutex;
use actix_protobuf::ProtoBuf;

use actix_web::{post, web, Responder};

use media_gateway_common::model::Media;

use crate::server::service::GatewayService;

#[post("/")]
async fn gateway(
    service: web::Data<Mutex<GatewayService>>,
    media: ProtoBuf<Media>
) -> impl Responder {
    let gateway_service = service.lock().unwrap();
    gateway_service.process(media)
}
