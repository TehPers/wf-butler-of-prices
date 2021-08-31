use crate::{controllers::commands_service, models::Config};
use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use anyhow::Context;
use std::{net::Ipv4Addr, sync::Arc, time::Duration};
use tracing::instrument;
use wfinfo_discord::DiscordRestClient;
use wfinfo_lib::reqwest::Client;
use wfinfo_wm::WarframeMarketRestClient;

#[instrument]
pub async fn start() -> anyhow::Result<()> {
    // Read config from environment
    let mut config: Config =
        envy::from_env().context("error reading config")?;

    // Shared data
    let raw_client = Client::builder()
        .timeout(Duration::from_secs(30))
        .https_only(true)
        .user_agent(concat!("TEST_BOT/", env!("CARGO_PKG_VERSION")))
        .build()
        .context("error creating reqwest client")?;
    let discord_client = DiscordRestClient::new(
        raw_client.clone(),
        config.client_id,
        Arc::new(std::mem::take(&mut config.client_secret)),
    );
    let wm_client = WarframeMarketRestClient::new(raw_client.clone());
    let port = config.port;

    // Start web server
    HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .app_data(Data::new(discord_client.clone()))
            .app_data(Data::new(wm_client.clone()))
            .app_data(Data::new(raw_client.clone()))
            .app_data(Data::new(config.clone()))
            .service(commands_service())
            .wrap(logger)
    })
    .bind((Ipv4Addr::UNSPECIFIED, port))?
    .run()
    .await
    .context("error running web server")
}
