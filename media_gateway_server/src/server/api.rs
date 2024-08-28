use actix_protobuf::ProtoBuf;
use actix_web::web::{Data, ReqData};
use actix_web::{HttpRequest, Responder};
use http::{HeaderMap, HeaderName, HeaderValue};
use opentelemetry::trace::TraceContextExt;
use tokio::sync::Mutex;

use media_gateway_common::model::Media;
use media_gateway_common::telemetry::{get_context_with_span, get_header_context};

use crate::server::service::gateway::GatewayService;
use crate::server::service::user::UserData;

pub async fn gateway(
    service: Data<Mutex<GatewayService>>,
    request: HttpRequest,
    media: ProtoBuf<Media>,
    user_data: Option<ReqData<UserData>>,
) -> impl Responder {
    let mut headers = HeaderMap::new();
    request
        .headers()
        .into_iter()
        .map(|e| e.to_owned())
        .for_each(|e| {
            headers.insert(
                HeaderName::try_from(e.0.as_str()).unwrap(),
                HeaderValue::try_from(e.1.as_bytes()).unwrap(),
            );
        });
    let parent_ctx = get_header_context(&headers);
    let ctx = get_context_with_span("store", &parent_ctx);

    let gateway_service = service.lock().await;
    let result = gateway_service.process(media, user_data);

    ctx.span().end();

    result
}
