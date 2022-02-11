use crate::{controllers::interactions_service, models::Config};
use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use anyhow::Context;
use std::net::Ipv4Addr;
use tracing::instrument;

#[instrument]
pub async fn start() -> anyhow::Result<()> {
    // Read config from environment
    let config: Config = envy::from_env().context("error reading config")?;

    // Shared data
    let port = config.port;

    // Start web server
    HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .app_data(Data::new(config.clone()))
            .service(interactions_service())
            .wrap(logger)
    })
    .bind((Ipv4Addr::UNSPECIFIED, port))?
    .run()
    .await
    .context("error running web server")
}
