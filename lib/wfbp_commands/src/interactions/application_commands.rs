use crate::interactions::{
    InteractionResponder, SlashCommandArgs, SlashCommandHandler,
};
use anyhow::{bail, Context};
use derive_more::{Display, Error};
use runtime_injector::Svc;
use std::collections::HashMap;
use wfbp_discord::{
    models::{
        ApplicationCommandInteractionData,
        ApplicationCommandInteractionDataType, ApplicationId, ChannelId,
        CreateApplicationCommand, CreateApplicationCommandType, GuildId,
        GuildMember, User,
    },
    routes::CreateGlobalApplicationCommand,
    DiscordRestClient,
};

#[derive(Clone, Debug, Display, Error)]
pub enum ApplicationCommandHandlerInitError {
    #[display(fmt = "multiple commands have the same name: '{name}'")]
    ConflictingCommands { name: String },
}

#[derive(Debug)]
pub struct ApplicationCommandHandler {
    client: DiscordRestClient,
    app_id: ApplicationId,
    slash_commands: HashMap<String, Svc<dyn SlashCommandHandler>>,
}

impl ApplicationCommandHandler {
    pub fn new(
        // client: Box<DiscordRestClient>,
        app_id: Box<ApplicationId>,
        slash_commands: Vec<Svc<dyn SlashCommandHandler>>,
    ) -> Result<Self, ApplicationCommandHandlerInitError> {
        let mut commands = HashMap::new();
        for command in slash_commands {
            if let Some(conflict) = commands.insert(command.name(), command) {
                return Err(
                    ApplicationCommandHandlerInitError::ConflictingCommands {
                        name: conflict.name(),
                    },
                );
            }
        }

        Ok(ApplicationCommandHandler {
            client: *client,
            app_id: *app_id,
            slash_commands: commands,
        })
    }

    pub async fn handle(
        &self,
        responder: InteractionResponder,
        guild_id: Option<GuildId>,
        channel_id: ChannelId,
        member: Option<GuildMember>,
        user: Option<User>,
        command_data: ApplicationCommandInteractionData,
    ) -> anyhow::Result<()> {
        let ApplicationCommandInteractionData {
            id: command_id,
            name: command_name,
            kind: command_type,
        } = command_data;

        match command_type {
            ApplicationCommandInteractionDataType::SlashCommand {
                resolved,
                options,
            } => {
                let handler =
                    self.slash_commands.get(&command_name).with_context(
                        || format!("no handler for {command_name}"),
                    )?;
                let opts = options.into_iter().flatten().collect();
                let args = SlashCommandArgs {
                    command_id,
                    opts,
                    resolver: resolved.map(Into::into).unwrap_or_default(),
                    responder,
                };
                handler.handle(args).await
            }
            ApplicationCommandInteractionDataType::User { target_id } => {
                bail!("TODO")
            }
            ApplicationCommandInteractionDataType::Message { target_id } => {
                bail!("TODO")
            }
        }
    }

    pub async fn register_all(&self) -> anyhow::Result<()> {
        for (name, command) in self.slash_commands.iter() {
            let command = CreateApplicationCommand {
                name: name.to_owned(),
                name_localizations: command.name_localizations(),
                description: command.description(),
                description_localizations: command.description_localizations(),
                default_permission: Some(command.default_permission()),
                kind: CreateApplicationCommandType::ChatInput {
                    options: Some(command.options()),
                },
            };

            CreateGlobalApplicationCommand::execute(
                &self.client,
                self.app_id,
                command,
            )
            .await?;
        }

        Ok(())
    }
}
