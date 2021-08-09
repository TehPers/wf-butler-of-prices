use crate::{
    middleware::CheckEd25519Signature,
    models::{Config, InteractionError},
};
use actix_web::{
    middleware::{Condition, Logger},
    post,
    web::{Data, Json},
    App, HttpServer,
};
use anyhow::Context;
use std::net::Ipv4Addr;
use tracing::{instrument, warn};
use wfinfo_lib::{
    client::WebhookDiscordClient,
    models::{
        Interaction, InteractionApplicationCommandCallbackData,
        InteractionResponse, InteractionType,
    },
};

#[instrument]
pub async fn start() -> anyhow::Result<()> {
    // Read config from environment
    let mut config: Config =
        envy::from_env().context("error reading config")?;

    // Shared data
    let discord_client =
        WebhookDiscordClient::new(std::mem::take(&mut config.app_secret))?;
    let port = config.port;

    // Start web server
    // TODO: fix errors returning non-json response
    HttpServer::new(move || {
        let logger = Logger::default();
        let signature_verifier =
            CheckEd25519Signature::new(config.public_key.clone());

        App::new()
            .app_data(Data::new(discord_client.clone()))
            .app_data(Data::new(config.clone()))
            .wrap(Condition::new(!config.ignore_signature, signature_verifier))
            .wrap(logger)
            .service(handle_interaction)
    })
    .bind((Ipv4Addr::UNSPECIFIED, port))?
    .run()
    .await
    .context("error running web server")
}

#[post("/api/GetInteraction")]
#[instrument(skip(interaction))]
async fn handle_interaction(
    interaction: Json<Interaction>,
    config: Data<Config>,
) -> Result<Json<InteractionResponse>, InteractionError> {
    let interaction = interaction.into_inner();

    // Verify application ID
    if interaction.application_id != config.app_id {
        return Err(InteractionError::UnauthorizedApplication);
    }

    // Handle interaction
    match interaction.kind {
        InteractionType::Ping => Ok(Json(InteractionResponse::Pong)),
        InteractionType::ApplicationCommand { .. } => {
            let response = InteractionResponse::ChannelMessageWithSource {
                data: InteractionApplicationCommandCallbackData {
                    content: Some("Hello, world!".to_owned()),
                    ..Default::default()
                },
            };

            Ok(Json(response))
        }
        interaction @ _ => {
            warn!(?interaction, "received unhandled interaction");
            Err(InteractionError::NotImplemented)
        }
    }
}
