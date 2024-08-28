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
//! * TLS (including a self-signed PEM encoded certificate)
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
//! See [configuration files](https://github.com/insight-platform/MediaGateway/blob/main/configuration/samples/server).
//! `out_stream` fields represents configuration for
//! [`WriterConfigBuilder`](savant_core::transport::zeromq::WriterConfigBuilder).
use std::env::args;
use std::num::NonZeroUsize;
use std::sync::Arc;

use actix_web::middleware::Condition;
use actix_web::web::scope;
use actix_web::{web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use anyhow::{anyhow, Result};
use log::info;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod, SslVerifyMode};
use openssl::x509::store::{X509Lookup, X509StoreBuilder};
use openssl::x509::verify::X509VerifyFlags;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use media_gateway_common::api::health;
use media_gateway_common::configuration::Credentials;
use media_gateway_common::health::HealthService;
use media_gateway_common::telemetry;
use server::configuration::GatewayConfiguration;

use crate::server::api::gateway;
use crate::server::security::quarantine::{
    AuthQuarantine, AuthQuarantineFactory, NoOpAuthQuarantine,
};
use crate::server::security::{basic_auth_validator, BasicAuthCheckResult};
use crate::server::service::cache::{Cache, CacheUsageFactory, NoOpCacheUsageTracker};
use crate::server::service::crypto::argon2::Argon2PasswordService;
use crate::server::service::crypto::PasswordService;
use crate::server::service::gateway::GatewayService;
use crate::server::service::user::{UserData, UserService};
use crate::server::storage::etcd::EtcdStorage;
use crate::server::storage::{EmptyStorage, Storage};

mod server;

type AuthAppData = (
    Box<dyn Storage<UserData> + Sync + Send>,
    Cache<Credentials, BasicAuthCheckResult>,
    Box<dyn AuthQuarantine + Sync + Send>,
);

fn main() -> Result<()> {
    println!("--------------------------------------------------------");
    println!("             In-Sight Media Gateway Server              ");
    println!("GitHub: https://github.com/insight-platform/MediaGateway");
    println!("This program is licensed under the BSL-1.1 license      ");
    println!("      For more information, see the LICENSE file        ");
    println!("           (c) 2024 BwSoft Management, LLC              ");
    println!("--------------------------------------------------------");

    let runtime = Runtime::new().unwrap();

    env_logger::init();
    let conf_arg = args()
        .nth(1)
        .ok_or_else(|| anyhow!("missing configuration argument"))?;
    info!("Configuration: {}", conf_arg);

    let conf = GatewayConfiguration::new(&conf_arg)?;

    if let Some(telemetry_conf) = conf.telemetry.as_ref() {
        runtime.block_on(async {
            telemetry::init(telemetry_conf);
        });
    }

    let bind_address = (conf.ip.as_str(), conf.port);
    let gateway_service = web::Data::new(Mutex::new(GatewayService::try_from(&conf)?));
    let health_service = web::Data::new(HealthService::new());
    let auth_enabled = conf.auth.is_some();
    let (user_storage, auth_cache, auth_quarantine): AuthAppData =
        if let Some(auth_conf) = conf.auth {
            let auth_check_result_cache_usage_tracker = CacheUsageFactory::from(
                auth_conf.basic.cache.usage.as_ref(),
                "auth check result".to_string(),
                &runtime,
            );
            let auth_quarantine = AuthQuarantineFactory::from(&auth_conf.basic, &runtime)?;
            let storage_cache_usage_tracker = CacheUsageFactory::from(
                auth_conf.basic.etcd.cache.usage.as_ref(),
                "user".to_string(),
                &runtime,
            );
            (
                Box::new(
                    EtcdStorage::try_from((
                        &auth_conf.basic.etcd,
                        &runtime,
                        storage_cache_usage_tracker.clone(),
                    ))
                    .unwrap(),
                ),
                Cache::new(
                    auth_conf.basic.cache.size,
                    auth_check_result_cache_usage_tracker,
                ),
                auth_quarantine,
            )
        } else {
            (
                Box::new(EmptyStorage {}),
                Cache::new(
                    NonZeroUsize::new(1).unwrap(),
                    Arc::new(Box::new(NoOpCacheUsageTracker {})),
                ),
                Box::new(NoOpAuthQuarantine {}),
            )
        };
    let basic_auth_cache: web::Data<Cache<Credentials, BasicAuthCheckResult>> =
        web::Data::new(auth_cache);
    let basic_auth_quarantine = web::Data::new(auth_quarantine);
    let password_service: web::Data<Box<dyn PasswordService + Sync + Send>> =
        web::Data::new(Box::new(Argon2PasswordService {}));
    let user_service = web::Data::new(UserService::new(user_storage));

    let mut http_server = HttpServer::new(move || {
        App::new()
            .service(
                scope("/")
                    .app_data(gateway_service.clone())
                    .app_data(user_service.clone())
                    .app_data(password_service.clone())
                    .app_data(basic_auth_cache.clone())
                    .app_data(basic_auth_quarantine.clone())
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

    http_server = if let Some(ssl_conf) = conf.tls {
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
        builder.set_private_key_file(ssl_conf.identity.key, SslFiletype::PEM)?;
        builder.set_certificate_chain_file(ssl_conf.identity.certificate)?;

        builder = if let Some(peer_tls_conf) = &ssl_conf.peers {
            let mut cert_store_builder = X509StoreBuilder::new().unwrap();

            let lookup_method = X509Lookup::hash_dir();
            let lookup = cert_store_builder.add_lookup(lookup_method).unwrap();
            lookup
                .add_dir(
                    peer_tls_conf.lookup_hash_directory.as_str(),
                    SslFiletype::PEM,
                )
                .unwrap();

            let cert_store_builder = if peer_tls_conf.crl_enabled {
                cert_store_builder
                    .set_flags(X509VerifyFlags::from_iter(vec![
                        X509VerifyFlags::CRL_CHECK,
                        X509VerifyFlags::CRL_CHECK_ALL,
                    ]))
                    .unwrap();
                cert_store_builder
            } else {
                cert_store_builder
            };

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

    let result = runtime.block_on(async { http_server.run().await.map_err(anyhow::Error::from) });

    telemetry::shutdown();

    result
}
