use actix_protobuf::ProtoBuf;
use actix_web::web::{Data, ReqData};
use actix_web::Responder;
use media_gateway_common::model::Media;
use tokio::sync::Mutex;

use crate::server::service::gateway::GatewayService;
use crate::server::service::user::UserData;

pub async fn gateway(
    service: Data<Mutex<GatewayService>>,
    media: ProtoBuf<Media>,
    user_data: Option<ReqData<UserData>>,
) -> impl Responder {
    let gateway_service = service.lock().await;
    gateway_service.process(media, user_data)
}
