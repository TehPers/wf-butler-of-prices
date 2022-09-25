#![warn(clippy::all, clippy::pedantic)]
#![forbid(unsafe_code)]

mod routes;
mod models;
mod startup;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use crate::models::Config;
    use anyhow::Context;
    use tracing::{error, Level};
    use tracing_subscriber::FmtSubscriber;

    // Setup tracing
    if cfg!(debug_assertions) {
        FmtSubscriber::builder()
            .compact()
            .with_max_level(Level::DEBUG)
            .try_init()
            .expect("error registering tracing subscriber");
    } else {
        FmtSubscriber::builder()
            .json()
            .with_max_level(Level::INFO)
            .try_init()
            .expect("error registering tracing subscriber");
    };

    // Read config from environment
    let config: Config = envy::from_env().context("error reading config")?;

    // Run application
    if let Err(error) = startup::start(config).await {
        error!("{:?}", error);
        Err(error)?;
    }

    Ok(())
}
