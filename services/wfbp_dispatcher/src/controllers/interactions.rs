use crate::models::{
    CheckSignatureError, Config, InteractionError, InteractionInputData,
    InteractionOutputData,
};
use actix_web::{
    dev::HttpServiceFactory,
    http::StatusCode,
    post,
    web::{scope, Data, Json},
};
use ed25519_dalek::{Signature, Verifier};
use std::collections::HashMap;
use tracing::instrument;
use wfbp_azure::functions::{
    FunctionsInput, FunctionsOutput, HttpOutput, RawHttpInput,
};
use wfbp_discord::models::{
    Interaction, InteractionApplicationCommandCallbackData,
    InteractionResponse, InteractionType,
};

pub const HEADER_SIGNATURE: &'static str = "x-signature-ed25519";
pub const HEADER_TIMESTAMP: &'static str = "x-signature-timestamp";

pub fn interactions_service() -> impl HttpServiceFactory + 'static {
    scope("/interactions").service(handle_interaction)
}

#[post("")]
#[instrument(skip(input, config))]
async fn handle_interaction(
    input: Json<FunctionsInput<InteractionInputData<RawHttpInput>>>,
    config: Data<Config>,
) -> Result<
    Json<
        FunctionsOutput<InteractionOutputData, HttpOutput<InteractionResponse>>,
    >,
    InteractionError,
> {
    // Validate signature
    if !config.ignore_signature {
        let timestamp = input
            .data
            .request
            .headers
            .get(HEADER_TIMESTAMP)
            .ok_or(CheckSignatureError::MissingHeader {
                header_name: HEADER_TIMESTAMP,
                status_code: StatusCode::UNAUTHORIZED,
            })?
            .join(",");
        // TODO: verify timestamp is valid
        let signature = input
            .data
            .request
            .headers
            .get(HEADER_SIGNATURE)
            .ok_or(CheckSignatureError::MissingHeader {
                header_name: HEADER_SIGNATURE,
                status_code: StatusCode::UNAUTHORIZED,
            })?
            .join(",");
        let signature = hex::decode(&signature)
            .ok()
            .and_then(|signature| Signature::from_bytes(&signature).ok())
            .ok_or_else(|| CheckSignatureError::InvalidSignature(signature))?;
        let message = format!("{}{}", timestamp, input.data.request.body);
        config
            .discord_public_key
            .verify(message.as_bytes(), &signature)
            .map_err(|_| CheckSignatureError::VerificationFailed)?;
    }

    // Verify application ID
    let interaction: Interaction =
        serde_json::from_str(&input.data.request.body)
            .map_err(|err| InteractionError::InvalidBody(err))?;
    if interaction.application_id != config.app_id {
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
            message: vec![input.data.request.body.clone()],
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
