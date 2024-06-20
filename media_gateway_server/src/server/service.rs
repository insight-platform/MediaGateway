use actix_protobuf::ProtoBuf;
use actix_web::HttpResponse;
use savant_core::message::Message;
use savant_core::transport::zeromq::{SyncWriter, WriterResult};
use twelf::reexports::log;
use twelf::reexports::log::error;

use media_gateway_common::model::Media;

use crate::server::configuration::GatewayConfiguration;

const LOG_ENTRY: &str = "gateway service";

#[derive(Clone)]
pub struct GatewayService {
    writer: SyncWriter,
}

impl GatewayService {
    pub fn new(writer: SyncWriter) -> Self {
        Self { writer }
    }
    pub fn process(&self, media: ProtoBuf<Media>) -> HttpResponse {
        let topic_result = std::str::from_utf8(&media.topic);
        if topic_result.is_err() {
            return HttpResponse::BadRequest().finish();
        }
        let topic = topic_result.unwrap();

        if media.message.is_none() {
            return HttpResponse::BadRequest().finish();
        }
        let message_result = Message::try_from(media.message.as_ref().unwrap());
        if message_result.is_err() {
            return HttpResponse::BadRequest().finish();
        }
        let message = message_result.unwrap();

        log::debug!(
            target: LOG_ENTRY,
            "Received message: topic: {}, message: {:?}, data: len={}",
            topic,
            message,
            media.data.len()
        );

        let data = media
            .data
            .iter()
            .map(|v| v.as_slice())
            .collect::<Vec<&[u8]>>();

        let result = self.writer.send_message(&topic, &message, &data);
        match result {
            Ok(WriterResult::SendTimeout) => HttpResponse::GatewayTimeout().finish(),
            Ok(WriterResult::AckTimeout(_)) => HttpResponse::BadGateway().finish(),
            Ok(WriterResult::Ack { .. }) => HttpResponse::Ok().finish(),
            Ok(WriterResult::Success { .. }) => HttpResponse::Ok().finish(),
            Err(e) => {
                error!(
                    target: LOG_ENTRY,
                    "Failed to send a message: {:?}",
                    e
                );
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}

impl TryFrom<&GatewayConfiguration> for GatewayService {
    type Error = anyhow::Error;

    fn try_from(configuration: &GatewayConfiguration) -> anyhow::Result<Self> {
        let writer = SyncWriter::try_from(&configuration.out_stream)?;
        Ok(GatewayService::new(writer))
    }
}
