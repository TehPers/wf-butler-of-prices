use crate::{
    middleware::ClientSecret,
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
use reqwest::Method;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
};
use wfinfo_http::routes;

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
        GetChannel {
            channel_id: Snowflake,
        },
        method = GET "/channels/{channel_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [channel_id.to_u64(), 0]
            )
        },
        response = [json] Channel,
    ),
    (
        ModifyChannel {
            channel_id: Snowflake,
        },
        method = PATCH "/channels/{channel_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [channel_id.to_u64(), 0],
            )
        },
        response = [json] Channel,
    ),
    (
        DeleteChannel {
            channel_id: Snowflake,
        },
        method = DELETE "/channels/{channel_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [channel_id.to_u64(), 0],
            )
        },
        response = [json] Channel,
    ),
    (
        GetChannelMessages {
            channel_id: Snowflake,
        },
        method = GET "/channels/{channel_id}/messages",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [channel_id.to_u64(), 0],
            )
        },
        response = [json] Vec<Message>,
    ),
    (
        GetChannelMessage {
            channel_id: Snowflake,
            message_id: Snowflake,
        },
        method = GET "/channels/{channel_id}/messages/{message_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [channel_id.to_u64(), 0],
            )
        },
        response = [json] Message,
    ),
    (
        CreateMessage {
            channel_id: Snowflake,
            message: CreateMessageModel,
        },
        body = [json] message,
        method = POST "/channels/{channel_id}/messages",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [channel_id.to_u64(), 0],
            )
        },
        response = [json] Message,
    ),
    // Interactions
    (
        GetGlobalApplicationCommands {
            application_id: Snowflake,
        },
        method = GET "/applications/{application_id}/commands",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        response = [json] Vec<ApplicationCommand>,
    ),
    (
        CreateGlobalApplicationCommand {
            application_id: Snowflake,
            command: CreateApplicationCommand,
        },
        body = [json] command,
        method = POST "/applications/{application_id}/commands",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        response = [json] ApplicationCommand,
    ),
    (
        GetGlobalApplicationCommand {
            application_id: Snowflake,
            command_id: Snowflake,
        },
        method = GET "/applications/{application_id}/commands/{command_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        response = [json] ApplicationCommand,
    ),
    (
        EditGlobalApplicationCommand {
            application_id: Snowflake,
            command_id: Snowflake,
            command: CreateApplicationCommand,
        },
        body = [json] command,
        method = PATCH "/applications/{application_id}/commands/{command_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        response = [json] ApplicationCommand,
    ),
    (
        DeleteGlobalApplicationCommand {
            application_id: Snowflake,
            command_id: Snowflake,
        },
        method = DELETE "/applications/{application_id}/commands/{command_id}",
        info =  |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        response = [json] ApplicationCommand,
    ),
    (
        GetGuildApplicationCommands {
            application_id: Snowflake,
            guild_id: Snowflake,
        },
        method = GET "/applications/{application_id}/guilds/{guild_id}/commands",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        response = [json] Vec<ApplicationCommand>,
    ),
    (
        BulkOverwriteGlobalApplicationCommands {
            application_id: Snowflake,
            commands: Vec<CreateApplicationCommand>,
        },
        body = [json] commands,
        method = PUT "/applications/{application_id}/commands",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        response = [json] Vec<ApplicationCommand>,
    ),
    (
        CreateGuildApplicationCommand {
            application_id: Snowflake,
            guild_id: Snowflake,
            command: CreateApplicationCommand,
        },
        body = [json] command,
        method = POST "/applications/{application_id}/guilds/{guild_id}/commands",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        response = [json] ApplicationCommand,
    ),
    (
        GetGuildApplicationCommand {
            application_id: Snowflake,
            guild_id: Snowflake,
            command_id: Snowflake
        },
        method = GET "/applications/{application_id}/guilds/{guild_id}/commands/{command_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        response = [json] ApplicationCommand,
    ),
    (
        EditGuildApplicationCommand {
            application_id: Snowflake,
            guild_id: Snowflake,
            command_id: Snowflake,
            command: CreateApplicationCommand
        },
        body = [json] command,
        method = PATCH "/applications/{application_id}/guilds/{guild_id}/commands/{command_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        response = [json] ApplicationCommand,
    ),
    (
        DeleteGuildApplicationCommand {
            application_id: Snowflake,
            guild_id: Snowflake,
            command_id: Snowflake,
        },
        method = DELETE "/applications/{application_id}/guilds/{guild_id}/commands/{command_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        response = [json] ApplicationCommand,
    ),
    (
        BulkOverwriteGuildApplicationCommands {
            application_id: Snowflake,
            guild_id: Snowflake,
            commands: Vec<CreateApplicationCommand>,
        },
        body = [json] commands,
        method = PUT "/applications/{application_id}/guilds/{guild_id}/commands",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        response = [json] Vec<ApplicationCommand>,
    ),
    (
        CreateInteractionResponse {
            interaction_id: Snowflake,
            interaction_token: String,
            response: InteractionResponse,
        },
        body = [json] response,
        method = POST "/interactions/{interaction_id}/{interaction_token}/callback",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        response = [json] Message,
    ),
    (
        GetOriginalInteractionResponse {
            application_id: Snowflake,
            interaction_token: String,
        },
        method = GET "/webhooks/{application_id}/{interaction_token}/messages/@original",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [application_id.to_u64(), hash_str(interaction_token)],
            )
        },
        response = [json] Message,
    ),
    (
        EditOriginalInteractionResponse {
            application_id: Snowflake,
            interaction_token: String,
            message: EditWebhookMessage,
        },
        body = [json] message,
        method = PATCH "/webhooks/{application_id}/{interaction_token}/messages/@original",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [application_id.to_u64(), hash_str(interaction_token)],
            )
        },
        response = [json] Message,
    ),
    (
        DeleteOriginalInteractionResponse {
            application_id: Snowflake,
            interaction_token: String,
        },
        method = DELETE "/webhooks/{application_id}/{interaction_token}/messages/@original",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [application_id.to_u64(), hash_str(interaction_token)],
            )
        },
        response = [json] (),
    ),
    (
        CreateFollowupMessage {
            application_id: Snowflake,
            interaction_token: String,
            message: CreateWebhookMessage,
        },
        body = [json] message,
        method = POST "/webhooks/{application_id}/{interaction_token}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [application_id.to_u64(), hash_str(interaction_token)],
            )
        },
        response = [json] Message,
    ),
    (
        GetFollowupMessage {
            application_id: Snowflake,
            interaction_token: String,
            message_id: Snowflake,
        },
        method = GET "/webhooks/{application_id}/{interaction_token}/messages/{message_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [application_id.to_u64(), hash_str(interaction_token)],
            )
        },
        response = [json] Message,
    ),
    (
        EditFollowupMessage {
            application_id: Snowflake,
            interaction_token: String,
            message_id: Snowflake,
        },
        method = PATCH "/webhooks/{application_id}/{interaction_token}/messages/{message_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [application_id.to_u64(), hash_str(interaction_token)],
            )
        },
        response = [json] Message,
    ),
    (
        DeleteFollowupMessage {
            application_id: Snowflake,
            interaction_token: String,
            message_id: Snowflake,
        },
        method = DELETE "/webhooks/{application_id}/{interaction_token}/messages/{message_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [application_id.to_u64(), hash_str(interaction_token)],
            )
        },
        response = [json] (),
    ),
    (
        GetGuildApplicationCommandPermissions {
            application_id: Snowflake,
            guild_id: Snowflake,
        },
        method = GET "/applications/{application_id}/guilds/{guild_id}/commands/permissions",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        response = [json] GuildApplicationCommandPermissions,
    ),
    (
        GetApplicationCommandPermissions {
            application_id: Snowflake,
            guild_id: Snowflake,
            command_id: Snowflake,
        },
        method = GET "/applications/{application_id}/guilds/{guild_id}/commands/{command_id}/permissions",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        response = [json] GuildApplicationCommandPermissions,
    ),
    (
        EditApplicationCommandPermissions {
            application_id: Snowflake,
            guild_id: Snowflake,
            command_id: Snowflake,
            permissions: CreateGuildApplicationCommandPermissions,
        },
        body = [json] permissions,
        method = PUT "/applications/{application_id}/guilds/{guild_id}/commands/{command_id}/permissions",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        response = [json] GuildApplicationCommandPermissions,
    ),
    (
        BatchEditApplicationCommandPermissions {
            application_id: Snowflake,
            guild_id: Snowflake,
            permissions: Vec<BatchEditGuildApplicationCommandPermissions>,
        },
        body = [json] permissions,
        method = PUT "/applications/{application_id}/guilds/{guild_id}/commands/permissions",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [0, 0],
            )
        },
        response = [json] Vec<GuildApplicationCommandPermissions>,
    ),
    // OAuth2
    (
        AuthenticateClientCredentialsGrant {
            scope: ClientCredentialsRequest,
            client_id: Snowflake,
            client_secret: Arc<ClientSecret>,
        },
        body = [form] scope,
        method = POST "/oauth2/token",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::without_auth(
                method,
                route,
                [0, 0],
            )
        },
        processor = |req| req.basic_auth(client_id, Some(&***client_secret)),
        response = [json] ClientCredentials,
    ),
}
