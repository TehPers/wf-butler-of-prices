use crate::{InteractionData, SlashCommand, SlashCommandData};
use anyhow::{bail, Context};
use std::{borrow::Cow, collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{debug, error};
use wfbp_discord::{
    models::{
        ApplicationCommandInteractionData, Interaction, InteractionType,
        Snowflake,
    },
    routes::BulkOverwriteGlobalApplicationCommands,
    DiscordRestClient,
};

pub struct CommandRegistry {
    slash_commands: RwLock<HashMap<Cow<'static, str>, SlashCommand>>,
}

impl CommandRegistry {
    pub fn new(
        slash_commands: impl IntoIterator<Item = SlashCommand>,
    ) -> Arc<Self> {
        let registry = CommandRegistry {
            slash_commands: RwLock::new(
                slash_commands
                    .into_iter()
                    .map(|command| (command.name.clone(), command))
                    .collect(),
            ),
        };

        Arc::new(registry)
    }

    pub async fn register_commands(
        &self,
        client: &DiscordRestClient,
        app_id: Snowflake,
    ) -> anyhow::Result<()> {
        let slash_commands = self.slash_commands.read().await;
        let commands = slash_commands.values().map(Into::into).collect();

        let result = BulkOverwriteGlobalApplicationCommands::execute(
            client, app_id, commands,
        )
        .await;
        if let Err(error) = result.as_ref() {
            error!("{:#?}", error);
        }
        result.context("error overriding application commands")?;

        Ok(())
    }

    pub async fn handle_interaction(
        &self,
        interaction: Interaction,
    ) -> anyhow::Result<()> {
        match interaction.kind {
            InteractionType::Ping => Ok(()),
            InteractionType::ApplicationCommand {
                data,
                guild_id,
                channel_id,
                member,
                user,
            } => {
                debug!("handling application command");
                let interaction_data = Arc::new(InteractionData {
                    id: interaction.id,
                    application_id: interaction.application_id,
                    token: interaction.token,
                    guild_id,
                    channel_id,
                    member,
                    user,
                });

                match data {
                    ApplicationCommandInteractionData::SlashCommand {
                        id,
                        name,
                        resolved,
                        options,
                    } => {
                        debug!("handling slash command");
                        let slash_commands = self.slash_commands.read().await;
                        let command = match slash_commands.get(name.as_str()) {
                            Some(command) => command,
                            None => bail!("command not found: '{name}'"),
                        };

                        let command_data = SlashCommandData {
                            command_id: id,
                            name,
                            resolved: resolved.unwrap_or_default(),
                            options: options.unwrap_or_default(),
                        };

                        command
                            .handle(interaction_data, command_data)
                            .await
                            .context("error handling command")
                    }
                    ApplicationCommandInteractionData::User { .. } => {
                        bail!("user commands not implemented")
                    }
                    ApplicationCommandInteractionData::Message { .. } => {
                        bail!("message commands not implemented")
                    }
                }
            }
            InteractionType::MessageComponent { .. } => {
                bail!("message components not implemented")
            }
            InteractionType::Autocomplete { .. } => {
                bail!("autocomplete not implemented")
            }
            InteractionType::ModalSubmit { .. } => {
                bail!("modals not implemented")
            }
        }
    }
}
