use std::collections::HashMap;

use anyhow::{bail, Context};
use wfbp_discord::{
    models::{
        ApplicationCommandInteractionData, GuildMember, Interaction,
        InteractionType, Snowflake, User,
    },
    DiscordRestClient,
};

use super::{SlashCommandHandler, SlashCommandOpts};

/// Handler for Discord interactions.
#[derive(Debug, Default)]
pub struct InteractionHandler {
    application_command_handler: ApplicationCommandHandler,
}

impl InteractionHandler {
    pub async fn handle(
        &self,
        client: DiscordRestClient,
        interaction: Interaction,
    ) -> anyhow::Result<()> {
        let root_data = RootInteractionData {
            client,
            app_id: interaction.application_id,
            response_token: interaction.token,
        };
        match interaction.kind {
            InteractionType::Ping => Ok(()),
            InteractionType::ApplicationCommand {
                guild_id,
                channel_id,
                member,
                user,
                data,
            } => {
                self.application_command_handler
                    .handle(root_data, guild_id, channel_id, member, user, data)
                    .await
            }
            InteractionType::MessageComponent {
                guild_id,
                channel_id,
                member,
                user,
                message,
                component_type,
            } => bail!("TODO"),
            InteractionType::Autocomplete {
                guild_id,
                channel_id,
                member,
                user,
            } => bail!("TODO"),
            InteractionType::ModalSubmit {
                guild_id,
                channel_id,
                member,
                user,
                message,
            } => bail!("TODO"),
        }
    }

    pub async fn register_all(&self) -> anyhow::Result<()> {
        // TODO
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct RootInteractionData {
    pub client: DiscordRestClient,
    pub app_id: Snowflake,
    pub response_token: String,
}

#[derive(Debug, Default)]
pub struct ApplicationCommandHandler {
    slash_commands: HashMap<String, Box<dyn SlashCommandHandler>>,
}

impl ApplicationCommandHandler {
    #[inline]
    pub fn slash_commands(
        &self,
    ) -> &HashMap<String, Box<dyn SlashCommandHandler>> {
        &self.slash_commands
    }

    #[inline]
    pub fn slash_commands_mut(
        &mut self,
    ) -> &mut HashMap<String, Box<dyn SlashCommandHandler>> {
        &mut self.slash_commands
    }

    pub async fn handle(
        &self,
        root_data: RootInteractionData,
        guild_id: Option<Snowflake>,
        channel_id: Snowflake,
        member: Option<GuildMember>,
        user: Option<User>,
        data: ApplicationCommandInteractionData,
    ) -> anyhow::Result<()> {
        match data {
            ApplicationCommandInteractionData::SlashCommand {
                id,
                name,
                resolved,
                options,
            } => {
                let handler = self
                    .slash_commands
                    .get(&name)
                    .with_context(|| format!("no handler for {}", name))?;
                let opts = SlashCommandOpts::new(options.into_iter().flatten());
                handler.handle(opts).await
            }
            ApplicationCommandInteractionData::User {
                id,
                name,
                target_id,
            } => bail!("TODO"),
            ApplicationCommandInteractionData::Message {
                id,
                name,
                target_id,
            } => bail!("TODO"),
        }
    }
}
