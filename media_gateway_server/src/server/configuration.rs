use std::time::Duration;

use media_gateway_common::configuration::BasicUser;
use savant_core::transport::zeromq::{SyncWriter, WriterConfigBuilder};
use serde::{Deserialize, Serialize};
use twelf::{config, Layer};

#[config]
#[derive(Debug, Serialize)]
pub struct GatewayConfiguration {
    pub(crate) ip: String,
    pub(crate) port: u16,
    pub(crate) ssl: Option<SslConfiguration>,
    pub(crate) out_stream: SinkConfiguration,
    pub(crate) auth: Option<AuthConfiguration>,
}

impl GatewayConfiguration {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let conf = Self::with_layers(&[Layer::Json(path.into())])?;
        Ok(conf)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SslConfiguration {
    pub(crate) certificate: String,
    pub(crate) certificate_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfiguration {
    pub(crate) basic: Vec<BasicUser>,
}

impl TryFrom<&SinkConfiguration> for SyncWriter {
    type Error = anyhow::Error;

    fn try_from(configuration: &SinkConfiguration) -> Result<Self, Self::Error> {
        let conf = WriterConfigBuilder::default()
            .url(&configuration.url)?
            .with_receive_timeout(configuration.receive_timeout.as_millis() as i32)?
            .with_send_timeout(configuration.send_timeout.as_millis() as i32)?
            .with_receive_retries(configuration.receive_retries as i32)?
            .with_send_retries(configuration.send_retries as i32)?
            .with_receive_hwm(configuration.receive_hwm as i32)?
            .with_send_hwm(configuration.send_hwm as i32)?
            .build()?;

        let w = SyncWriter::new(&conf)?;
        w.is_started();
        Ok(w)
    }
}

// copy-paste from Replay
#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Clone)]
pub struct SinkConfiguration {
    pub(crate) url: String,
    pub(crate) send_timeout: Duration,
    pub(crate) send_retries: usize,
    pub(crate) receive_timeout: Duration,
    pub(crate) receive_retries: usize,
    pub(crate) send_hwm: usize,
    pub(crate) receive_hwm: usize,
    pub(crate) inflight_ops: usize,
}

impl Default for SinkConfiguration {
    fn default() -> Self {
        Self {
            url: String::from("dealer+connect:ipc:///tmp/in"),
            send_timeout: Duration::from_secs(1),
            send_retries: 3,
            receive_timeout: Duration::from_secs(1),
            receive_retries: 3,
            send_hwm: 1000,
            receive_hwm: 1000,
            inflight_ops: 100,
        }
    }
}

#[allow(clippy::too_many_arguments)]
impl SinkConfiguration {
    pub fn new(
        url: &str,
        send_timeout: Duration,
        send_retries: usize,
        receive_timeout: Duration,
        receive_retries: usize,
        send_hwm: usize,
        receive_hwm: usize,
        inflight_ops: usize,
    ) -> Self {
        Self {
            url: url.to_string(),
            send_timeout,
            send_retries,
            receive_timeout,
            receive_retries,
            send_hwm,
            receive_hwm,
            inflight_ops,
        }
    }

    #[cfg(test)]
    pub fn test_dealer_connect_sink() -> Self {
        Self::new(
            "dealer+connect:ipc:///tmp/in",
            Duration::from_secs(1),
            3,
            Duration::from_secs(1),
            3,
            1000,
            100,
            100,
        )
    }
}
// copy-paste from Replay
