use crate::{
    middleware::CheckEd25519Signature,
    models::{AdminCommand, AdminCommandError, Config},
};
use actix_web::{
    dev::HttpServiceFactory,
    middleware::Condition,
    post,
    web::{scope, Data, Json},
};
use tracing::{error, instrument};
use wfinfo_lib::{
    client::DiscordRestClient, http::Route, models::CreateApplicationCommand,
};

pub fn commands_service(config: &Config) -> impl HttpServiceFactory {
    scope("/api/commands")
        .wrap(Condition::new(
            !config.ignore_signature,
            CheckEd25519Signature::new(config.admin_public_key.clone().into()),
        ))
        .service(handle_command)
}

#[post("")]
#[instrument]
async fn handle_command(
    config: Data<Config>,
    command: Json<AdminCommand>,
    client: Data<DiscordRestClient>,
) -> Result<String, AdminCommandError> {
    // TODO: oauth2 client credentials

    match command.into_inner() {
        AdminCommand::RegisterCommands => {
            Route::create_global_application_command(
                client.as_ref(),
                config.app_id,
                CreateApplicationCommand {
                    name: "test".into(),
                    description: "Test command".into(),
                    options: None,
                    default_permission: None,
                },
            )
            .await
            .map_err(|cause| {
                error!("{:?}", cause);
                AdminCommandError::CommandFailed { cause }
            })?;

            Ok("Done!".to_owned())
        }
    }
}
