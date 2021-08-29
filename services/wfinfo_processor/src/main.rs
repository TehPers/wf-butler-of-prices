mod controllers;
mod middleware;
mod models;
mod startup;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    use tracing::{error, Level};
    use tracing_subscriber::FmtSubscriber;

    // Setup tracing
    FmtSubscriber::builder()
        .compact()
        .with_max_level(Level::DEBUG)
        .try_init()
        .expect("error registering tracing subscriber");

    // Run application
    if let Err(error) = startup::start().await {
        error!("{:?}", error);
        Err(error)?;
    }

    Ok(())
}
