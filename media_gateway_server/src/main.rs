use std::env::args;
use std::sync::{Arc, Mutex};

use actix_web::{web, App, HttpServer};
use anyhow::{anyhow, Result};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use twelf::reexports::log::info;

use server::configuration::GatewayConfiguration;

use crate::server::api::{gateway, health};
use crate::server::service::gateway::GatewayService;
use crate::server::service::health::HealthService;

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
    let bind_address = ("127.0.0.1", conf.port);
    let gateway_service = GatewayService::try_from(&conf)?;
    let health_service = Arc::new(HealthService::new());
    let mut http_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Mutex::new(gateway_service.clone())))
            .app_data(web::Data::new(health_service.clone()))
            .service(gateway)
            .service(health)
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
