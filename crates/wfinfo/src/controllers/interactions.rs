use crate::models::{
    CheckSignatureError, Config, InteractionError, InteractionInputData,
    InteractionOutputData,
};
use actix_web::{
    dev::HttpServiceFactory,
    http::StatusCode,
    post,
    web::{scope, Data, Json},
    HttpRequest,
};
use ed25519_dalek::{Signature, Verifier};
use std::collections::HashMap;
use tracing::{instrument, warn};
use wfinfo_azure::functions::{
    FunctionsInput, FunctionsOutput, HttpOutput, RawHttpInput,
};
use wfinfo_lib::models::{
    Interaction, InteractionApplicationCommandCallbackData,
    InteractionResponse, InteractionResponseDataFlags, InteractionType,
    Snowflake,
};

pub const HEADER_SIGNATURE: &'static str = "x-signature-ed25519";
pub const HEADER_TIMESTAMP: &'static str = "x-signature-timestamp";

pub fn interactions_service(
    config: &Config,
) -> impl HttpServiceFactory + 'static {
    scope("/interactions").service(handle_interaction2)
}

#[post("")]
#[instrument(skip(request, input, config))]
async fn handle_interaction2(
    request: HttpRequest,
    input: Json<FunctionsInput<InteractionInputData<RawHttpInput>>>,
    config: Data<Config>,
) -> Result<Json<FunctionsOutput<InteractionOutputData>>, InteractionError> {
    // Signature validation
    let timestamp = request
        .headers()
        .get(HEADER_TIMESTAMP)
        .ok_or(CheckSignatureError::MissingHeader {
            header_name: HEADER_TIMESTAMP,
            status_code: StatusCode::UNAUTHORIZED,
        })?
        .to_str()
        .map_err(|_| CheckSignatureError::InvalidTimestamp)?;
    let signature = request
        .headers()
        .get(HEADER_SIGNATURE)
        .ok_or(CheckSignatureError::MissingHeader {
            header_name: HEADER_SIGNATURE,
            status_code: StatusCode::UNAUTHORIZED,
        })?
        .to_str()
        .ok()
        .and_then(|signature| hex::decode(signature).ok())
        .and_then(|signature| signature.try_into().ok())
        .map(|signature| Signature::new(signature))
        .ok_or(CheckSignatureError::InvalidSignature)?;
    let message = format!("{}{}", timestamp, input.data.request.body);
    config
        .discord_public_key
        .verify(message.as_bytes(), &signature)
        .map_err(|_| CheckSignatureError::InvalidSignature)?;

    // TODO
    dbg!(&input);

    let interaction: Interaction =
        serde_json::from_str(&input.data.request.body)?;
    if interaction.application_id != config.app_id {
        return Err(InteractionError::UnauthorizedApplication);
    }

    Ok(Json(FunctionsOutput {
        outputs: InteractionOutputData {
            response: HttpOutput {
                status_code: StatusCode::OK.as_u16(),
                headers: HashMap::new(),
                body: InteractionResponse::DeferredChannelMessageWithSource {
                    data: InteractionApplicationCommandCallbackData {
                        flags: Some(InteractionResponseDataFlags::EPHEMERAL),
                        ..Default::default()
                    },
                },
            },
            message: "test-message".into(),
        },
        logs: vec![],
        return_value: None,
    }))
}
