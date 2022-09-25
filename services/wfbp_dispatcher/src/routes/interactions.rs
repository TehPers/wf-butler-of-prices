use crate::models::{
    Config, InteractionError, InteractionInput, InteractionOutput,
};
use axum::{
    body::{Body, Bytes},
    error_handling::HandleErrorLayer,
    handler::Handler,
    http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::{MethodRouter, Route},
    BoxError, Extension, Json,
};
use ed25519_dalek::PublicKey;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tower::ServiceBuilder;
use tracing::{error, instrument};
use wfbp_discord::models::{
    Interaction, InteractionApplicationCommandCallbackData,
    InteractionResponse, InteractionType,
};
use wfbp_util::{
    layers::{CheckEd25519SignatureLayer, CheckSignatureError},
    models::functions::{FunctionInput, FunctionsOutput, HttpOutput},
    util::{body_to_bytes, ServiceBuilderExt},
};

pub fn interactions_service(
    discord_public_key: Arc<PublicKey>,
) -> MethodRouter {
    let layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_errors))
        // Parse the body from the Functions runtime
        .try_map_request_body(body_to_bytes)
        .try_map_request_body(deserialize_body)
        // Convert it into the inner HTTP trigger's body
        .try_map_request_body(into_body)
        // Check the signature of the request
        .layer(CheckEd25519SignatureLayer::new(discord_public_key))
        .check_service::<Route, Request<Body>, _, _>();

    MethodRouter::new().post(handle_interaction.layer(layer))
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

async fn deserialize_body<T>(body: Bytes) -> Result<T, InteractionError>
where
    T: DeserializeOwned,
{
    Ok(serde_json::from_slice(&body)?)
}

async fn into_body(
    input: FunctionInput<InteractionInput>,
) -> Result<Body, InteractionError> {
    let body = serde_json::to_vec(&input.data.request.body)?;
    Ok(Body::from(Bytes::from(body)))
}

#[instrument(skip_all)]
async fn handle_interaction(
    Json(interaction): Json<Interaction>,
    Extension(config): Extension<Arc<Config>>,
) -> Result<
    Json<FunctionsOutput<InteractionOutput, HttpOutput>>,
    InteractionError,
> {
    // Verify application ID
    if interaction.application_id != config.app_id.into() {
        return Err(InteractionError::UnauthorizedApplication);
    }

    // Create HTTP response
    // TODO: check if response should be ephemeral
    let response = match &interaction.kind {
        InteractionType::Ping => InteractionResponse::Pong,
        InteractionType::ApplicationCommand { .. }
        | InteractionType::MessageComponent { .. }
        | InteractionType::Autocomplete { .. }
        | InteractionType::ModalSubmit { .. } => {
            InteractionResponse::DeferredChannelMessageWithSource {
                data: InteractionApplicationCommandCallbackData {
                    ..Default::default()
                },
            }
        }
    };

    // Enqueue message and return HTTP response
    let response = FunctionsOutput {
        outputs: InteractionOutput {
            message: vec![serde_json::to_string(&interaction)?].into(),
        },
        logs: vec![],
        return_value: HttpOutput {
            status: StatusCode::OK,
            headers: {
                let mut headers = HeaderMap::new();
                headers.insert(
                    CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                );
                headers
            },
            body: serde_json::to_string(&response)?,
        },
    };

    Ok(Json(response))
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use ed25519_dalek::Keypair;
    use serde_json::{json, Value};
    use wfbp_discord::models::{
        ApplicationId, ChannelId, GuildId, InteractionId,
    };

    const PRIVATE_KEY: &[u8] = &[
        0xB3, 0x15, 0xA2, 0x2D, 0xD0, 0x79, 0x05, 0x75, 0x04, 0x9E, 0xA3, 0x99,
        0xC5, 0x04, 0x1D, 0x27, 0xC4, 0xDD, 0xE3, 0xFB, 0xA4, 0x2D, 0x67, 0x0F,
        0x51, 0xA4, 0x33, 0xE0, 0x83, 0xE1, 0xEA, 0xAE, 0xB9, 0x31, 0xC7, 0xCD,
        0x87, 0x08, 0x0A, 0xF7, 0xF6, 0x7C, 0x1D, 0x22, 0xEB, 0x87, 0x9A, 0xD3,
        0x3F, 0x66, 0x05, 0xE3, 0x42, 0x0E, 0xD8, 0x74, 0x00, 0xCC, 0xA9, 0x1C,
        0x72, 0x53, 0xE3, 0x90,
    ];

    #[tokio::test]
    async fn handle_interaction_responds_to_pings() {
        let config = Arc::new(Config {
            app_id: ApplicationId::new(1),
            client_id: ApplicationId::new(1),
            client_secret: Default::default(),
            ignore_signature: Default::default(),
            discord_public_key: Keypair::from_bytes(PRIVATE_KEY)
                .expect("invalid private key")
                .public
                .into(),
            port: Default::default(),
        });
        let interaction = Json(Interaction {
            id: InteractionId::new(2),
            application_id: ApplicationId::new(3),
            kind: InteractionType::Ping,
            guild_id: Some(GuildId::new(4)),
            channel_id: ChannelId::new(5),
            member: None,
            user: None,
            token: String::new(),
            version: 0,
        });

        let response = handle_interaction(interaction, Extension(config))
            .await
            .expect("failed to handle interaction")
            .0;

        assert_eq!(response.return_value.status, StatusCode::OK);
        assert_eq!(
            serde_json::from_str::<Value>(&response.return_value.body)
                .expect("invalid response JSON"),
            json!({ "type": 1 }),
        );
    }
}
