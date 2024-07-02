//! The media gateway client.
//!
//! The module provides [`GatewayClient`] and [`ForwardResult`].
use std::fs;

use anyhow::anyhow;
use http_auth_basic::Credentials;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Certificate, Client, Identity, StatusCode};

use media_gateway_common::model::Media;

use crate::configuration::GatewayClientConfiguration;

/// The result of [`GatewayClient::forward_message`] method.
#[derive(Debug)]
pub enum ForwardResult {
    /// Represents success
    Success,
    /// Represents the error caused by
    /// [`WriterResult::SendTimeout`](savant_core::transport::zeromq::WriterResult::SendTimeout)
    SendTimeout,
    /// Represents the error caused by
    /// [`WriterResult::AckTimeout`](savant_core::transport::zeromq::WriterResult::AckTimeout)
    AckTimeout,
}

/// The client for the media gateway server.
///
/// The recommended way to create a new instance is via [`GatewayClientConfiguration`].
/// ```rust no_run
/// # use anyhow::Result;
/// # use media_gateway_client::client::GatewayClient;
/// # use media_gateway_client::configuration::GatewayClientConfiguration;
/// # fn main() -> Result<()> {
/// let conf = GatewayClientConfiguration::new("client.json")?;
/// let client = GatewayClient::try_from(&conf)?;
/// # Ok(())
/// # }
/// ```
///
/// The main method is [`GatewayClient::forward_message`]. After usage of the client
/// [`GatewayClient::shutdown`] method should be called to release resources.
pub struct GatewayClient {
    url: String,
    client: Client,
}

impl GatewayClient {
    /// Constructs a new instance of the client. The **recommended** way is to construct via
    /// [`GatewayClientConfiguration`].
    ///
    /// # Arguments
    /// * `reader` - a reader for messages to be forwarded to the media gateway server
    /// * `client` - a client to forward messages
    /// * `url` - an endpoint of the media gateway service to accept messages
    ///
    /// # Details
    ///
    /// Before calling this method the reader must be started and the client must be fully
    /// configured (SSL certificates, [`AUTHORIZATION`] header as a default headers).
    pub fn new(client: Client, url: String) -> Self {
        Self { client, url }
    }

    /// Receives the messages using [`SyncReader`] and sends it to the media gateway server.
    pub async fn forward_message(&self, media: &Media) -> anyhow::Result<ForwardResult> {
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
}

impl TryFrom<&GatewayClientConfiguration> for GatewayClient {
    type Error = anyhow::Error;

    fn try_from(configuration: &GatewayClientConfiguration) -> Result<Self, Self::Error> {
        let mut client_builder = Client::builder().tls_built_in_root_certs(true);

        client_builder = if let Some(ssl_conf) = &configuration.ssl {
            client_builder = if let Some(server_ssl_conf) = &ssl_conf.server {
                let buf = fs::read(&server_ssl_conf.certificate)?;
                let cert = Certificate::from_pem(&buf)?;

                client_builder.add_root_certificate(cert)
            } else {
                client_builder
            };

            if let Some(client_ssl_conf) = &ssl_conf.client {
                let cert = fs::read(&client_ssl_conf.certificate)?;
                let key = fs::read(&client_ssl_conf.certificate_key)?;
                let identity = Identity::from_pkcs8_pem(&cert, &key)?;

                client_builder.identity(identity)
            } else {
                client_builder
            }
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

        let client = GatewayClient::new(Client::default(), gateway_url);

        let actual_result = client.forward_message(&media).await;

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
