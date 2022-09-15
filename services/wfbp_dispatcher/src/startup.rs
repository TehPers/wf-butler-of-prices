use crate::{
    controllers::interactions_service,
    models::{Config, InteractionError},
};
use anyhow::Context;
use axum::{
    body::{Body, Bytes},
    error_handling::HandleErrorLayer,
    http::Request,
    response::{IntoResponse, Response},
    routing::Route,
    Router, Server,
};
use serde::Deserialize;
use std::{net::Ipv4Addr, sync::Arc};
use tower::{BoxError, ServiceBuilder};
use tracing::error;
use wfbp_util::{
    layers::{CheckEd25519SignatureLayer, CheckSignatureError},
    models::functions::{FunctionInput, FunctionsHttpInput},
    util::{body_to_bytes, ServiceBuilderExt as _},
};

pub async fn start() -> anyhow::Result<()> {
    // Read config from environment
    let config: Config = envy::from_env().context("error reading config")?;

    // Setup web server
    let layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_errors))
        // Parse the body from the Functions runtime
        .try_map_request_body(body_to_bytes)
        .try_map_request_body(deserialize_body)
        // Convert it into the inner HTTP trigger's body
        .try_map_request_body(into_body)
        // Check the signature of the request
        .layer(CheckEd25519SignatureLayer::new(Arc::new(
            config.discord_public_key.into(),
        )))
        .check_service::<Route, Request<Body>, _, _>();
    let app = Router::new()
        .route("/interactions", interactions_service())
        .layer(layer);

    // Start web server
    Server::bind(&(Ipv4Addr::UNSPECIFIED, config.port).into())
        .serve(app.into_make_service())
        .await
        .context("error running web server")
}

#[derive(Clone, Debug, Deserialize)]
struct InteractionInput {
    pub request: FunctionsHttpInput,
}

async fn handle_errors(error: BoxError) -> Response {
    macro_rules! match_downcast {
        (
            $error:expr,
            $(if $type:ty : $pattern:pat => $value:expr,)*
            $(else: $default_pattern:pat_param => $default_value:expr,)?
        ) => {
            loop {
                let error = $error;
                $(
                    let error = match error.downcast::<$type>().map(|err| *err) {
                        Ok($pattern) => break $value,
                        Err(error) => error
                    };
                )*
                $(
                    let $default_pattern = error;
                    break $default_value;
                )?
            }
        };
    }

    // Downcast the error
    let error = match_downcast!(
        error,
        if InteractionError: error => error,
        if CheckSignatureError: error => error.into(),
        else: _ => InteractionError::InternalServerError,
    );

    // Unhandled error
    error!(?error, "unexpected error");
    InteractionError::InternalServerError.into_response()
}

async fn deserialize_body(
    body: Bytes,
) -> Result<FunctionInput<InteractionInput>, InteractionError> {
    Ok(serde_json::from_slice(&body)?)
}

async fn into_body(
    input: FunctionInput<InteractionInput>,
) -> Result<Body, InteractionError> {
    let body = serde_json::to_vec(&input.data.request.body)?;
    Ok(Body::from(Bytes::from(body)))
}
