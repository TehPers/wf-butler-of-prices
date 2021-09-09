use crate::{
    models::{
        ApplicationCommand, BatchEditGuildApplicationCommandPermissions,
        Channel, ClientCredentials, ClientCredentialsRequest,
        CreateApplicationCommand, CreateGuildApplicationCommandPermissions,
        CreateMessage as CreateMessageModel, CreateWebhookMessage,
        EditWebhookMessage, GuildApplicationCommandPermissions,
        InteractionResponse, Message, Snowflake,
    },
    rate_limit::RateLimitBucket,
};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
};
use wfinfo_lib::{reqwest::Method, routes};

#[derive(Clone, Debug)]
pub struct DiscordRouteInfo {
    pub needs_auth: bool,
    pub bucket: RateLimitBucket,
}

impl DiscordRouteInfo {
    pub fn without_auth(
        method: Method,
        route: &'static str,
        major_params: [u64; 2],
    ) -> Self {
        DiscordRouteInfo {
            needs_auth: false,
            bucket: RateLimitBucket::new(method, route, major_params),
        }
    }

    pub fn with_auth(
        method: Method,
        route: &'static str,
        major_params: [u64; 2],
    ) -> Self {
        DiscordRouteInfo {
            needs_auth: true,
            bucket: RateLimitBucket::new(method, route, major_params),
        }
    }
}

