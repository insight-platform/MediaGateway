//! A client application for [`media_gateway_server`](https://github.com/insight-platform/MediaGateway).
//!
//! The application reads messages from [ZeroMQ](https://zeromq.org/) using
//! [`NonBlockingReader`](savant_core::transport::zeromq::NonBlockingReader) from `savant_core`
//! crate and sends them to the server using [`Client`](reqwest::Client) from `reqwest` crate
//! concurrently.

//! To run the client
//! ```bash
//! media_gateway_client config.json
//! ```
//!
//! Following features are supported:
//! * SSL (including a self-signed PEM encoded certificate)
//! * client certificate authentication
//! * basic authentication
//!
//! # Examples
//! See [configuration files](https://github.com/insight-platform/MediaGateway/blob/main/samples/client).
//! `in_stream` fields represents configuration for
//! [`ReaderConfigBuilder`](savant_core::transport::zeromq::ReaderConfigBuilder).
use std::env::args;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use log::info;

use crate::configuration::GatewayClientConfiguration;
use crate::service::GatewayClientService;

mod client;
pub mod configuration;
mod service;

#[tokio::main]
async fn main() -> Result<()> {
    println!("--------------------------------------------------------");
    println!("              In-Sight Media Gateway Client             ");
    println!("GitHub: https://github.com/insight-platform/MediaGateway");
    println!("This program is licensed under the BSL-1.1 license      ");
    println!("      For more information, see the LICENSE file        ");
    println!("           (c) 2024 BwSoft Management, LLC              ");
    println!("--------------------------------------------------------");

    env_logger::init();
    let conf_arg = args()
        .nth(1)
        .ok_or_else(|| anyhow!("missing configuration argument"))?;
    info!("Configuration: {}", conf_arg);

    let conf = GatewayClientConfiguration::new(&conf_arg)?;

    let service = Arc::new(GatewayClientService::try_from(&conf)?);
    let service_to_stop = service.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        service_to_stop
            .stop()
            .expect("Error while stopping the service");
    });

    service.run().await
}
