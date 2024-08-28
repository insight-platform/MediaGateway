use std::sync::{Arc, OnceLock};
use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use opentelemetry::trace::{FutureExt, Status, TraceContextExt};
use opentelemetry::KeyValue;
use savant_core::transport::zeromq::{NonBlockingReader, ReaderResult};
use tokio::sync::{mpsc, Mutex};
use tokio_timerfd::sleep;

use media_gateway_common::model::Media;
use media_gateway_common::statistics::StatisticsService;
use media_gateway_common::telemetry::{get_context_with_span, get_message_context};

use crate::client::{ForwardResult, GatewayClient};
use crate::configuration::GatewayClientConfiguration;
use crate::retry::{Retry, RetryStrategy};
use crate::wait::WaitStrategy;

const STAT_STAGE_NAME: &str = "client-relay";

pub struct GatewayClientService {
    channel_size: usize,
    client: Arc<GatewayClient>,
    reader: Arc<Mutex<NonBlockingReader>>,
    wait_strategy: WaitStrategy,
    retry_strategy: RetryStrategy,
    statistics_service: Arc<Option<StatisticsService>>,
    started: Arc<OnceLock<()>>,
    stopped: Arc<OnceLock<()>>,
}

impl GatewayClientService {
    pub fn new(
        client: GatewayClient,
        reader: NonBlockingReader,
        wait_strategy: WaitStrategy,
        retry_strategy: RetryStrategy,
        channel_size: usize,
        statistics_service: Option<StatisticsService>,
    ) -> Self {
        Self {
            channel_size,
            client: Arc::new(client),
            reader: Arc::new(Mutex::new(reader)),
            wait_strategy,
            retry_strategy,
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
                            let parent_ctx = get_message_context(&message);
                            let ctx = get_context_with_span("process", &parent_ctx);
                            let queue_ctx = get_context_with_span("queue", &ctx);
                            let queue_span = queue_ctx.span();

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
                            if let Err(e) = sender.send((id, media, ctx)).await {
                                log::warn!("Error while sharing message: {:?}", e);
                                if queue_span.is_recording() {
                                    queue_span.record_error(&e);
                                    queue_span
                                        .set_status(Status::error("error while sharing message"));
                                }
                                queue_span.end();
                                break;
                            }
                            if queue_span.is_recording() {
                                queue_span.set_status(Status::Ok);
                            }
                            queue_span.end();
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
        let sender_retry_strategy = self.retry_strategy.clone();

        let sender_task: tokio::task::JoinHandle<Result<()>> = tokio::spawn(async move {
            log::info!("Message sending is started");
            while let Some((id, mut media, ctx)) = receiver.recv().await {
                let ctx = get_context_with_span("forward", &ctx);
                let span = ctx.span();

                let mut retry: Option<Retry> = None;
                loop {
                    let retry_number = retry.as_ref().map_or(0, |e| e.number());
                    let forward_result = client
                        .forward_message(&mut media)
                        .with_context(ctx.clone())
                        .await;
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
                            if retry.is_some() {
                                log::info!(
                                    "Success while sending message on {} retry",
                                    retry.unwrap().number()
                                );
                            } else {
                                log::debug!("Success while sending message (retry=0)");
                            }
                            if span.is_recording() {
                                span.add_event(
                                    "attempt",
                                    vec![
                                        KeyValue::new("number", retry_number as i64),
                                        KeyValue::new("result", ForwardResult::Success.to_string()),
                                    ],
                                );
                                span.set_status(Status::Ok);
                            }
                            span.end();
                            ctx.span().end();
                            break;
                        }
                        Ok(result) => {
                            log::warn!(
                                "Failure while sending message (retry={}): {:?}",
                                retry_number,
                                result
                            );
                            if span.is_recording() {
                                span.add_event(
                                    "attempt",
                                    vec![
                                        KeyValue::new("number", retry_number as i64),
                                        KeyValue::new("result", result.to_string()),
                                    ],
                                );
                            }
                        }
                        Err(e) => {
                            log::warn!(
                                "Error while sending message (retry={}): {:?}",
                                retry_number,
                                e
                            );
                            if span.is_recording() {
                                span.add_event(
                                    "attempt",
                                    vec![
                                        KeyValue::new("number", retry_number as i64),
                                        KeyValue::new("error", e.to_string()),
                                    ],
                                );
                            }
                        }
                    }
                    let next_retry = sender_retry_strategy.next_retry(retry);
                    let sleep_duration = next_retry.delay();
                    retry = Some(next_retry);
                    log::warn!("Next retry after {} nanoseconds", sleep_duration.as_nanos());
                    sleep(sleep_duration)
                        .await
                        .expect("Error while sleeping between attmpts to send a message")
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
        let retry_strategy = match &configuration.retry_strategy {
            Some(RetryStrategy::Exponential {
                initial_delay,
                maximum_delay,
                multiplier,
            }) => {
                if initial_delay > maximum_delay {
                    return Err(anyhow!("Invalid initial_delay: greater than maximum_delay"));
                }
                if *multiplier < 2 {
                    return Err(anyhow!("Invalid multiplier: less than 2"));
                }
                RetryStrategy::Exponential {
                    initial_delay: *initial_delay,
                    maximum_delay: *maximum_delay,
                    multiplier: *multiplier,
                }
            }
            None => RetryStrategy::Exponential {
                initial_delay: Duration::from_millis(1),
                maximum_delay: Duration::from_secs(1),
                multiplier: 2,
            },
        };
        Ok(GatewayClientService::new(
            client,
            reader,
            wait_strategy,
            retry_strategy,
            configuration.in_stream.inflight_ops,
            statistics_service,
        ))
    }
}
