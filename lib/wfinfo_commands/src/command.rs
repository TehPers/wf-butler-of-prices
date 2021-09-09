use crate::{FromOption, FromOptionError};
use async_recursion::async_recursion;
use async_trait::async_trait;
use derive_more::{Display, Error};
use std::{
    borrow::Cow,
    fmt::{Debug, Formatter},
    sync::Arc,
};
use wfinfo_discord::{
    models::{
        ApplicationCommand, ApplicationCommandInteractionDataOption,
        ApplicationCommandInteractionDataOptionType,
        ApplicationCommandInteractionDataResolved, ApplicationCommandOption,
        ApplicationCommandOptionChoice, ApplicationCommandOptionType,
        CreateApplicationCommand, GuildMember, Snowflake, User,
    },
    routes::CreateGlobalApplicationCommand,
    DiscordRestClient,
};
use wfinfo_lib::http::RequestError;

pub struct SlashCommand {
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub options: Vec<CommandOption>,
    pub default_permission: Option<bool>,
    pub callback: Option<Box<dyn CommandCallback>>,
}

impl SlashCommand {
    pub async fn register(
        &self,
        client: &DiscordRestClient,
        application_id: Snowflake,
    ) -> Result<ApplicationCommand, RequestError> {
        CreateGlobalApplicationCommand::execute(
            client,
            application_id,
            self.into(),
        )
        .await
    }

    pub async fn handle(
        &self,
        interaction_data: Arc<InteractionData>,
        root_data: SlashCommandData,
    ) -> Result<(), HandleInteractionError> {
        if root_data.name != self.name {
            return Err(HandleInteractionError::UnknownCommand(
                root_data.name.clone(),
            ));
        }

        // Callback
        if let Some(callback) = self.callback.as_ref() {
            let option_registry =
                CommandOptionRegistry::new(&root_data.options);
            callback
                .invoke(interaction_data.clone(), &root_data, option_registry)
                .await?;
        }

        // Options
        for invoke_option in root_data.options.iter() {
            self.options
                .iter()
                .find(|option| option.name == invoke_option.name)
                .ok_or_else(|| {
                    HandleInteractionError::UnknownOption(
                        invoke_option.name.clone(),
                    )
                })?
                .handle(interaction_data.clone(), &root_data, &invoke_option)
                .await?;
        }

        Ok(())
    }
}

#[async_trait]
pub trait CommandCallback: Send + Sync + 'static {
    async fn invoke<'a>(
        &self,
        interaction_data: Arc<InteractionData>,
        invoke_data: &'a SlashCommandData,
        options: CommandOptionRegistry<'a>,
    ) -> Result<(), HandleInteractionError>;
}

#[async_trait]
impl<F> CommandCallback for F
where
    F: Send + Sync + 'static,
    F: for<'a> Fn(
        Arc<InteractionData>,
        &'a SlashCommandData,
        CommandOptionRegistry<'a>,
    ) -> Result<(), HandleInteractionError>,
{
    async fn invoke<'a>(
        &self,
        interaction_data: Arc<InteractionData>,
        invoke_data: &'a SlashCommandData,
        options: CommandOptionRegistry<'a>,
    ) -> Result<(), HandleInteractionError> {
        (self)(interaction_data, invoke_data, options)
    }
}

#[derive(Clone, Debug)]
pub struct InteractionData {
    pub id: Snowflake,
    pub application_id: Snowflake,
    pub token: String,
    pub guild_id: Option<Snowflake>,
    pub channel_id: Snowflake,
    pub member: Option<GuildMember>,
    pub user: Option<User>,
}

#[derive(Clone, Debug)]
pub struct SlashCommandData {
    pub command_id: Snowflake,
    pub name: String,
    pub resolved: ApplicationCommandInteractionDataResolved,
    pub options: Vec<ApplicationCommandInteractionDataOption>,
}

