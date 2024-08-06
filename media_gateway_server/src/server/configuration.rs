use std::num::NonZeroUsize;
use std::time::Duration;

use savant_core::transport::zeromq::{SyncWriter, WriterConfigBuilder};
use serde::{Deserialize, Serialize};
use twelf::{config, Layer};

use media_gateway_common::configuration::{
    ClientTlsConfiguration, Credentials, Identity, StatisticsConfiguration,
};

#[config]
#[derive(Debug, Serialize)]
pub struct GatewayConfiguration {
    pub(crate) ip: String,
    pub(crate) port: u16,
    pub(crate) tls: Option<ServerTlsConfiguration>,
    pub(crate) out_stream: SinkConfiguration,
    pub(crate) auth: Option<AuthConfiguration>,
    pub(crate) statistics: Option<StatisticsConfiguration>,
}

impl GatewayConfiguration {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let conf = Self::with_layers(&[Layer::Json(path.into())])?;
        Ok(conf)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerTlsConfiguration {
    pub identity: Identity,
    pub peer_lookup_hash_directory: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfiguration {
    pub(crate) basic: BasicAuthConfiguration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BasicAuthConfiguration {
    pub etcd: EtcdConfiguration,
    pub cache: CacheConfiguration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheConfiguration {
    pub size: NonZeroUsize,
    pub usage: Option<CacheUsage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheUsage {
    pub period: Duration,
    pub evicted_threshold: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EtcdConfiguration {
    pub urls: Vec<String>,
    pub tls: Option<ClientTlsConfiguration>,
    pub credentials: Option<Credentials>,
    pub path: String,
    pub data_format: EtcdDataFormat,
    pub lease_timeout: Duration,
    pub connect_timeout: Duration,
    pub cache: CacheConfiguration,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EtcdDataFormat {
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "yaml")]
    Yaml,
}

impl TryFrom<&SinkConfiguration> for SyncWriter {
    type Error = anyhow::Error;

    fn try_from(configuration: &SinkConfiguration) -> Result<Self, Self::Error> {
        let mut builder = WriterConfigBuilder::default()
            .url(&configuration.url)?
            .with_receive_timeout(configuration.receive_timeout.as_millis() as i32)?
            .with_send_timeout(configuration.send_timeout.as_millis() as i32)?
            .with_receive_retries(configuration.receive_retries as i32)?
            .with_send_retries(configuration.send_retries as i32)?
            .with_receive_hwm(configuration.receive_hwm as i32)?
            .with_send_hwm(configuration.send_hwm as i32)?;

        builder = if configuration.fix_ipc_permissions.is_some() {
            builder.with_fix_ipc_permissions(configuration.fix_ipc_permissions)?
        } else {
            builder
        };

        let conf = builder.build()?;
        let w = SyncWriter::new(&conf)?;
        w.is_started();
        Ok(w)
    }
}

// copy-paste from Replay except removal of inflight_ops and addition of fix_ipc_permissions to
// SinkConfiguration
#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Clone)]
pub struct SinkConfiguration {
    pub(crate) url: String,
    pub(crate) send_timeout: Duration,
    pub(crate) send_retries: usize,
    pub(crate) receive_timeout: Duration,
    pub(crate) receive_retries: usize,
    pub(crate) send_hwm: usize,
    pub(crate) receive_hwm: usize,
    pub(crate) fix_ipc_permissions: Option<u32>,
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
            fix_ipc_permissions: None,
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
        fix_ipc_permissions: Option<u32>,
    ) -> Self {
        Self {
            url: url.to_string(),
            send_timeout,
            send_retries,
            receive_timeout,
            receive_retries,
            send_hwm,
            receive_hwm,
            fix_ipc_permissions,
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
            None,
        )
    }
}
// copy-paste from Replay except removal of inflight_ops and addition of fix_ipc_permissions to
// SinkConfiguration
