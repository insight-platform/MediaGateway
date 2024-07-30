//! The media gateway client configuration.
//!
//! The module provides [`GatewayClientConfiguration`].
use std::time::Duration;

use savant_core::transport::zeromq::{NonBlockingReader, ReaderConfigBuilder};
use serde::{Deserialize, Serialize};
use twelf::{config, Layer};

use media_gateway_common::configuration::{BasicUser, StatisticsConfiguration};

use crate::wait::WaitStrategy;

/// SSL settings to connect to the media gateway server.
#[derive(Debug, Serialize, Deserialize)]
pub struct SslConfiguration {
    /// Server settings
    pub server: Option<ServerSslConfiguration>,
    /// Client settings
    pub client: Option<ClientSslConfiguration>,
}

/// Server SSL settings.
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerSslConfiguration {
    /// A path to a self-signed PEM encoded server certificate or PEM encoded CA certificate
    pub certificate: String,
}

/// Client SSL settings. See [`Identity::from_pkcs8_pem`](reqwest::Identity::from_pkcs8_pem).
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientSslConfiguration {
    /// A path to a chain of PEM encoded X509 certificates, with the leaf certificate first
    pub certificate: String,
    /// A path to a PEM encoded PKCS #8 formatted private key
    pub certificate_key: String,
}

/// Authentication settings to connect to the media gateway server.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfiguration {
    /// Credentials for basic authentication.
    pub basic: BasicUser,
}

/// A configuration for [`GatewayClient`](crate::client::GatewayClient).
#[config]
#[derive(Debug, Serialize)]
pub struct GatewayClientConfiguration {
    /// A string representation of an IP address or a host name to bind to
    pub ip: String,
    /// A port to bind to
    pub port: u16,
    /// An endpoint of the media gateway service to accept messages
    pub url: String,
    /// Reader configuration
    pub in_stream: SourceConfiguration,
    /// A strategy how to wait for data while reading
    pub wait_strategy: Option<WaitStrategy>,
    /// SSL settings
    pub ssl: Option<SslConfiguration>,
    /// Authentication settings
    pub auth: Option<AuthConfiguration>,
    /// Statistics settings
    pub statistics: Option<StatisticsConfiguration>,
}

impl GatewayClientConfiguration {
    /// Reads a configuration from JSON file.
    ///
    /// # Arguments
    /// * `path` - a path to the JSON file
    ///
    /// # Examples
    /// See [config.json](https://github.com/insight-platform/MediaGateway/blob/main/samples/configuration/client/default_config.json).
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let conf = Self::with_layers(&[Layer::Json(path.into())])?;
        Ok(conf)
    }
}

// copy-paste from Replay (except documentation) and visibility of SourceConfiguration fields
/// A representation for [`savant_core::transport::zeromq::TopicPrefixSpec`].
#[derive(Debug, Serialize, Deserialize)]
pub enum TopicPrefixSpec {
    /// Represents [`savant_core::transport::zeromq::TopicPrefixSpec::SourceId`].
    #[serde(rename = "source_id")]
    SourceId(String),
    /// Represents [`savant_core::transport::zeromq::TopicPrefixSpec::Prefix`].
    #[serde(rename = "prefix")]
    Prefix(String),
    /// Represents [`savant_core::transport::zeromq::TopicPrefixSpec::None`].
    #[serde(rename = "none")]
    None,
}

/// A configuration for [`SyncReader`](savant_core::transport::zeromq::Reader).
#[derive(Debug, Serialize, Deserialize)]
pub struct SourceConfiguration {
    /// ZeroMQ socket address in the form `<type>+<bind>:<source>` where
    ///
    /// * `<type>` is one of `sub`, `rep`, `router`,
    /// * `<bind>` is one of `bind`, `connect`,
    /// * `<source>` is one of `ipc://<path>`, `tcp://<address>`.
    ///
    /// # Examples
    ///
    /// * `sub+connect:ipc://tmp/test`
    /// * `rep+bind:tcp://127.0.0.1:2345`
    pub url: String,
    /// A timeout to receive a message
    pub receive_timeout: Duration,
    /// A high-water mark to receive messages
    pub receive_hwm: usize,
    /// A topic
    pub topic_prefix_spec: TopicPrefixSpec,
    /// A size of a routing cache
    pub source_cache_size: usize,
    /// Permissions for the IPC endpoint. See [`std::fs::set_permissions`].
    pub fix_ipc_permissions: Option<u32>,
    /// The maximum number of read messages
    pub inflight_ops: usize,
}

impl TryFrom<&SourceConfiguration> for NonBlockingReader {
    type Error = anyhow::Error;

    fn try_from(source_conf: &SourceConfiguration) -> Result<NonBlockingReader, Self::Error> {
        let conf = ReaderConfigBuilder::default().url(&source_conf.url)?;
        let conf = if let Some(fix_ipc_permissions) = source_conf.fix_ipc_permissions {
            conf.with_fix_ipc_permissions(Some(fix_ipc_permissions))?
        } else {
            ReaderConfigBuilder::default().url(&source_conf.url)?
        };
        let conf = conf
            .with_receive_timeout(source_conf.receive_timeout.as_millis() as i32)?
            .with_receive_hwm(source_conf.receive_hwm as i32)?
            .with_topic_prefix_spec((&source_conf.topic_prefix_spec).into())?
            .with_routing_cache_size(source_conf.source_cache_size)?
            .build()?;

        let mut reader = NonBlockingReader::new(&conf, source_conf.inflight_ops)?;
        reader.start()?;
        Ok(reader)
    }
}
impl From<&TopicPrefixSpec> for savant_core::transport::zeromq::TopicPrefixSpec {
    fn from(value: &TopicPrefixSpec) -> Self {
        match value {
            TopicPrefixSpec::SourceId(value) => Self::SourceId(value.clone()),
            TopicPrefixSpec::Prefix(value) => Self::Prefix(value.clone()),
            TopicPrefixSpec::None => Self::None,
        }
    }
}
// copy-paste from Replay (except documentation) and visibility of SourceConfiguration fields
