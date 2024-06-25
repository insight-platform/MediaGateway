use std::fs::File;
use std::io::Read;

use anyhow::anyhow;
use http_auth_basic::Credentials;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Certificate, Client, StatusCode};
use savant_core::transport::zeromq::{ReaderResult, SyncReader};

use media_gateway_common::model::Media;

use crate::client::ForwardResult::ReadError;
use crate::configuration::GatewayClientConfiguration;

#[derive(Debug)]
pub enum ForwardResult {
    Success,
    SendTimeout,
    AckTimeout,
    ReadError(ReaderResult),
}

pub struct GatewayClient {
    url: String,
    reader: SyncReader,
    client: Client,
}

impl GatewayClient {
    pub fn new(reader: SyncReader, client: Client, url: String) -> Self {
        Self {
            reader,
            client,
            url,
        }
    }

    pub async fn forward_message(&self) -> anyhow::Result<ForwardResult> {
        let reader_result = self.reader.receive()?;
        match reader_result {
            ReaderResult::Message {
                message,
                topic,
                data,
                ..
            } => {
                let media = Media {
                    message: Option::from(savant_protobuf::generated::Message::from(
                        message.as_ref(),
                    )),
                    topic,
                    data,
                };
                let data = media.to_proto()?;
                let send_result = self
                    .client
                    .post(&self.url)
                    .body(data)
                    .header(CONTENT_TYPE, "application/protobuf")
                    .send()
                    .await;
                match send_result {
                    Ok(response) => match response.status() {
                        StatusCode::OK => Ok(ForwardResult::Success),
                        StatusCode::GATEWAY_TIMEOUT => Ok(ForwardResult::SendTimeout),
                        StatusCode::BAD_GATEWAY => Ok(ForwardResult::AckTimeout),
                        status_code => Err(anyhow!("Invalid HTTP status: {}", status_code)),
                    },
                    Err(e) => Err(anyhow!("Error while sending a message").context(e.to_string())),
                }
            }
            _ => Ok(ReadError(reader_result)),
        }
    }

    pub fn shutdown(&self) -> anyhow::Result<()> {
        self.reader.shutdown()
    }
}

impl TryFrom<&GatewayClientConfiguration> for GatewayClient {
    type Error = anyhow::Error;

