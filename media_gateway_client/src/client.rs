use anyhow::anyhow;
use reqwest::{Client, StatusCode};
use reqwest::header::CONTENT_TYPE;
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
        Self { reader, client, url }
    }

    pub async fn forward_message(&self) -> anyhow::Result<ForwardResult> {
        let reader_result = self.reader.receive()?;
        match reader_result {
            ReaderResult::Message { message, topic, data, .. } => {
                let media = Media {
                    message,
                    topic,
                    data,
                };
                let data = serde_json::to_string(&media)?;
                let send_result = self.client.post(&self.url)
                    .body(data)
                    .header(CONTENT_TYPE, "application/json")
                    .send()
                    .await;
                match send_result {
                    Ok(response) => match response.status() {
                        StatusCode::OK => Ok(ForwardResult::Success),
                        StatusCode::GATEWAY_TIMEOUT => Ok(ForwardResult::SendTimeout),
                        StatusCode::BAD_GATEWAY => Ok(ForwardResult::AckTimeout),
                        status_code => Err(anyhow!("Invalid HTTP status: {}", status_code))
                    }
                    Err(e) => Err(anyhow!("Error while sending a message").context(e.to_string()))
                }
            }
            _ => Ok(ReadError(reader_result))
        }
    }
}

impl TryFrom<&GatewayClientConfiguration> for GatewayClient {
    type Error = anyhow::Error;

    fn try_from(configuration: &GatewayClientConfiguration) -> anyhow::Result<Self> {
        let reader = SyncReader::try_from(&configuration.in_stream)?;
        let client = Client::default();
        Ok(GatewayClient::new(reader, client, configuration.url.clone()))
    }
}
