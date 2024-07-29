use std::sync::{Arc, OnceLock};
use std::time::Duration;

use anyhow::{bail, Result};
use savant_core::transport::zeromq::{NonBlockingReader, ReaderResult};
use tokio::sync::{mpsc, Mutex};
use tokio::task::yield_now;

use media_gateway_common::model::Media;
use media_gateway_common::statistics::StatisticsService;

use crate::client::{ForwardResult, GatewayClient};
use crate::configuration::GatewayClientConfiguration;
use crate::wait::WaitStrategy;

const STAT_STAGE_NAME: &str = "client-relay";

pub struct GatewayClientService {
    channel_size: usize,
    client: Arc<GatewayClient>,
    reader: Arc<Mutex<NonBlockingReader>>,
    wait_strategy: WaitStrategy,
    statistics_service: Arc<Option<StatisticsService>>,
    started: Arc<OnceLock<()>>,
    stopped: Arc<OnceLock<()>>,
}

impl GatewayClientService {
    pub fn new(
        client: GatewayClient,
        reader: NonBlockingReader,
        wait_strategy: WaitStrategy,
        channel_size: usize,
        statistics_service: Option<StatisticsService>,
    ) -> Self {
        Self {
            channel_size,
            client: Arc::new(client),
            reader: Arc::new(Mutex::new(reader)),
            wait_strategy,
            statistics_service: Arc::new(statistics_service),
            started: Arc::new(OnceLock::new()),
            stopped: Arc::new(OnceLock::new()),
        }
    }

    pub async fn run(&self) -> Result<()> {
        let started_result = self.started.set(());
        if started_result.is_err() {
            bail!("Service has already been started.")
        }
        log::info!("Service is being started");

        let (sender, mut receiver) = mpsc::channel(self.channel_size);

        let reader_lock = self.reader.clone();
        let reader_stopped = self.stopped.clone();
        let reader_wait_strategy = self.wait_strategy.clone();
        let reader_statistics_service = self.statistics_service.clone();

        let reader_task = tokio::spawn(async move {
            log::info!("Message reading is started");
            loop {
                if reader_stopped.get().is_some() {
                    log::info!("Message reading is being stopped");
                    break;
                }
                let reader = reader_lock.lock().await;
                let receive_result = reader.try_receive();
                if receive_result.is_none() {
                    log::trace!("No message received, yielding");
                    reader_wait_strategy.wait().await;
                    continue;
                }
                match receive_result.unwrap() {
                    Ok(reader_result) => match reader_result {
                        ReaderResult::Message {
                            message,
                            topic,
                            data,
                            ..
                        } => {
                            log::debug!("Success while reading message");
                            let id = match reader_statistics_service.as_ref() {
                                Some(service) => match service.register_message_start() {
                                    Ok(id) => Some(id),
                                    Err(e) => {
                                        log::warn!(
                                            "Error while starting message statistics: {:?}",
                                            e
                                        );
                                        None
                                    }
                                },
                                None => None,
                            };
                            let media = Media {
                                message: Option::from(savant_protobuf::generated::Message::from(
                                    message.as_ref(),
                                )),
                                topic,
                                data,
                            };
                            if let Err(e) = sender.send((id, media)).await {
                                log::warn!("Error while sharing message: {:?}", e);
                                break;
                            }
                        }
                        ReaderResult::Timeout => {
                            log::debug!(
                                "Timeout while receiving message, waiting for the next message"
                            );
                        }
                        _ => {
                            log::warn!("Unexpected reader result: {:?}", reader_result)
                        }
                    },
                    Err(e) => bail!("Error while receiving message: {:?}", e),
                };
            }
            let shutdown_result = reader_lock.lock().await.shutdown();
            if let Some(e) = shutdown_result.err() {
                log::warn!("Error while shutting down reader: {:?}", e);
            }
            log::info!("Message reading is stopped");
            Ok(())
        });

        let client = self.client.clone();
        let sender_statistics_service = self.statistics_service.clone();

        let sender_task: tokio::task::JoinHandle<Result<()>> = tokio::spawn(async move {
            log::info!("Message sending is started");
            while let Some((id, media)) = receiver.recv().await {
                let mut retry: u32 = 0;
                loop {
                    let forward_result = client.forward_message(&media).await;
                    match forward_result {
                        Ok(ForwardResult::Success) => {
                            if let Some(stat_id) = id {
                                if let Err(e) = sender_statistics_service
                                    .as_ref()
                                    .as_ref()
                                    .unwrap()
                                    .register_message_end(stat_id)
                                {
                                    log::warn!("Error while ending message statistics: {:?}", e)
                                }
                            }
                            log::debug!("Success while sending message (retry={})", retry);
                            if retry > 0 {
                                log::info!("Success while sending message on {} retry", retry);
                            }
                            break;
                        }
                        Ok(result) => {
                            log::warn!(
                                "Failure while sending message (retry={}): {:?}",
                                retry,
                                result
                            );
                        }
                        Err(e) => {
                            log::warn!("Error while sending message (retry={}): {:?}", retry, e)
                        }
                    }
                    let next_retry = retry.checked_add(1);
                    retry = if let Some(n) = next_retry {
                        n
                    } else {
                        log::warn!("Retry overflow while sending message, resetting");
                        0
                    };
                    yield_now().await
                }
            }
            log::info!("Message sending is being stopped");
            log::info!("Message sending is stopped");
            Ok(())
        });
        let _ = reader_task.await.expect("Error in message reading task");
        let _ = sender_task.await.expect("Error in message sending task");
        log::info!("Service is stopped");
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        log::info!("Service is being stopped");
        let stopped_result = self.stopped.set(());
        if stopped_result.is_err() {
            bail!("Service has already been stopped")
        }
        Ok(())
    }
}

impl TryFrom<&GatewayClientConfiguration> for GatewayClientService {
    type Error = anyhow::Error;

    fn try_from(
        configuration: &GatewayClientConfiguration,
    ) -> std::result::Result<Self, Self::Error> {
        let reader = NonBlockingReader::try_from(&configuration.in_stream)?;
        let client = GatewayClient::try_from(configuration)?;
        let statistics_service = if let Some(statistics_conf) = &configuration.statistics {
            Some(StatisticsService::try_from((
                statistics_conf,
                STAT_STAGE_NAME,
            ))?)
        } else {
            None
        };
        let wait_strategy = match &configuration.wait_strategy {
            Some(strategy) => strategy.clone(),
            None => WaitStrategy::Sleep(Duration::from_millis(1)),
        };
        Ok(GatewayClientService::new(
            client,
            reader,
            wait_strategy,
            configuration.in_stream.inflight_ops,
            statistics_service,
        ))
    }
}