fn hash_str(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

routes! {
    (
        GetChannel { channel_id: Snowflake },
        method = GET "/channels/{channel_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [channel_id.to_u64(), 0]
            )
        },
        helper = Channel,
    ),
    (
        ModifyChannel { channel_id: Snowflake },
        method = PATCH "/channels/{channel_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [channel_id.to_u64(), 0],
            )
        },
        helper = Channel,
    ),
    (
        DeleteChannel { channel_id: Snowflake },
        method = DELETE "/channels/{channel_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [channel_id.to_u64(), 0],
            )
        },
        helper = Channel,
    ),
    (
        GetChannelMessages { channel_id: Snowflake },
        method = GET "/channels/{channel_id}/messages",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [channel_id.to_u64(), 0],
            )
        },
        helper = Vec<Message>,
    ),
    (
        GetChannelMessage { channel_id: Snowflake, message_id: Snowflake },
        method = GET "/channels/{channel_id}/messages/{message_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [channel_id.to_u64(), 0],
            )
        },
        helper = Message,
    ),
    (
        CreateMessage { channel_id: Snowflake },
        body = message: CreateMessageModel,
        method = POST "/channels/{channel_id}/messages",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [channel_id.to_u64(), 0],
            )
        },
        helper = Message,
    ),
    // Interactions
    (
        GetGlobalApplicationCommands { application_id: Snowflake },
        method = GET "/applications/{application_id}/commands",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        helper = Vec<ApplicationCommand>,
    ),
    (
        CreateGlobalApplicationCommand { application_id: Snowflake },
        body = command: CreateApplicationCommand,
        method = POST "/applications/{application_id}/commands",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        helper = ApplicationCommand,
    ),
    (
        GetGlobalApplicationCommand { application_id: Snowflake, command_id: Snowflake },
        method = GET "/applications/{application_id}/commands/{command_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        helper = ApplicationCommand,
    ),
    (
        EditGlobalApplicationCommand { application_id: Snowflake, command_id: Snowflake },
        body = command: CreateApplicationCommand,
        method = PATCH "/applications/{application_id}/commands/{command_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        helper = ApplicationCommand,
    ),
    (
        DeleteGlobalApplicationCommand { application_id: Snowflake, command_id: Snowflake },
        method = DELETE "/applications/{application_id}/commands/{command_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        helper = ApplicationCommand,
    ),
    (
        GetGuildApplicationCommands { application_id: Snowflake, guild_id: Snowflake },
        method = GET "/applications/{application_id}/guilds/{guild_id}/commands",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        helper = Vec<ApplicationCommand>,
    ),
    (
        BulkOverwriteGlobalApplicationCommands { application_id: Snowflake },
        body = commands: Vec<CreateApplicationCommand>,
        method = PUT "/applications/{application_id}/commands",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        helper = Vec<ApplicationCommand>,
    ),
    (
        CreateGuildApplicationCommand { application_id: Snowflake, guild_id: Snowflake },
        body = command: CreateApplicationCommand,
        method = POST "/applications/{application_id}/guilds/{guild_id}/commands",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        helper = ApplicationCommand,
    ),
    (
        GetGuildApplicationCommand { application_id: Snowflake, guild_id: Snowflake, command_id: Snowflake },
        method = GET "/applications/{application_id}/guilds/{guild_id}/commands/{command_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        helper = ApplicationCommand,
    ),
    (
        EditGuildApplicationCommand { application_id: Snowflake, guild_id: Snowflake, command_id: Snowflake },
        body = command: CreateApplicationCommand,
        method = PATCH "/applications/{application_id}/guilds/{guild_id}/commands/{command_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        helper = ApplicationCommand,
    ),
    (
        DeleteGuildApplicationCommand { application_id: Snowflake, guild_id: Snowflake, command_id: Snowflake },
        method = DELETE "/applications/{application_id}/guilds/{guild_id}/commands/{command_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        helper = ApplicationCommand,
    ),
    (
        BulkOverwriteGuildApplicationCommands { application_id: Snowflake, guild_id: Snowflake },
        body = commands: Vec<CreateApplicationCommand>,
        method = PUT "/applications/{application_id}/guilds/{guild_id}/commands",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        helper = Vec<ApplicationCommand>,
    ),
    (
        CreateInteractionResponse { interaction_id: Snowflake, interaction_token: String },
        body = response: InteractionResponse,
        method = POST "/interactions/{interaction_id}/{interaction_token}/callback",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        helper = Message,
    ),
    (
        GetOriginalInteractionResponse { application_id: Snowflake, interaction_token: String },
        method = GET "/webhooks/{application_id}/{interaction_token}/messages/@original",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [application_id.to_u64(), hash_str(interaction_token)],
            )
        },
        helper = Message,
    ),
    (
        EditOriginalInteractionResponse { application_id: Snowflake, interaction_token: String },
        body = message: EditWebhookMessage,
        method = PATCH "/webhooks/{application_id}/{interaction_token}/messages/@original",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [application_id.to_u64(), hash_str(interaction_token)],
            )
        },
        helper = Message,
    ),
    (
        DeleteOriginalInteractionResponse { application_id: Snowflake, interaction_token: String },
        method = DELETE "/webhooks/{application_id}/{interaction_token}/messages/@original",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [application_id.to_u64(), hash_str(interaction_token)],
            )
        },
        helper = (),
    ),
    (
        CreateFollowupMessage { application_id: Snowflake, interaction_token: String },
        body = message: CreateWebhookMessage,
        method = POST "/webhooks/{application_id}/{interaction_token}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [application_id.to_u64(), hash_str(interaction_token)],
            )
        },
        helper = Message,
    ),
    (
        GetFollowupMessage { application_id: Snowflake, interaction_token: String, message_id: Snowflake },
        method = GET "/webhooks/{application_id}/{interaction_token}/messages/{message_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [application_id.to_u64(), hash_str(interaction_token)],
            )
        },
        helper = Message,
    ),
    (
        EditFollowupMessage { application_id: Snowflake, interaction_token: String, message_id: Snowflake },
        method = PATCH "/webhooks/{application_id}/{interaction_token}/messages/{message_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [application_id.to_u64(), hash_str(interaction_token)],
            )
        },
        helper = Message,
    ),
    (
        DeleteFollowupMessage { application_id: Snowflake, interaction_token: String, message_id: Snowflake },
        method = DELETE "/webhooks/{application_id}/{interaction_token}/messages/{message_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [application_id.to_u64(), hash_str(interaction_token)],
            )
        },
        helper = (),
    ),
    (
        GetGuildApplicationCommandPermissions { application_id: Snowflake, guild_id: Snowflake },
        method = GET "/applications/{application_id}/guilds/{guild_id}/commands/permissions",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        helper = GuildApplicationCommandPermissions,
    ),
    (
        GetApplicationCommandPermissions { application_id: Snowflake, guild_id: Snowflake, command_id: Snowflake },
        method = GET "/applications/{application_id}/guilds/{guild_id}/commands/{command_id}/permissions",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        helper = GuildApplicationCommandPermissions,
    ),
    (
        EditApplicationCommandPermissions { application_id: Snowflake, guild_id: Snowflake, command_id: Snowflake },
        body = permissions: CreateGuildApplicationCommandPermissions,
        method = PUT "/applications/{application_id}/guilds/{guild_id}/commands/{command_id}/permissions",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        helper = GuildApplicationCommandPermissions,
    ),
    (
        BatchEditApplicationCommandPermissions { application_id: Snowflake, guild_id: Snowflake },
        body = permissions: Vec<BatchEditGuildApplicationCommandPermissions>,
        method = PUT "/applications/{application_id}/guilds/{guild_id}/commands/permissions",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        helper = Vec<GuildApplicationCommandPermissions>,
    ),
    // OAuth2
    (
        AuthenticateClientCredentialsGrant { },
        body = [form] scope: ClientCredentialsRequest,
        extra = { client_id: Snowflake, client_secret: Arc<String> },
        method = POST "/oauth2/token",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::without_auth(
                method,
                route,
                [0, 0],
            )
        },
        processor = |req| req.basic_auth(client_id, Some(client_secret)),
        helper = ClientCredentials,
    ),
}
