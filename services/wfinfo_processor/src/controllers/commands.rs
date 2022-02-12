use crate::models::{AdminCommand, Config};
use actix_web::{
    dev::HttpServiceFactory,
    error::ErrorInternalServerError,
    http::StatusCode,
    post,
    web::{scope, Data, Json},
};
use serde::Deserialize;
use std::collections::HashMap;
use tracing::instrument;
use wfinfo_azure::functions::{
    FunctionsInput, FunctionsOutput, HttpInput, HttpOutput,
};
use wfinfo_commands::CommandRegistry;
use wfinfo_discord::DiscordRestClient;

pub fn commands_service() -> impl HttpServiceFactory {
    scope("/commands").service(handle_command)
}

#[derive(Clone, Debug, Deserialize)]
pub struct Input {
    pub command: HttpInput<AdminCommand>,
}

#[post("")]
#[instrument(skip(input, command_registry, discord_client, config))]
async fn handle_command(
    input: Json<FunctionsInput<Input>>,
    command_registry: Data<CommandRegistry>,
    discord_client: Data<DiscordRestClient>,
    config: Data<Config>,
) -> Result<Json<FunctionsOutput<HttpOutput<String>>>, actix_web::Error> {
    match input.data.command.body {
        AdminCommand::RegisterCommands => command_registry
            .register_commands(discord_client.as_ref(), config.app_id)
            .await
            .map_err(ErrorInternalServerError)?,
    }

    Ok(Json(FunctionsOutput {
        outputs: HttpOutput {
            status_code: StatusCode::OK.as_u16(),
            headers: HashMap::new(),
            body: "Success!".into(),
        },
        logs: vec![],
        return_value: Default::default(),
    }))
}
