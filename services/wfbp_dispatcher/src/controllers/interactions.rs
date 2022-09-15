use crate::models::{Config, InteractionError, InteractionOutputData};
use axum::{
    body::HttpBody, http::StatusCode, routing::MethodRouter, Extension, Json, BoxError,
};
use std::{collections::HashMap, sync::Arc};
use tracing::instrument;
use wfbp_azure::functions::{FunctionsOutput, HttpOutput};
use wfbp_discord::models::{
    Interaction, InteractionApplicationCommandCallbackData,
    InteractionResponse, InteractionType,
};

pub fn interactions_service<B>() -> MethodRouter<B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    MethodRouter::new().post(handle_interaction)
}

#[instrument(skip_all)]
async fn handle_interaction(
    Json(interaction): Json<Interaction>,
    Extension(config): Extension<Arc<Config>>,
) -> Result<
    Json<
        FunctionsOutput<InteractionOutputData, HttpOutput<InteractionResponse>>,
    >,
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
        outputs: InteractionOutputData {
            message: vec![serde_json::to_string(&interaction)?],
        },
        logs: vec![],
        return_value: HttpOutput {
            status_code: StatusCode::OK.as_u16(),
            headers: {
                let mut headers = HashMap::new();
                headers
                    .insert("content-type".into(), "application/json".into());
                headers
            },
            body: response,
        },
    };

    Ok(Json(response))
}
