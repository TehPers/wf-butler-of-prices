mod middleware;
mod models;
mod startup;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    use tracing::error;

    // Setup tracing
    tracing_subscriber::fmt().compact().try_init().unwrap();

    // Run application
    if let Err(error) = startup::start().await {
        error!("{:?}", error);
        Err(error)?;
    }

    Ok(())
}
