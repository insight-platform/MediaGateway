//! The media gateway server.
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
//! # Examples
//! See [configuration files](https://github.com/insight-platform/MediaGateway/blob/main/samples/server).
use std::collections::HashMap;
use std::env::args;

use actix_web::middleware::Condition;
use actix_web::web::scope;
use actix_web::{web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use anyhow::{anyhow, Result};
use log::info;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
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
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
        builder.set_private_key_file(ssl_conf.certificate_key, SslFiletype::PEM)?;
        builder.set_certificate_chain_file(ssl_conf.certificate)?;

        http_server.bind_openssl(bind_address, builder).unwrap()
    } else {
        http_server.bind(bind_address).unwrap()
    };

    http_server.run().await.map_err(anyhow::Error::from)
}
