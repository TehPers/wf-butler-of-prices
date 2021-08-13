use crate::models::Config;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use anyhow::Context;
use std::net::Ipv4Addr;
use tracing::instrument;
use wfinfo_lib::client::DiscordRestClient;

#[instrument]
pub async fn start() -> anyhow::Result<()> {
    // Read config from environment
    let mut config: Config =
        envy::from_env().context("error reading config")?;

    // Shared data
    let discord_client = DiscordRestClient::new(
        config.client_id,
        std::mem::take(&mut config.client_secret),
    )?;
    let port = config.port;

    // Start web server
    // TODO: fix errors returning non-json response
    HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .app_data(Data::new(discord_client.clone()))
            .app_data(Data::new(config.clone()))
            // TODO: remove this at some point so no user data is logged
            // .wrap(LogBody)
            .wrap(logger)
    })
    .bind((Ipv4Addr::UNSPECIFIED, port))?
    .run()
    .await
    .context("error running web server")
}
