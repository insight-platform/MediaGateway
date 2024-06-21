use std::time::Duration;

use savant_core::transport::zeromq::{ReaderConfigBuilder, SyncReader};
use serde::{Deserialize, Serialize};
use twelf::{config, Layer};

#[derive(Debug, Serialize, Deserialize)]
pub struct SslConfiguration {
    pub certificate: String,
}

#[config]
#[derive(Debug, Serialize)]
pub struct GatewayClientConfiguration {
    pub url: String,
    pub in_stream: SourceConfiguration,
    pub ssl: Option<SslConfiguration>,
}

impl GatewayClientConfiguration {
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

// copy-paste from Replay
#[derive(Debug, Serialize, Deserialize)]
pub enum TopicPrefixSpec {
    #[serde(rename = "source_id")]
    SourceId(String),
    #[serde(rename = "prefix")]
    Prefix(String),
    #[serde(rename = "none")]
    None,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceConfiguration {
    pub(crate) url: String,
    pub(crate) receive_timeout: Duration,
    pub(crate) receive_hwm: usize,
    pub(crate) topic_prefix_spec: TopicPrefixSpec,
    pub(crate) source_cache_size: usize,
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
// copy-paste from Replay
