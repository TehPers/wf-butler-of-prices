use crate::interactions::ApplicationCommandHandler;
use anyhow::bail;
use runtime_injector::Svc;
use wfbp_discord::{
    models::{
        ApplicationId, CreateWebhookMessage, Interaction, InteractionType,
        Message,
    },
    DiscordRestClient,
};
use wfbp_http::RequestError;

/// Handler for Discord interactions.
#[derive(Debug)]
pub struct InteractionHandler {
    application_command_handler: Svc<ApplicationCommandHandler>,
}

impl InteractionHandler {
    pub fn new(
        application_command_handler: Svc<ApplicationCommandHandler>,
    ) -> Self {
        Self {
            application_command_handler,
        }
    }

    pub async fn handle(
        &self,
        client: DiscordRestClient,
        interaction: Interaction,
    ) -> anyhow::Result<()> {
        let Interaction {
            id: interaction_id,
            application_id,
            kind: interaction_type,
            guild_id,
            channel_id,
            member,
            user,
            token: interaction_token,
            version: _,
        } = interaction;
        let responder = InteractionResponder {
            client,
            app_id: interaction.application_id,
            interaction_token,
        };
        match interaction_type {
            InteractionType::Ping => Ok(()),
            InteractionType::ApplicationCommand { data, locale } => {
                self.application_command_handler
                    .handle(responder, guild_id, channel_id, member, user, data)
                    .await
            }
            InteractionType::MessageComponent {
                message: _message,
                component_type: _component_type,
                locale,
            } => bail!("TODO"),
            InteractionType::Autocomplete { locale } => bail!("TODO"),
            InteractionType::ModalSubmit {
                message: _message,
                locale,
            } => bail!("TODO"),
        }
    }

    pub async fn register_all(&self) -> anyhow::Result<()> {
        self.application_command_handler.register_all().await
    }
}

#[derive(Clone, Debug)]
pub struct InteractionResponder {
    pub client: DiscordRestClient,
    pub app_id: ApplicationId,
    pub interaction_token: String,
}

impl InteractionResponder {
    /// The Discord REST API client.
    pub fn client(&self) -> &DiscordRestClient {
        &self.client
    }

    /// The application ID.
    pub fn app_id(&self) -> ApplicationId {
        self.app_id
    }

    /// The interaction token.
    pub fn interaction_token(&self) -> &str {
        &self.interaction_token
    }

    /// Creates a response message to the command.
    pub async fn respond(
        &self,
        message: CreateWebhookMessage,
    ) -> Result<Message, RequestError> {
        wfbp_discord::routes::CreateFollowupMessage::execute(
            &self.client,
            self.app_id,
            self.interaction_token.clone(),
            message,
        )
        .await
    }
}

#[derive(Clone, Debug)]
pub struct CommandData {}
