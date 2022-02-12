use anyhow::{bail, Context};
use std::sync::{Arc, Weak};
use tokio::sync::RwLock;
use wfinfo_commands::{
    create_callback, CommandBuilder, CommandRegistry, InteractionData,
    SlashCommand,
};
use wfinfo_discord::{
    models::{CreateWebhookMessage, Snowflake},
    routes::CreateFollowupMessage,
    DiscordRestClient,
};

pub fn admin_command(
    discord_client: DiscordRestClient,
    command_registry: Arc<RwLock<Option<Weak<CommandRegistry>>>>,
    app_id: Snowflake,
) -> SlashCommand {
    CommandBuilder::new()
        .name("admin")
        .description("Admin commands")
        .default_permission(true)
        .subcommand_group_option(|builder| {
            builder.name("commands")
                .description("Commands relating to command management")
                .subcommand_option(|builder| {
                    builder.name("reset")
                        .description("Resets the registered Discord commands")
                        .callback(create_callback! {
                            capture: {
                                discord_client: DiscordRestClient = discord_client.clone(),
                                command_registry: Arc<RwLock<Option<Weak<CommandRegistry>>>> = command_registry.clone(),
                                app_id: Snowflake = app_id,
                            },
                            handler: async |interaction_data, _, _| {
                                reset_commands(
                                    interaction_data,
                                    discord_client.clone(),
                                    command_registry.clone(),
                                    *app_id,
                                )
                                .await
                            }
                        })
                })
        })
        .build()
}

async fn reset_commands(
    interaction_data: Arc<InteractionData>,
    discord_client: DiscordRestClient,
    command_registry: Arc<RwLock<Option<Weak<CommandRegistry>>>>,
    app_id: Snowflake,
) -> anyhow::Result<()> {
    let command_registry = command_registry.read().await;
    let command_registry = match command_registry.as_ref() {
        None => bail!("command registry not set"),
        Some(command_registry) => command_registry,
    };
    let command_registry = match command_registry.upgrade() {
        None => bail!("command registry is already dropped"),
        Some(command_registry) => command_registry,
    };

    command_registry
        .register_commands(&discord_client, app_id)
        .await
        .context("error registering commands")?;

    // Send response
    CreateFollowupMessage::execute(
        &discord_client,
        app_id,
        interaction_data.token.clone(),
        CreateWebhookMessage {
            content: Some("Done!".into()),
            ..Default::default()
        },
    )
    .await
    .context("error creating response")?;

    Ok(())
}
