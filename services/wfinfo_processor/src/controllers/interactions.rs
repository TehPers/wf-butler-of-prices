use crate::models::CommandError;
use actix_web::{
    dev::HttpServiceFactory,
    post,
    web::{scope, Data, Json},
};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use tracing::{error, instrument};
use wfinfo_azure::functions::{FunctionsInput, FunctionsOutput};
use wfinfo_commands::CommandRegistry;
use wfinfo_discord::models::Interaction;

pub fn interactions_service() -> impl HttpServiceFactory {
    scope("/interactions").service(handle_interaction)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Input {
    pub command: String,
}

#[post("")]
#[instrument(skip(input, command_registry))]
async fn handle_interaction(
    input: Json<FunctionsInput<Input>>,
    command_registry: Data<CommandRegistry>,
) -> Result<Json<FunctionsOutput<()>>, CommandError> {
    let input_body: String = serde_json::from_str(&input.data.command)
        .map_err(CommandError::ParseError)?;
    let input =
        serde_json::from_str(&input_body).map_err(CommandError::ParseError);
    if input.is_err() {
        error!("{:#?}", input_body);
    }
    let input: Interaction = input?;

    let result = command_registry
        .handle_interaction(input)
        .await
        .context("error handling interaction");
    if let Err(error) = result {
        error!("{:?}", error);
    }

    Ok(Json(FunctionsOutput {
        outputs: (),
        logs: vec![],
        return_value: None,
    }))
}
