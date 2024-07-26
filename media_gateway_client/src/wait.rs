use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::task::yield_now;
use tokio_timerfd::sleep;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WaitStrategy {
    #[serde(rename = "yield")]
    Yield,
    #[serde(rename = "sleep")]
    Sleep(Duration),
}

impl WaitStrategy {
    pub async fn wait(&self) {
        match self {
            WaitStrategy::Yield => yield_now().await,
            WaitStrategy::Sleep(duration) => {
                sleep(duration.clone()).await.expect("Error while sleeping")
            }
        }
    }
}
