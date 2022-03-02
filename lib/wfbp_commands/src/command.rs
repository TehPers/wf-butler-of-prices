use crate::{FromOption, FromOptionError};
use async_recursion::async_recursion;
use async_trait::async_trait;
use derive_more::{Display, Error};
use std::{
    borrow::Cow,
    fmt::{Debug, Formatter},
    sync::Arc,
};
use wfbp_discord::{
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
use wfbp_http::RequestError;

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
            execute_callback(
                interaction_data.clone(),
                &root_data,
                &root_data.options,
                callback.as_ref(),
            )
            .await?;
        }

        // Options
        handle_options(
            interaction_data.clone(),
            &root_data,
            &self.options,
            root_data.options.iter(),
        )
        .await?;

        Ok(())
    }
}

async fn execute_callback<C: ?Sized + CommandCallback>(
    interaction_data: Arc<InteractionData>,
    root_data: &SlashCommandData,
    option_data: &[ApplicationCommandInteractionDataOption],
    callback: &C,
) -> Result<(), HandleInteractionError> {
    let option_registry = CommandOptionRegistry::new(option_data);
    callback
        .invoke(interaction_data, &root_data, option_registry)
        .await?;
    Ok(())
}

async fn handle_options(
    interaction_data: Arc<InteractionData>,
    root_data: &SlashCommandData,
    options: &[CommandOption],
    option_data: impl IntoIterator<Item = &ApplicationCommandInteractionDataOption>,
) -> Result<(), HandleInteractionError> {
    for option_data in option_data.into_iter() {
        options
            .iter()
            .find(|option| option.name == option_data.name)
            .ok_or_else(|| {
                HandleInteractionError::UnknownOption(option_data.name.clone())
            })?
            .handle(interaction_data.clone(), &root_data, &option_data)
            .await?;
    }

    Ok(())
}

impl Debug for SlashCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SlashCommand")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("options", &self.options)
            .field("default_permission", &self.default_permission)
            .finish_non_exhaustive()
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
    #[display(fmt = "missing input options from interaction data")]
    MissingOptions,
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
        invoke_data: &ApplicationCommandInteractionDataOption,
    ) -> Result<(), HandleInteractionError> {
        match &self.kind {
            CommandOptionType::SubCommand {
                options,
                callback: Some(ref callback),
            } => {
                // Options
                let option_data =
                    match &invoke_data.kind {
                        ApplicationCommandInteractionDataOptionType::SubCommand {
                            options,
                        } => options
                            .as_ref()
                            .map(Vec::as_slice)
                            .unwrap_or(&[]),
                    _ => return Err(HandleInteractionError::InvalidData(
                        self.name.to_string(),
                    ))
                };
                handle_options(
                    interaction_data.clone(),
                    &root_data,
                    options,
                    option_data,
                )
                .await?;

                // Callback
                execute_callback(
                    interaction_data.clone(),
                    root_data,
                    option_data,
                    callback.as_ref(),
                )
                .await?;

                Ok(())
            }
            CommandOptionType::SubCommandGroup { options } => {
                let option_data = match &invoke_data.kind {
                    ApplicationCommandInteractionDataOptionType::SubCommandGroup {
                        options
                    } => options
                        .as_ref()
                        .map(Vec::as_slice)
                        .unwrap_or(&[]),
                    _ => return Err(HandleInteractionError::InvalidData(
                        self.name.to_string(),
                    ))
                };

                // Options
                handle_options(
                    interaction_data.clone(),
                    &root_data,
                    options,
                    option_data,
                )
                .await?;

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
        self.get_optional_option(name).and_then(|inner| {
            inner.ok_or_else(|| GetOptionError::MissingOption(name.to_string()))
        })
    }

    pub fn get_optional_option<T: FromOption<'a>>(
        &self,
        name: &str,
    ) -> Result<Option<T>, GetOptionError> {
        self.options
            .iter()
            .find(|option| option.name == name)
            .map(|option| {
                T::from_option(option)
                    .map_err(GetOptionError::InvalidOptionValue)
            })
            .transpose()
    }
}

#[derive(Debug, Display, Error)]
#[non_exhaustive]
pub enum GetOptionError {
    #[display(fmt = "option not found: '{}'", _0)]
    MissingOption(#[error(ignore)] String),
    #[display(fmt = "invalid option")]
    InvalidOptionValue(FromOptionError),
    #[display(fmt = "{}", _0)]
    Custom(#[error(ignore)] anyhow::Error),
}
