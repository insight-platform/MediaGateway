//! The media gateway server.
//!
//! The server accepts messages via HTTP(s) and writers them to [ZeroMQ](https://zeromq.org/)
//! using [`SyncWriter`](savant_core::transport::zeromq::SyncWriter) from `savant_core`.
//!
//! To run the server
//! ```bash
//! media_gateway_server config.json
//! ```
//!
//! Following features are supported:
//! * SSL (including a self-signed PEM encoded certificate)
//! * client certificate authentication
//! * basic authentication with an in-memory user data storage
//!
//! # API
//! * an endpoint to process messages
//! ```
//! POST / HTTP/1.1
//! Host: <host>
//! Content-Type: application/protobuf
//! Content-Length: <length>
//!
//! <data>
//! ```
//! where data is [`Media`](media_gateway_common::model::Media)
//!
//! Responses:
//!
//!| HTTP status code | Description                                                                                                                                                                |
//!|------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
//!| 200              | Corresponds to [`WriterResult::Ack`](savant_core::transport::zeromq::WriterResult::Ack) or [`WriterResult::Success`](savant_core::transport::zeromq::WriterResult::Success)|
//!| 504              | Corresponds to [`WriterResult::SendTimeout`](savant_core::transport::zeromq::WriterResult::SendTimeout)                                                                    |
//!| 502              | Corresponds to [`WriterResult::AckTimeout`](savant_core::transport::zeromq::WriterResult::AckTimeout)                                                                      |
//!
//! * a health endpoint
//! ```
//! GET /health HTTP/1.1
//! Host: <host>
//! ```
//! If the server is healthy an HTTP response with 200 OK status code and the body below will be
//! returned
//! ```json
//! {"status":"healthy"}
//! ```
//! # Examples
//! See [configuration files](https://github.com/insight-platform/MediaGateway/blob/main/samples/server).
//! `out_stream` fields represents configuration for
//! [`WriterConfigBuilder`](savant_core::transport::zeromq::WriterConfigBuilder).
use std::collections::HashMap;
use std::env::args;

use actix_web::middleware::Condition;
use actix_web::web::scope;
use actix_web::{web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use anyhow::{anyhow, Result};
use log::info;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod, SslVerifyMode};
use openssl::x509::store::{X509Lookup, X509StoreBuilder};
use openssl::x509::verify::X509VerifyFlags;
use tokio::sync::Mutex;

use server::configuration::GatewayConfiguration;

use crate::server::api::{gateway, health};
use crate::server::security::basic_auth_validator;
use crate::server::service::gateway::GatewayService;
use crate::server::service::health::HealthService;
use crate::server::service::user::UserService;

mod server;

#[actix_web::main]
async fn main() -> Result<()> {
    println!("--------------------------------------------------------");
    println!("             In-Sight Media Gateway Server              ");
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

    let conf = GatewayConfiguration::new(&conf_arg)?;
    let bind_address = (conf.ip.as_str(), conf.port);
    let gateway_service = web::Data::new(Mutex::new(GatewayService::try_from(&conf)?));
    let health_service = web::Data::new(HealthService::new());
    let auth_enabled = conf.auth.is_some();
    let users = if let Some(auth_conf) = conf.auth {
        HashMap::from_iter(
            auth_conf
                .basic
                .iter()
                .map(|e| (e.id.clone(), e.password.clone())),
        )
    } else {
        HashMap::new()
    };
    let user_service = web::Data::new(UserService::new(users));
    let mut http_server = HttpServer::new(move || {
        App::new()
            .service(
                scope("/")
                    .app_data(gateway_service.clone())
                    .app_data(user_service.clone())
                    .route("", web::post().to(gateway))
                    .wrap(Condition::new(
                        auth_enabled,
                        HttpAuthentication::basic(basic_auth_validator),
                    )),
            )
            .service(
                scope("/health")
                    .app_data(health_service.clone())
                    .route("", web::get().to(health)),
            )
    });

    http_server = if let Some(ssl_conf) = conf.ssl {
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
        builder.set_private_key_file(ssl_conf.server.certificate_key, SslFiletype::PEM)?;
        builder.set_certificate_chain_file(ssl_conf.server.certificate)?;

        builder = if let Some(client_ssl_conf) = &ssl_conf.client {
            let mut cert_store_builder = X509StoreBuilder::new().unwrap();

            let lookup_method = X509Lookup::hash_dir();
            let lookup = cert_store_builder.add_lookup(lookup_method).unwrap();
            lookup
                .add_dir(&client_ssl_conf.certificate_directory, SslFiletype::PEM)
                .unwrap();

            cert_store_builder
                .set_flags(X509VerifyFlags::from_iter(vec![
                    X509VerifyFlags::CRL_CHECK,
                    X509VerifyFlags::CRL_CHECK_ALL,
                ]))
                .unwrap();

            builder
                .set_verify_cert_store(cert_store_builder.build())
                .unwrap();
            builder.set_verify(SslVerifyMode::from_iter(vec![
                SslVerifyMode::PEER,
                SslVerifyMode::FAIL_IF_NO_PEER_CERT,
            ]));
            builder
        } else {
            builder
        };

        http_server.bind_openssl(bind_address, builder).unwrap()
    } else {
        http_server.bind(bind_address).unwrap()
    };

    http_server.run().await.map_err(anyhow::Error::from)
}