#[non_exhaustive]
#[derive(Debug, Display, Error)]
pub enum HandleInteractionError {
    #[display(fmt = "unknown command '{}'", _0)]
    UnknownCommand(#[error(ignore)] String),
    #[display(fmt = "unknown option: '{}'", _0)]
    UnknownOption(#[error(ignore)] String),
    #[display(fmt = "{}", _0)]
    OptionError(GetOptionError),
    #[display(fmt = "invalid options: '{}'", _0)]
    InvalidData(#[error(ignore)] String),
    #[display(fmt = "{}", _0)]
    Custom(#[error(ignore)] anyhow::Error),
}

impl From<anyhow::Error> for HandleInteractionError {
    fn from(error: anyhow::Error) -> Self {
        HandleInteractionError::Custom(error)
    }
}

impl From<GetOptionError> for HandleInteractionError {
    fn from(error: GetOptionError) -> Self {
        HandleInteractionError::OptionError(error)
    }
}

impl From<&SlashCommand> for CreateApplicationCommand {
    fn from(command: &SlashCommand) -> Self {
        CreateApplicationCommand::ChatInput {
            name: command.name.to_string(),
            description: command.description.to_string(),
            options: if command.options.is_empty() {
                None
            } else {
                Some(command.options.iter().map(Into::into).collect())
            },
            default_permission: command.default_permission,
        }
    }
}

#[derive(Debug)]
pub struct CommandOption {
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub kind: CommandOptionType,
}

pub enum CommandOptionType {
    SubCommand {
        options: Vec<CommandOption>,
        callback: Option<Box<dyn CommandCallback>>,
    },
    SubCommandGroup {
        options: Vec<CommandOption>,
    },
    String {
        required: Option<bool>,
        choices: Option<Vec<Choice<Cow<'static, str>>>>,
    },
    Integer {
        required: Option<bool>,
        choices: Option<Vec<Choice<i64>>>,
    },
    Number {
        required: Option<bool>,
        choices: Option<Vec<Choice<f64>>>,
    },
    Boolean {
        required: Option<bool>,
    },
    User {
        required: Option<bool>,
    },
    Channel {
        required: Option<bool>,
    },
    Role {
        required: Option<bool>,
    },
    Mentionable {
        required: Option<bool>,
    },
}

impl CommandOption {
    #[async_recursion]
    pub async fn handle(
        &self,
        interaction_data: Arc<InteractionData>,
        root_data: &SlashCommandData,
        command_data: &ApplicationCommandInteractionDataOption,
    ) -> Result<(), HandleInteractionError> {
        match &self.kind {
            CommandOptionType::SubCommand {
                options,
                callback: Some(ref callback),
            } => match &command_data.kind {
                ApplicationCommandInteractionDataOptionType::SubCommand {
                    options: invoke_options,
                } => {
                    // Options
                    for invoke_option in root_data.options.iter() {
                        options
                            .iter()
                            .find(|option| option.name == invoke_option.name)
                            .ok_or_else(|| {
                                HandleInteractionError::UnknownOption(
                                    invoke_option.name.clone(),
                                )
                            })?
                            .handle(
                                interaction_data.clone(),
                                &root_data,
                                &invoke_option,
                            )
                            .await?;
                    }

                    // Callback
                    callback
                        .invoke(
                            interaction_data.clone(),
                            root_data,
                            CommandOptionRegistry::new(invoke_options),
                        )
                        .await
                }
                _ => Err(HandleInteractionError::InvalidData(
                    self.name.to_string(),
                )),
            },
            CommandOptionType::SubCommandGroup { options } => {
                // Options
                for invoke_option in root_data.options.iter() {
                    options
                        .iter()
                        .find(|option| option.name == invoke_option.name)
                        .ok_or_else(|| {
                            HandleInteractionError::UnknownOption(
                                invoke_option.name.clone(),
                            )
                        })?
                        .handle(
                            interaction_data.clone(),
                            &root_data,
                            &invoke_option,
                        )
                        .await?;
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl From<&CommandOption> for ApplicationCommandOption {
    fn from(option: &CommandOption) -> Self {
        let kind = match &option.kind {
            CommandOptionType::SubCommand { options, .. } => {
                ApplicationCommandOptionType::SubCommand {
                    options: if options.is_empty() {
                        None
                    } else {
                        Some(options.iter().map(Into::into).collect())
                    },
                }
            }
            CommandOptionType::SubCommandGroup { options, .. } => {
                ApplicationCommandOptionType::SubCommandGroup {
                    options: if options.is_empty() {
                        None
                    } else {
                        Some(options.iter().map(Into::into).collect())
                    },
                }
            }
            CommandOptionType::String { required, choices } => {
                ApplicationCommandOptionType::String {
                    required: *required,
                    choices: choices.as_ref().map(|choices| {
                        choices.iter().map(Into::into).collect()
                    }),
                }
            }
            CommandOptionType::Integer { required, choices } => {
                ApplicationCommandOptionType::Integer {
                    required: *required,
                    choices: choices.as_ref().map(|choices| {
                        choices.iter().map(Into::into).collect()
                    }),
                }
            }
            CommandOptionType::Number { required, choices } => {
                ApplicationCommandOptionType::Number {
                    required: *required,
                    choices: choices.as_ref().map(|choices| {
                        choices.iter().map(Into::into).collect()
                    }),
                }
            }
            CommandOptionType::Boolean { required } => {
                ApplicationCommandOptionType::Boolean {
                    required: *required,
                }
            }
            CommandOptionType::User { required } => {
                ApplicationCommandOptionType::User {
                    required: *required,
                }
            }
            CommandOptionType::Channel { required } => {
                ApplicationCommandOptionType::Channel {
                    required: *required,
                }
            }
            CommandOptionType::Role { required } => {
                ApplicationCommandOptionType::Role {
                    required: *required,
                }
            }
            CommandOptionType::Mentionable { required } => {
                ApplicationCommandOptionType::Mentionable {
                    required: *required,
                }
            }
        };

        ApplicationCommandOption {
            name: option.name.to_string(),
            description: option.description.to_string(),
            kind,
        }
    }
}

impl Debug for CommandOptionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandOptionType::SubCommand {
                options,
                callback: _,
            } => f
                .debug_struct("SubCommand")
                .field("options", options)
                .finish_non_exhaustive(),
            CommandOptionType::SubCommandGroup { options } => f
                .debug_struct("SubCommandGroup")
                .field("options", options)
                .finish(),
            CommandOptionType::String { required, choices } => f
                .debug_struct("String")
                .field("required", required)
                .field("choices", choices)
                .finish(),
            CommandOptionType::Integer { required, choices } => f
                .debug_struct("Integer")
                .field("required", required)
                .field("choices", choices)
                .finish(),
            CommandOptionType::Number { required, choices } => f
                .debug_struct("Number")
                .field("required", required)
                .field("choices", choices)
                .finish(),
            CommandOptionType::Boolean { required } => f
                .debug_struct("Boolean")
                .field("required", required)
                .finish(),
            CommandOptionType::User { required } => {
                f.debug_struct("User").field("required", required).finish()
            }
            CommandOptionType::Channel { required } => f
                .debug_struct("Channel")
                .field("required", required)
                .finish(),
            CommandOptionType::Role { required } => {
                f.debug_struct("Role").field("required", required).finish()
            }
            CommandOptionType::Mentionable { required } => f
                .debug_struct("Mentionable")
                .field("required", required)
                .finish(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Choice<T> {
    pub name: Cow<'static, str>,
    pub value: T,
}

impl<S: Clone + Into<T>, T> From<&Choice<S>>
    for ApplicationCommandOptionChoice<T>
{
    fn from(choice: &Choice<S>) -> Self {
        ApplicationCommandOptionChoice {
            name: choice.name.to_string(),
            value: choice.value.clone().into(),
        }
    }
}

pub struct CommandOptionRegistry<'a> {
    options: &'a [ApplicationCommandInteractionDataOption],
}

impl<'a> CommandOptionRegistry<'a> {
    pub fn new(options: &'a [ApplicationCommandInteractionDataOption]) -> Self {
        CommandOptionRegistry { options }
    }

    pub fn get_raw_option(
        &self,
        name: &str,
    ) -> Option<&ApplicationCommandInteractionDataOption> {
        self.options.iter().find(|option| option.name == name)
    }

    pub fn get_option<T: FromOption<'a>>(
        &self,
        name: &str,
    ) -> Result<T, GetOptionError> {
        self.options
            .iter()
            .find(|option| option.name == name)
            .ok_or(GetOptionError::MissingOption)
            .and_then(|option| {
                T::from_option(option)
                    .map_err(GetOptionError::InvalidOptionValue)
            })
    }
}

#[derive(Debug, Display, Error)]
#[non_exhaustive]
pub enum GetOptionError {
    #[display(fmt = "option not found")]
    MissingOption,
    #[display(fmt = "invalid option")]
    InvalidOptionValue(FromOptionError),
    #[display(fmt = "{}", _0)]
    Custom(#[error(ignore)] anyhow::Error),
}