    fn try_from(configuration: &GatewayClientConfiguration) -> anyhow::Result<Self> {
        let reader = SyncReader::try_from(&configuration.in_stream)?;

        let mut client_builder = Client::builder();

        client_builder = if let Some(ssl_conf) = &configuration.ssl {
            let mut buf = Vec::new();
            File::open(&ssl_conf.certificate)?.read_to_end(&mut buf)?;
            let cert = Certificate::from_pem(&buf)?;

            client_builder.add_root_certificate(cert)
        } else {
            client_builder
        };

        client_builder = if let Some(auth_conf) = &configuration.auth {
            let mut headers = HeaderMap::new();

            let mut auth_value = HeaderValue::from_str(
                &Credentials::new(&auth_conf.basic.id, &auth_conf.basic.password).as_http_header(),
            )?;
            auth_value.set_sensitive(true);
            headers.insert(AUTHORIZATION, auth_value);

            client_builder.default_headers(headers)
        } else {
            client_builder
        };

        Ok(GatewayClient::new(
            reader,
            client_builder.build()?,
            configuration.url.clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use rand::Rng;
    use reqwest::{Client, StatusCode};
    use savant_core::message::Message;
    use savant_core::transport::zeromq::{
        ReaderConfigBuilder, ReaderResult, SyncReader, SyncWriter, WriterConfigBuilder,
        WriterResult,
    };
    use wiremock::matchers::{body_bytes, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use media_gateway_common::model::Media;

    use crate::client::{ForwardResult, GatewayClient};

    #[tokio::test]
    async fn forward_message_success() {
        forward_test(Some(StatusCode::OK), Ok(ForwardResult::Success)).await
    }

    #[tokio::test]
    async fn forward_message_send_timeout() {
        forward_test(
            Some(StatusCode::GATEWAY_TIMEOUT),
            Ok(ForwardResult::SendTimeout),
        )
        .await
    }

    #[tokio::test]
    async fn forward_message_ack_timeout() {
        forward_test(Some(StatusCode::BAD_GATEWAY), Ok(ForwardResult::AckTimeout)).await
    }

    #[tokio::test]
    async fn forward_message_invalid_http_status() {
        let http_status = StatusCode::BAD_REQUEST;
        forward_test(
            Some(http_status),
            Err(anyhow!("Invalid HTTP status: {}", http_status)),
        )
        .await
    }

    #[tokio::test]
    async fn forward_message_http_error() {
        forward_test(None, Err(anyhow!("Error while sending a message"))).await
    }

    #[tokio::test]
    async fn forward_message_read_error() {
        let ipc = format!("ipc:///tmp/test{}", rand::random::<u16>());
        let reader = SyncReader::new(
            &ReaderConfigBuilder::default()
                .url(format!("sub+connect:{}", ipc).as_str())
                .unwrap()
                .build()
                .unwrap(),
        )
        .unwrap();

        let gateway_url = format!(
            "http://127.0.0.1:{}/",
            rand::thread_rng().gen_range(40000..50000)
        );
        let client = GatewayClient::new(reader, Client::default(), gateway_url);
        let actual_result = client.forward_message().await;
        assert!(matches!(
            actual_result,
            Ok(ForwardResult::ReadError(ReaderResult::Timeout))
        ));
    }

    async fn forward_test(
        http_status: Option<StatusCode>,
        expected_result: anyhow::Result<ForwardResult>,
    ) {
        let message = Message::unknown("message".to_string());
        let topic = "topic";
        let data: Vec<&[u8]> = vec![&[1]];
        let media = Media {
            message: Option::from(savant_protobuf::generated::Message::from(&message)),
            topic: topic.as_bytes().to_vec(),
            data: data.iter().map(|e| e.to_vec()).collect::<Vec<Vec<u8>>>(),
        };

        let gateway_path = "/";
        let gateway_url = if let Some(status) = http_status {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path(gateway_path))
                .and(body_bytes(
                    media.to_proto().expect("http mock body setup failed"),
                ))
                .respond_with(ResponseTemplate::new(status))
                .mount(&server)
                .await;
            server.uri() + gateway_path
        } else {
            format!(
                "http://127.0.0.1:{}{}",
                rand::thread_rng().gen_range(40000..50000),
                gateway_path
            )
        };

        let ipc = format!("ipc:///tmp/test{}", rand::random::<u16>());
        let writer = SyncWriter::new(
            &WriterConfigBuilder::default()
                .url(format!("pub+bind:{}", ipc).as_str())
                .unwrap()
                .build()
                .unwrap(),
        )
        .unwrap();

        let reader = SyncReader::new(
            &ReaderConfigBuilder::default()
                .url(format!("sub+connect:{}", ipc).as_str())
                .unwrap()
                .build()
                .unwrap(),
        )
        .unwrap();

        let client = GatewayClient::new(reader, Client::default(), gateway_url);

        let write_result = writer
            .send_message(topic, &message, &data)
            .expect("writing failed");
        assert!(matches!(write_result, WriterResult::Success { .. }));

        let actual_result = client.forward_message().await;

        writer.shutdown().unwrap();
        client.reader.shutdown().unwrap();

        match expected_result {
            Ok(expected_forward_result) => {
                assert!(actual_result.is_ok());
                match actual_result.unwrap() {
                    ForwardResult::Success => {
                        assert!(matches!(expected_forward_result, ForwardResult::Success))
                    }
                    ForwardResult::SendTimeout => assert!(matches!(
                        expected_forward_result,
                        ForwardResult::SendTimeout
                    )),
                    ForwardResult::AckTimeout => {
                        assert!(matches!(expected_forward_result, ForwardResult::AckTimeout))
                    }
                    ForwardResult::ReadError(reader_result) => {
                        if let ForwardResult::ReadError(expected_read_error) =
                            expected_forward_result
                        {
                            match reader_result {
                                ReaderResult::Message { .. } => panic!(
                                    "Expected: {:?} but got ReaderResult::Message{{..}}",
                                    expected_read_error
                                ),
                                ReaderResult::Timeout => {
                                    assert!(matches!(expected_read_error, ReaderResult::Timeout))
                                }
                                ReaderResult::PrefixMismatch { topic, routing_id } => {
                                    let actual_topic = topic;
                                    let actual_routing_id = routing_id;
                                    assert!(matches!(
                                        expected_read_error,
                                        ReaderResult::PrefixMismatch {topic, routing_id}
                                            if topic == actual_topic
                                                && routing_id == actual_routing_id
                                    ))
                                }
                                ReaderResult::RoutingIdMismatch { topic, routing_id } => {
                                    let actual_topic = topic;
                                    let actual_routing_id = routing_id;
                                    assert!(matches!(
                                        expected_read_error,
                                        ReaderResult::RoutingIdMismatch {topic, routing_id}
                                            if topic == actual_topic
                                                && routing_id == actual_routing_id
                                    ))
                                }
                                ReaderResult::TooShort(x) => assert!(matches!(
                                    expected_read_error,
                                    ReaderResult::TooShort(y) if x == y
                                )),
                                ReaderResult::Blacklisted(x) => assert!(matches!(
                                    expected_read_error,
                                    ReaderResult::Blacklisted(y) if x == y
                                )),
                            }
                        } else {
                            panic!(
                                "Expected: {:?}, actual: {:?}",
                                expected_forward_result,
                                ForwardResult::ReadError(reader_result)
                            );
                        }
                    }
                }
            }
            Err(expected_error) => {
                if let Err(actual_error) = actual_result {
                    assert_eq!(
                        expected_error.root_cause().to_string(),
                        actual_error.root_cause().to_string()
                    )
                } else {
                    panic!(
                        "Expected error {:?} but got ok {:?}",
                        expected_error,
                        actual_result.unwrap()
                    );
                }
            }
        }
    }
}
