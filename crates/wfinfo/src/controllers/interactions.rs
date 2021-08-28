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
use wfinfo_azure::functions::{
    FunctionsInput, FunctionsOutput, HttpOutput, RawHttpInput,
};
use wfinfo_lib::models::{
    Interaction, InteractionApplicationCommandCallbackData,
    InteractionResponse, InteractionResponseDataFlags, InteractionType,
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
    // Signature validation
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
            .and_then(|signature| signature.try_into().ok())
            .map(|signature| Signature::new(signature))
            .ok_or_else(|| CheckSignatureError::InvalidSignature(signature))?;
        let message = format!("{}{}", timestamp, input.data.request.body);
        config
            .discord_public_key
            .verify(message.as_bytes(), &signature)
            .map_err(|_| CheckSignatureError::VerificationFailed)?;
    }

    // TODO
    let interaction: Interaction =
        serde_json::from_str(&input.data.request.body)
            .map_err(|err| InteractionError::InvalidBody(err))?;
    if interaction.application_id != config.app_id {
        return Err(InteractionError::UnauthorizedApplication);
    }

    let response = match &interaction.kind {
        InteractionType::Ping => InteractionResponse::Pong,
        InteractionType::ApplicationCommand { .. } => {
            InteractionResponse::DeferredChannelMessageWithSource {
                data: InteractionApplicationCommandCallbackData {
                    flags: Some(InteractionResponseDataFlags::EPHEMERAL),
                    ..Default::default()
                },
            }
        }
        InteractionType::MessageComponent { .. } => {
            InteractionResponse::DeferredChannelMessageWithSource {
                data: InteractionApplicationCommandCallbackData {
                    flags: Some(InteractionResponseDataFlags::EPHEMERAL),
                    ..Default::default()
                },
            }
        }
    };

    Ok(Json(FunctionsOutput {
        outputs: InteractionOutputData {
            message: vec![input.data.request.body.clone()],
        },
        logs: vec![],
        return_value: HttpOutput {
            status_code: StatusCode::OK.as_u16(),
            headers: HashMap::new(),
            body: response,
        },
    }))
}
