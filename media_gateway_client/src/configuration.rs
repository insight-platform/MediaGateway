//! The gateway client configuration.
//!
//! The module provides [`GatewayClientConfiguration`].
use std::time::Duration;

use savant_core::transport::zeromq::{ReaderConfigBuilder, SyncReader};
use serde::{Deserialize, Serialize};
use twelf::{config, Layer};

use media_gateway_common::configuration::BasicUser;

/// SSL settings to connect to the media gateway server.
#[derive(Debug, Serialize, Deserialize)]
pub struct SslConfiguration {
    /// A path to a self-signed PEM encoded server certificate
    pub certificate: String,
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
    /// An endpoint of the media gateway service to accept messages
    pub url: String,
    /// Reader configuration
    pub in_stream: SourceConfiguration,
    /// SSL settings
    pub ssl: Option<SslConfiguration>,
    /// Authentication settings
    pub auth: Option<AuthConfiguration>,
}

impl GatewayClientConfiguration {
    /// Reads a configuration from JSON file.
    ///
    /// # Arguments
    /// * `path` - a path to the JSON file
    ///
    /// # Examples
    /// See [config.json](https://github.com/insight-platform/MediaGateway/blob/develop/samples/client/config.json)
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let conf = Self::with_layers(&[Layer::Json(path.into())])?;
        Ok(conf)
    }
}

impl TryFrom<&SourceConfiguration> for SyncReader {
    type Error = anyhow::Error;

    fn try_from(source_conf: &SourceConfiguration) -> Result<SyncReader, Self::Error> {
        let conf = ReaderConfigBuilder::default()
            .url(&source_conf.url)?
            .with_receive_timeout(source_conf.receive_timeout.as_millis() as i32)?
            .with_receive_hwm(source_conf.receive_hwm as i32)?
            .with_topic_prefix_spec((&source_conf.topic_prefix_spec).into())?
            .with_routing_cache_size(source_conf.source_cache_size)?
            .with_fix_ipc_permissions(source_conf.fix_ipc_permissions)?
            .build()?;
        let reader = SyncReader::new(&conf)?;
        reader.is_started();
        Ok(reader)
    }
}

// copy-paste from Replay (except documentation)
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
    pub(crate) url: String,
    /// A timeout to receive a message
    pub(crate) receive_timeout: Duration,
    /// A high-water mark to receive messages
    pub(crate) receive_hwm: usize,
    /// A topic
    pub(crate) topic_prefix_spec: TopicPrefixSpec,
    /// A size of routing cache
    pub(crate) source_cache_size: usize,
    /// Permissions for IPC endpoint. See [`std::fs::Permissions::from_mode`]
    pub(crate) fix_ipc_permissions: Option<u32>,
    pub(crate) inflight_ops: usize,
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
// copy-paste from Replay (except documentation)
