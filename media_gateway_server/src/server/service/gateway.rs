use actix_protobuf::ProtoBuf;
use actix_web::web::ReqData;
use actix_web::HttpResponse;
use log::{debug, error, info};
use opentelemetry::trace::{Status, TraceContextExt};
use opentelemetry::Context;
use savant_core::message::Message;
use savant_core::transport::zeromq::{SyncWriter, WriterResult};

use media_gateway_common::model::Media;
use media_gateway_common::statistics::StatisticsService;

use crate::server::configuration::GatewayConfiguration;
use crate::server::service::user::UserData;

const STAT_STAGE_NAME: &str = "server-relay";

pub struct GatewayService {
    writer: SyncWriter,
    statistics_service: Option<StatisticsService>,
}

impl GatewayService {
    pub fn new(writer: SyncWriter, statistics_service: Option<StatisticsService>) -> Self {
        Self {
            writer,
            statistics_service,
        }
    }
    pub fn process(
        &self,
        media: ProtoBuf<Media>,
        user_data: Option<ReqData<UserData>>,
    ) -> HttpResponse {
        let ctx = Context::current();
        let span = ctx.span();

        let topic_result = std::str::from_utf8(&media.topic);
        if topic_result.is_err() {
            span.set_status(Status::error("invalid topic"));
            return HttpResponse::BadRequest().finish();
        }
        let topic = topic_result.unwrap();

        if media.message.is_none() {
            span.set_status(Status::error("no message"));
            return HttpResponse::BadRequest().finish();
        }
        let message_result = Message::try_from(media.message.as_ref().unwrap());
        if message_result.is_err() {
            span.set_status(Status::error("invalid message"));
            return HttpResponse::BadRequest().finish();
        }
        let id = match self.statistics_service.as_ref() {
            Some(service) => match service.register_message_start() {
                Ok(id) => Some(id),
                Err(e) => {
                    log::warn!("Error while starting message statistics: {:?}", e);
                    None
                }
            },
            None => None,
        };

        let message = message_result.unwrap();

        debug!(
            "Received message: topic: {}, message: {:?}, data: len={}",
            topic,
            message,
            media.data.len()
        );

        if let Some(user_data) = user_data {
            if user_data.allowed_routing_labels.is_some()
                && !user_data
                    .allowed_routing_labels
                    .as_ref()
                    .unwrap()
                    .matches(&message.meta().routing_labels)
            {
                span.set_status(Status::error("forbidden by user allowed routing labels"));
                return HttpResponse::Unauthorized().finish();
            }
        }

        let data = media
            .data
            .iter()
            .map(|v| v.as_slice())
            .collect::<Vec<&[u8]>>();

        let result = self.writer.send_message(topic, &message, &data);
        let (response, status) = match result {
            Ok(WriterResult::SendTimeout) => (
                HttpResponse::GatewayTimeout().finish(),
                Status::error(format!("{:?}", result)),
            ),
            Ok(WriterResult::AckTimeout(_)) => (
                HttpResponse::BadGateway().finish(),
                Status::error(format!("{:?}", result)),
            ),
            Ok(WriterResult::Ack { .. }) => (HttpResponse::Ok().finish(), Status::Ok),
            Ok(WriterResult::Success { .. }) => (HttpResponse::Ok().finish(), Status::Ok),
            Err(e) => {
                error!("Failed to send a message: {:?}", e);
                if span.is_recording() {
                    span.record_error(e.as_ref());
                }
                (
                    HttpResponse::InternalServerError().finish(),
                    Status::error("store failure"),
                )
            }
        };
        if let Some(stat_id) = id {
            if let Err(e) = self
                .statistics_service
                .as_ref()
                .unwrap()
                .register_message_end(stat_id)
            {
                log::warn!("Error while ending message statistics: {:?}", e)
            }
        }
        span.set_status(status);
        response
    }
}

impl TryFrom<&GatewayConfiguration> for GatewayService {
    type Error = anyhow::Error;

    fn try_from(configuration: &GatewayConfiguration) -> anyhow::Result<Self> {
        let writer = SyncWriter::try_from(&configuration.out_stream)?;
        let statistics_service = if let Some(statistics_config) = &configuration.statistics {
            Some(StatisticsService::try_from((
                statistics_config,
                STAT_STAGE_NAME,
            ))?)
        } else {
            None
        };
        Ok(GatewayService::new(writer, statistics_service))
    }
}

