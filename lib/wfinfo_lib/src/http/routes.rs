use crate::{
    models::{
        ApplicationCommand, BatchEditGuildApplicationCommandPermissions,
        Channel, ClientCredentials, ClientCredentialsRequest,
        CreateApplicationCommand, CreateGuildApplicationCommandPermissions,
        CreateMessage as CreateMessageModel, CreateWebhookMessage,
        EditWebhookMessage, GuildApplicationCommandPermissions,
        InteractionResponse, Message, Snowflake,
    },
    request::RateLimitBucket,
    routes,
};
use reqwest::{Client, Method, RequestBuilder};
use std::{fmt::Display, sync::Arc};

/// A route within the Discord REST API.
///
/// The route's [`Display`] implementation determines the path in the Discord
/// REST API.
pub trait Route: Display + Send {
    /// The HTTP method used for requests to this route.
    fn method(&self) -> Method;

    /// The bucket associated with this route.
    fn bucket(&self) -> RateLimitBucket;

    /// Creates an HTTP request to this route.
    fn make_request(&self, client: &Client, base_url: &str) -> RequestBuilder;

    /// Whether this HTTP request must be authenticated.
    fn needs_auth(&self) -> bool;
}

routes! {
    (
        GetChannel { channel_id: Snowflake },
        method = GET "/channels/{channel_id}",
        major_params = [channel_id.to_u64(), 0],
        helper = get_channel -> Channel,
    ),
    (
        ModifyChannel { channel_id: Snowflake },
        method = PATCH "/channels/{channel_id}",
        major_params = [channel_id.to_u64(), 0],
        helper = modify_channel -> Channel,
    ),
    (
        DeleteChannel { channel_id: Snowflake },
        method = DELETE "/channels/{channel_id}",
        major_params = [channel_id.to_u64(), 0],
        helper = delete_channel -> Channel,
    ),
    (
        GetChannelMessages { channel_id: Snowflake },
        method = GET "/channels/{channel_id}/messages",
        major_params = [channel_id.to_u64(), 0],
        helper = get_channel_messages -> Vec<Message>,
    ),
    (
        GetChannelMessage { channel_id: Snowflake, message_id: Snowflake },
        method = GET "/channels/{channel_id}/messages/{message_id}",
        major_params = [channel_id.to_u64(), 0],
        helper = get_channel_message -> Message,
    ),
    (
        CreateMessage { channel_id: Snowflake },
        body = message: CreateMessageModel,
        method = POST "/channels/{channel_id}/messages",
        major_params = [channel_id.to_u64(), 0],
        helper = create_message -> Message,
    ),
    // Interactions
    (
        GetGlobalApplicationCommands { application_id: Snowflake },
        method = GET "/applications/{application_id}/commands",
        helper = get_global_application_commands -> Vec<ApplicationCommand>,
    ),
    (
        CreateGlobalApplicationCommand { application_id: Snowflake },
        body = command: CreateApplicationCommand,
        method = POST "/applications/{application_id}/commands",
        helper = create_global_application_command -> ApplicationCommand,
    ),
    (
        GetGlobalApplicationCommand { application_id: Snowflake, command_id: Snowflake },
        method = GET "/applications/{application_id}/commands/{command_id}",
        helper = get_global_application_command -> ApplicationCommand,
    ),
    (
        EditGlobalApplicationCommand { application_id: Snowflake, command_id: Snowflake },
        body = command: CreateApplicationCommand,
        method = PATCH "/applications/{application_id}/commands/{command_id}",
        helper = edit_global_application_command -> ApplicationCommand,
    ),
    (
        DeleteGlobalApplicationCommand { application_id: Snowflake, command_id: Snowflake },
        method = DELETE "/applications/{application_id}/commands/{command_id}",
        helper = delete_global_application_command -> ApplicationCommand,
    ),
    (
        GetGuildApplicationCommands { application_id: Snowflake, guild_id: Snowflake },
        method = GET "/applications/{application_id}/guilds/{guild_id}/commands",
        helper = get_guild_application_commands -> Vec<ApplicationCommand>,
    ),
    (
        BulkOverwriteGlobalApplicationCommands { application_id: Snowflake },
        body = commands: Vec<CreateApplicationCommand>,
        method = PUT "/application/{application_id}/commands",
        helper = bulk_overwrite_global_application_commands -> Vec<ApplicationCommand>,
    ),
    (
        CreateGuildApplicationCommand { application_id: Snowflake, guild_id: Snowflake },
        body = command: CreateApplicationCommand,
        method = POST "/applications/{application_id}/guilds/{guild_id}/commands",
        helper = create_guild_application_command -> ApplicationCommand,
    ),
    (
        GetGuildApplicationCommand { application_id: Snowflake, guild_id: Snowflake, command_id: Snowflake },
        method = GET "/applications/{application_id}/guilds/{guild_id}/commands/{command_id}",
        helper = get_guild_application_command -> ApplicationCommand,
    ),
    (
        EditGuildApplicationCommand { application_id: Snowflake, guild_id: Snowflake, command_id: Snowflake },
        body = command: CreateApplicationCommand,
        method = PATCH "/applications/{application_id}/guilds/{guild_id}/commands/{command_id}",
        helper = edit_guild_application_command -> ApplicationCommand,
    ),
    (
        DeleteGuildApplicationCommand { application_id: Snowflake, guild_id: Snowflake, command_id: Snowflake },
        method = DELETE "/applications/{application_id}/guilds/{guild_id}/commands/{command_id}",
        helper = delete_guild_application_command -> ApplicationCommand,
    ),
    (
        BulkOverwriteGuildApplicationCommands { application_id: Snowflake, guild_id: Snowflake },
        body = commands: Vec<CreateApplicationCommand>,
        method = PUT "/applications/{application_id}/guilds/{guild_id}/commands",
        helper = bulk_overwrite_guild_application_commands -> Vec<ApplicationCommand>,
    ),
    (
        CreateInteractionResponse { interaction_id: Snowflake, interaction_token: String },
        body = response: InteractionResponse,
        method = POST "/interactions/{interaction_id}/{interaction_token}/callback",
        helper = create_interaction_response -> Message,
    ),
    (
        GetOriginalInteractionResponse { application_id: Snowflake, interaction_token: String },
        method = GET "/webhooks/{application_id}/{interaction_token}/messages/@original",
        helper = get_original_interaction_response -> Message,
    ),
    (
        EditOriginalInteractionResponse { application_id: Snowflake, interaction_token: String },
        body = message: EditWebhookMessage,
        method = PATCH "/webhooks/{application_id}/{interaction_token}/messages/@original",
        helper = edit_original_interaction_response -> Message,
    ),
    (
        DeleteOriginalInteractionResponse { application_id: Snowflake, interaction_token: String },
        method = DELETE "/webhooks/{application_id}/{interaction_token}/messages/@original",
        helper = delete_original_interaction_response -> (),
    ),
    (
        CreateFollowupMessage { application_id: Snowflake, interaction_token: String },
        body = message: CreateWebhookMessage,
        method = POST "/webhooks/{application_id}/{interaction_token}",
        helper = create_followup_message -> Message,
    ),
    (
        GetFollowupMessage { application_id: Snowflake, interaction_token: String, message_id: Snowflake },
        method = GET "/webhooks/{application_id}/{interaction_token}/messages/{message_id}",
        helper = get_followup_message -> Message,
    ),
    (
        EditFollowupMessage { application_id: Snowflake, interaction_token: String, message_id: Snowflake },
        method = PATCH "/webhooks/{application_id}/{interaction_token}/messages/{message_id}",
        helper = edit_followup_message -> Message,
    ),
    (
        DeleteFollowupMessage { application_id: Snowflake, interaction_token: String, message_id: Snowflake },
        method = DELETE "/webhooks/{application_id}/{interaction_token}/messages/{message_id}",
        helper = delete_followup_message -> (),
    ),
    (
        GetGuildApplicationCommandPermissions { application_id: Snowflake, guild_id: Snowflake },
        method = GET "/applications/{application_id}/guilds/{guild_id}/commands/permissions",
        helper = get_guild_application_command_permissions -> GuildApplicationCommandPermissions,
    ),
    (
        GetApplicationCommandPermissions { application_id: Snowflake, guild_id: Snowflake, command_id: Snowflake },
        method = GET "/applications/{application_id}/guilds/{guild_id}/commands/{command_id}/permissions",
        helper = get_application_command_permissions -> GuildApplicationCommandPermissions,
    ),
    (
        EditApplicationCommandPermissions { application_id: Snowflake, guild_id: Snowflake, command_id: Snowflake },
        body = permissions: CreateGuildApplicationCommandPermissions,
        method = PUT "/applications/{application_id}/guilds/{guild_id}/commands/{command_id}/permissions",
        helper = edit_application_command_permissions -> GuildApplicationCommandPermissions,
    ),
    (
        BatchEditApplicationCommandPermissions { application_id: Snowflake, guild_id: Snowflake },
        body = permissions: Vec<BatchEditGuildApplicationCommandPermissions>,
        method = PUT "/applications/{application_id}/guilds/{guild_id}/commands/permissions",
        helper = batch_edit_application_command_permissions -> Vec<GuildApplicationCommandPermissions>,
    ),
    // OAuth2
    (
        AuthenticateClientCredentialsGrant { },
        body = [form] scope: ClientCredentialsRequest,
        extra = { client_id: Snowflake, client_secret: Arc<String> },
        method = POST "/oauth2/token",
        processor = |req| req.basic_auth(client_id, Some(client_secret)),
        needs_auth = false,
        helper = authenticate_client_credentials_grant -> ClientCredentials,
    ),
}
