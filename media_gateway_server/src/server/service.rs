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

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use actix_protobuf::ProtoBuf;
    use actix_web::http::StatusCode;
    use rand::Rng;
    use savant_core::message::Message;
    use savant_core::transport::zeromq::{ReaderConfigBuilder, ReaderResult, SyncReader, SyncWriter, WriterConfigBuilder};

    use media_gateway_common::model::Media;

    use crate::server::service::GatewayService;

    #[test]
    fn process_invalid_topic() {
        let message = new_message();
        let media = Media {
            message: Option::from(savant_protobuf::generated::Message::from(&message)),
            topic: vec![0, 159, 146, 150],
            data: vec![],
        };
        let service = new_service();

        let response = service.process(ProtoBuf(media));

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

        let response = service.process(ProtoBuf(media));

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

        let response = service.process(ProtoBuf(media.clone()));

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

        let response = service.process(ProtoBuf(media.clone()));

        assert_eq!(response.status(), StatusCode::OK);

        reader_thread.join().expect("reader thread join failure");
    }

    #[test]
    fn process_bad_gateway() {
        let (_, media) = new_message_and_media();
        let service = new_service_with_url(format!("req+connect:{}", new_tcp()).as_str());

        let response = service.process(ProtoBuf(media.clone()));

        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    }

    fn new_service() -> GatewayService {
        new_service_with_url(format!("pub+bind:{}", new_ipc()).as_str())
    }

    fn new_service_with_url(url: &str) -> GatewayService {
        let writer = SyncWriter::new(&WriterConfigBuilder::default()
            .url(url).unwrap()
            .build().unwrap())
            .unwrap();
        writer.is_started();
        GatewayService {
            writer
        }
    }

    fn new_reader(url: &str) -> SyncReader {
        let reader = SyncReader::new(&ReaderConfigBuilder::default()
            .url(url).unwrap()
            .build().unwrap())
            .unwrap();
        reader.is_started();
        reader
    }

    fn new_ipc() -> String {
        format!("ipc:///tmp/test{}", rand::random::<u16>())
    }

    fn new_tcp() -> String {
        format!("tcp://127.0.0.1:{}", rand::thread_rng().gen_range(40000..50000))
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

    fn check_reader_result_message(reader_result: &anyhow::Result<ReaderResult>, message: &Message, topic: &Vec<u8>, data: &Vec<Vec<u8>>) {
        match reader_result {
            Ok(ReaderResult::Message { message: reader_message, topic: reader_topic, data: reader_data, .. }) => {
                assert_eq!(message.meta().seq_id, reader_message.meta().seq_id);
                assert_eq!(message.meta().routing_labels, reader_message.meta().routing_labels);
                assert_eq!(message.meta().protocol_version, reader_message.meta().protocol_version);
                assert_eq!(message.meta().span_context.0, reader_message.meta().span_context.0);
                assert_eq!(message.as_unknown(), reader_message.as_unknown());
                assert_eq!(topic, reader_topic);
                assert_eq!(data, reader_data);
            }
            _ => panic!("Unexpected reader result: {:?}", reader_result)
        };
    }
}