impl Drop for GatewayService {
    fn drop(&mut self) {
        info!("Shutting down writer");
        let result = self.writer.shutdown();
        if result.is_err() {
            error!("Failed to shutdown writer {:?}", result.unwrap_err());
        }
    }
}
#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use actix_protobuf::ProtoBuf;
    use actix_web::http::StatusCode;
    use actix_web::web::ReqData;
    use actix_web::{FromRequest, HttpMessage};
    use rand::Rng;
    use savant_core::message::label_filter::LabelFilterRule;
    use savant_core::message::Message;
    use savant_core::transport::zeromq::{
        ReaderConfigBuilder, ReaderResult, SyncReader, SyncWriter, WriterConfigBuilder,
    };

    use media_gateway_common::model::Media;

    use crate::server::service::gateway::GatewayService;
    use crate::server::service::user::UserData;

    #[test]
    fn process_invalid_topic() {
        let message = new_message();
        let media = Media {
            message: Option::from(savant_protobuf::generated::Message::from(&message)),
            topic: vec![0, 159, 146, 150],
            data: vec![],
        };
        let service = new_service();

        let response = service.process(ProtoBuf(media), None);

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn process_empty_message() {
        let media = Media {
            message: None,
            topic: "topic".as_bytes().to_vec(),
            data: vec![vec![1]],
        };
        let service = new_service();

        let response = service.process(ProtoBuf(media), None);

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn process_ok_success() {
        let (message, media) = new_message_and_media();
        let ipc = new_ipc();
        let service = new_service_with_url(format!("pub+bind:{}", ipc).as_str());
        let reader = new_reader(format!("sub+connect:{}", ipc).as_str());

        // timeout to connect writer and reader
        thread::sleep(Duration::from_secs(1));

        let response = service.process(ProtoBuf(media.clone()), None);

        assert_eq!(response.status(), StatusCode::OK);

        let reader_result = reader.receive();
        check_reader_result_message(&reader_result, &message, &media.topic, &media.data);
        reader.shutdown().expect("reader shutdown failure");
    }

    #[test]
    fn process_ok_ack() {
        let (message, media) = new_message_and_media();
        let topic = media.topic.clone();
        let data = media.data.clone();
        let tcp = new_tcp();
        let service = new_service_with_url(format!("req+connect:{}", tcp).as_str());
        let reader_thread = thread::spawn(move || {
            let reader = new_reader(format!("rep+bind:{}", &tcp).as_str());
            let reader_result = reader.receive();
            reader.shutdown().unwrap();
            check_reader_result_message(&reader_result, &message, &topic, &data);
        });

        let response = service.process(ProtoBuf(media.clone()), None);

        assert_eq!(response.status(), StatusCode::OK);

        reader_thread.join().expect("reader thread join failure");
    }

    #[test]
    fn process_bad_gateway() {
        let (_, media) = new_message_and_media();
        let service = new_service_with_url(format!("req+connect:{}", new_tcp()).as_str());

        let response = service.process(ProtoBuf(media.clone()), None);

        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    }

    #[test]
    fn process_no_user_labels() {
        process_labels(None, StatusCode::OK)
    }

    #[test]
    fn process_invalid_labels() {
        process_labels(
            Some(LabelFilterRule::Set("label".to_string())),
            StatusCode::UNAUTHORIZED,
        )
    }

    fn process_labels(
        user_label_filter_rule: Option<LabelFilterRule>,
        expected_status: StatusCode,
    ) {
        let (_message, media) = new_message_and_media();
        let user_data = UserData {
            password_hash: "".to_string(),
            allowed_routing_labels: user_label_filter_rule,
        };
        let request = actix_web::test::TestRequest::default().to_srv_request();
        request.extensions_mut().insert(user_data);

        let user_data = futures::executor::block_on(ReqData::extract(request.request())).unwrap();
        let ipc = new_ipc();
        let service = new_service_with_url(format!("pub+bind:{}", ipc).as_str());

        let response = service.process(ProtoBuf(media.clone()), Some(user_data));

        assert_eq!(response.status(), expected_status);
    }

    fn new_service() -> GatewayService {
        new_service_with_url(format!("pub+bind:{}", new_ipc()).as_str())
    }

    fn new_service_with_url(url: &str) -> GatewayService {
        let writer = SyncWriter::new(
            &WriterConfigBuilder::default()
                .url(url)
                .unwrap()
                .build()
                .unwrap(),
        )
        .unwrap();
        writer.is_started();
        GatewayService {
            writer,
            statistics_service: None,
        }
    }

    fn new_reader(url: &str) -> SyncReader {
        let reader = SyncReader::new(
            &ReaderConfigBuilder::default()
                .url(url)
                .unwrap()
                .build()
                .unwrap(),
        )
        .unwrap();
        reader.is_started();
        reader
    }

    fn new_ipc() -> String {
        format!("ipc:///tmp/test{}", rand::random::<u16>())
    }

    fn new_tcp() -> String {
        format!(
            "tcp://127.0.0.1:{}",
            rand::thread_rng().gen_range(40000..50000)
        )
    }

    fn new_message() -> Message {
        Message::unknown("message".to_string())
    }

    fn new_message_and_media() -> (Message, Media) {
        let message = new_message();
        let media = Media {
            message: Option::from(savant_protobuf::generated::Message::from(&message)),
            topic: "topic".as_bytes().to_vec(),
            data: vec![vec![1]],
        };
        (message, media)
    }

    fn check_reader_result_message(
        reader_result: &anyhow::Result<ReaderResult>,
        message: &Message,
        topic: &Vec<u8>,
        data: &Vec<Vec<u8>>,
    ) {
        match reader_result {
            Ok(ReaderResult::Message {
                message: reader_message,
                topic: reader_topic,
                data: reader_data,
                ..
            }) => {
                assert_eq!(message.meta().seq_id, reader_message.meta().seq_id);
                assert_eq!(
                    message.meta().routing_labels,
                    reader_message.meta().routing_labels
                );
                assert_eq!(
                    message.meta().protocol_version,
                    reader_message.meta().protocol_version
                );
                assert_eq!(
                    message.meta().span_context.0,
                    reader_message.meta().span_context.0
                );
                assert_eq!(message.as_unknown(), reader_message.as_unknown());
                assert_eq!(topic, reader_topic);
                assert_eq!(data, reader_data);
            }
            _ => panic!("Unexpected reader result: {:?}", reader_result),
        };
    }
}
