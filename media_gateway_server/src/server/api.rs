use actix_protobuf::ProtoBuf;
use actix_web::{web, Responder};
use tokio::sync::Mutex;

use media_gateway_common::model::Media;

use crate::server::service::gateway::GatewayService;

pub async fn gateway(
    service: web::Data<Mutex<GatewayService>>,
    media: ProtoBuf<Media>,
) -> impl Responder {
    let gateway_service = service.lock().await;
    gateway_service.process(media)
}
