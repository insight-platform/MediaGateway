use anyhow::bail;
use savant_core::pipeline::{
    Pipeline, PipelineConfiguration, PipelineConfigurationBuilder, PipelineStagePayloadType,
};
use savant_core::primitives::frame::{
    VideoFrameContent, VideoFrameProxy, VideoFrameTranscodingMethod,
};

use crate::configuration::StatisticsConfiguration;

const STAT_SOURCE_ID: &str = "statistics";

pub struct StatisticsService {
    name: String,
    pipeline: Pipeline,
}

impl StatisticsService {
    pub fn new(configuration: PipelineConfiguration, name: &str) -> Self {
        let pipeline = Pipeline::new(
            vec![(
                name.to_string(),
                PipelineStagePayloadType::Frame,
                None,
                None,
            )],
            configuration,
        )
        .expect("invalid pipeline");

        Self {
            name: name.to_string(),
            pipeline,
        }
    }

    pub fn register_message_start(&self) -> anyhow::Result<i64> {
        let video_frame_proxy = VideoFrameProxy::new(
            STAT_SOURCE_ID,
            "",
            0,
            0,
            VideoFrameContent::None,
            VideoFrameTranscodingMethod::Copy,
            &None,
            None,
            (1, 1000000),
            0,
            None,
            None,
        );
        self.pipeline
            .add_frame(self.name.as_str(), video_frame_proxy)
    }

    pub fn register_message_end(&self, id: i64) -> anyhow::Result<()> {
        self.pipeline.delete(id).map(|_e| ())
    }
}

impl TryFrom<(&StatisticsConfiguration, &str)> for StatisticsService {
    type Error = anyhow::Error;

    fn try_from(value: (&StatisticsConfiguration, &str)) -> Result<Self, Self::Error> {
        let configuration = value.0;
        if configuration.frame_period.is_none() && configuration.timestamp_period.is_none() {
            bail!("At least one of frame_period and timestamp_period should be specified")
        }
        let timestamp_period = match configuration.timestamp_period {
            Some(duration) => {
                let duration_ms = i64::try_from(duration.as_millis())?;
                Some(duration_ms)
            }
            None => None,
        };
        let pipeline_configuration = PipelineConfigurationBuilder::default()
            .collection_history(configuration.history_size)
            .timestamp_period(timestamp_period)
            .frame_period(configuration.frame_period)
            .build()?;

        Ok(StatisticsService::new(pipeline_configuration, value.1))
    }
}
