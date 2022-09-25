use crate::{models::Config, routes::interactions_service};
use anyhow::Context;
use axum::{Router, Server};
use std::{net::Ipv4Addr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

fn build_app(config: Config) -> Router {
    Router::new()
        .route(
            "/interactions",
            interactions_service(Arc::new(config.discord_public_key.into())),
        )
        .layer(ServiceBuilder::new().compression())
}

pub async fn start(config: Config) -> anyhow::Result<()> {
    let port = config.port;
    let app = build_app(config);
    Server::bind(&(Ipv4Addr::UNSPECIFIED, port).into())
        .serve(app.into_make_service())
        .await
        .context("error running web server")
}
