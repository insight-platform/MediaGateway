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
//! See [configuration files](https://github.com/insight-platform/MediaGateway/blob/main/samples/configuration/client).
//! `in_stream` fields represents configuration for
//! [`ReaderConfigBuilder`](savant_core::transport::zeromq::ReaderConfigBuilder).
use std::env::args;
use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use anyhow::{anyhow, Result};
use log::info;
use tokio::signal::{ctrl_c, unix};

use media_gateway_common::api::health;
use media_gateway_common::health::HealthService;

use crate::configuration::GatewayClientConfiguration;
use crate::service::GatewayClientService;

mod client;
pub mod configuration;
mod service;
mod wait;

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
    let bind_address = (conf.ip.as_str(), conf.port);

    let health_service = web::Data::new(HealthService::new());
    let service = Arc::new(GatewayClientService::try_from(&conf)?);
    let service_to_stop = service.clone();

    tokio::spawn(async move {
        let mut interrupt_signal = unix::signal(unix::SignalKind::interrupt()).unwrap();
        let mut shutdown_signal = unix::signal(unix::SignalKind::terminate()).unwrap();
        let mut quit_signal = unix::signal(unix::SignalKind::quit()).unwrap();
        tokio::select! {
            _ = ctrl_c() => {},
            _ = interrupt_signal.recv() => {}
            _ = shutdown_signal.recv() => {}
            _ = quit_signal.recv() => {}
        }
        service_to_stop
            .stop()
            .expect("Error while stopping the service");
    });

    tokio::spawn(async move { service.run().await });

    HttpServer::new(move || {
        App::new()
            .app_data(health_service.clone())
            .route("/health", web::get().to(health))
    })
    .bind(bind_address)?
    .run()
    .await
    .map_err(anyhow::Error::from)
}
