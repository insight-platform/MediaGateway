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
use std::fs;
use std::io::BufReader;
use std::sync::Arc;

use actix_web::middleware::Condition;
use actix_web::web::scope;
use actix_web::{web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use anyhow::{anyhow, Result};
use log::info;
use rustls::pki_types::CertificateRevocationListDer;
use rustls::server::{NoClientAuth, WebPkiClientVerifier};
use rustls::RootCertStore;
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
    println!("                 In-Sight Media Gateway                 ");
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
        let client_verifier = if let Some(client_ssl_conf) = &ssl_conf.client {
            let mut cert_store = RootCertStore::empty();
            let cert_file = fs::File::open(&client_ssl_conf.certificates).unwrap();
            let mut cert_reader = BufReader::new(cert_file);
            for cert in rustls_pemfile::certs(&mut cert_reader) {
                cert_store.add(cert.unwrap()).unwrap()
            }

            let mut client_verifier_builder = WebPkiClientVerifier::builder(cert_store.into());
            client_verifier_builder = if let Some(clr_files) = &client_ssl_conf.crls {
                let crls = clr_files
                    .iter()
                    .map(|clr_file| CertificateRevocationListDer::from(fs::read(clr_file).unwrap()))
                    .collect::<Vec<_>>();

                client_verifier_builder.with_crls(crls)
            } else {
                client_verifier_builder
            };
            client_verifier_builder.build()?
        } else {
            Arc::new(NoClientAuth)
        };

        let mut certs_file = BufReader::new(fs::File::open(&ssl_conf.server.certificate).unwrap());
        let mut key_file =
            BufReader::new(fs::File::open(&ssl_conf.server.certificate_key).unwrap());
        let tls_certs = rustls_pemfile::certs(&mut certs_file)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file)
            .next()
            .unwrap()
            .unwrap();

        let tls_config = rustls::ServerConfig::builder()
            .with_client_cert_verifier(client_verifier)
            .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
            .unwrap();
        http_server
            .bind_rustls_0_22(bind_address, tls_config)
            .unwrap()
    } else {
        http_server.bind(bind_address).unwrap()
    };

    http_server.run().await.map_err(anyhow::Error::from)
}
