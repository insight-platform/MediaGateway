//! A client for [`media_gateway_server`](https://github.com/insight-platform/MediaGateway).
//!
//! The [`GatewayClient`](client::GatewayClient) reads messages from [ZeroMQ](https://zeromq.org/)
//! using [`SyncReader`](savant_core::transport::zeromq::SyncReader) from `savant_core`
//! crate and sends them to the server using [`Client`](reqwest::Client) from `reqwest` crate.
//! The [`GatewayClient`](client::GatewayClient) can be configured via a JSON file.
//!
//! # Example
//! ```rust no_run
//!use media_gateway_client::client::{ForwardResult, GatewayClient};
//!use media_gateway_client::configuration::GatewayClientConfiguration;
//!
//! #[tokio::main]
//! async fn main() {
//!     let conf = GatewayClientConfiguration::new("client.json")
//!         .expect("invalid gateway client configuration file");
//!     let mut client = GatewayClient::try_from(&conf)
//!         .expect("invalid gateway client configuration");
//!
//!     let result = client.forward_message().await;
//!     match result {
//!         Ok(ForwardResult::Success) => {
//!             // success processing
//!         }
//!         Ok(_) | Err(_) => {
//!             // failure processing
//!         }
//!     }
//!     client.shutdown().expect("gateway client shutdown failure")
//! }
//! ```
pub mod client;
pub mod configuration;
