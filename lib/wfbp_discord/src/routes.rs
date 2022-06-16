use crate::{
    middleware::ClientSecret,
    models::{
        ApplicationCommand, ApplicationCommandId, ApplicationId,
        BatchEditGuildApplicationCommandPermissions, Channel, ChannelId,
        ClientCredentials, ClientCredentialsRequest, CreateApplicationCommand,
        CreateGuildApplicationCommandPermissions,
        CreateMessage as CreateMessageModel, CreateWebhookMessage,
        EditWebhookMessage, Guild, GuildApplicationCommandPermissions, GuildId,
        GuildMember, GuildPreview, InteractionId, InteractionResponse, Message,
        MessageId, Role, User, UserId,
    },
    RateLimitBucket,
};
use reqwest::Method;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
};
use wfbp_http::routes;

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
    // Interactions
    (
        GetGlobalApplicationCommands {
            application_id: ApplicationId,
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
            application_id: ApplicationId,
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
            application_id: ApplicationId,
            command_id: ApplicationCommandId,
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
            application_id: ApplicationId,
            command_id: ApplicationCommandId,
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
            application_id: ApplicationId,
            command_id: ApplicationCommandId,
        },
        method = DELETE "/applications/{application_id}/commands/{command_id}",
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
        GetGuildApplicationCommands {
            application_id: ApplicationId,
            guild_id: GuildId,
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
            application_id: ApplicationId,
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
            application_id: ApplicationId,
            guild_id: GuildId,
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
            application_id: ApplicationId,
            guild_id: GuildId,
            command_id: ApplicationCommandId,
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
            application_id: ApplicationId,
            guild_id: GuildId,
            command_id: ApplicationCommandId,
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
            application_id: ApplicationId,
            guild_id: GuildId,
            command_id: ApplicationCommandId,
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
            application_id: ApplicationId,
            guild_id: GuildId,
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
            interaction_id: InteractionId,
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
            application_id: ApplicationId,
            interaction_token: String,
        },
        method = GET "/webhooks/{application_id}/{interaction_token}/messages/@original",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [application_id.into_inner().to_u64(), hash_str(interaction_token)],
            )
        },
        response = [json] Message,
    ),
    (
        EditOriginalInteractionResponse {
            application_id: ApplicationId,
            interaction_token: String,
            message: EditWebhookMessage,
        },
        body = [json] message,
        method = PATCH "/webhooks/{application_id}/{interaction_token}/messages/@original",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [application_id.into_inner().to_u64(), hash_str(interaction_token)],
            )
        },
        response = [json] Message,
    ),
    (
        DeleteOriginalInteractionResponse {
            application_id: ApplicationId,
            interaction_token: String,
        },
        method = DELETE "/webhooks/{application_id}/{interaction_token}/messages/@original",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [application_id.into_inner().to_u64(), hash_str(interaction_token)],
            )
        },
        response = [empty] (),
    ),
    (
        CreateFollowupMessage {
            application_id: ApplicationId,
            interaction_token: String,
            message: CreateWebhookMessage,
        },
        body = [json] message,
        method = POST "/webhooks/{application_id}/{interaction_token}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [application_id.into_inner().to_u64(), hash_str(interaction_token)],
            )
        },
        response = [json] Message,
    ),
    (
        GetFollowupMessage {
            application_id: ApplicationId,
            interaction_token: String,
            message_id: MessageId,
        },
        method = GET "/webhooks/{application_id}/{interaction_token}/messages/{message_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [application_id.into_inner().to_u64(), hash_str(interaction_token)],
            )
        },
        response = [json] Message,
    ),
    (
        EditFollowupMessage {
            application_id: ApplicationId,
            interaction_token: String,
            message_id: MessageId,
        },
        method = PATCH "/webhooks/{application_id}/{interaction_token}/messages/{message_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [application_id.into_inner().to_u64(), hash_str(interaction_token)],
            )
        },
        response = [json] Message,
    ),
    (
        DeleteFollowupMessage {
            application_id: ApplicationId,
            interaction_token: String,
            message_id: MessageId,
        },
        method = DELETE "/webhooks/{application_id}/{interaction_token}/messages/{message_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [application_id.into_inner().to_u64(), hash_str(interaction_token)],
            )
        },
        response = [empty] (),
    ),
    (
        GetGuildApplicationCommandPermissions {
            application_id: ApplicationId,
            guild_id: GuildId,
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
            application_id: ApplicationId,
            guild_id: GuildId,
            command_id: ApplicationCommandId,
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
            application_id: ApplicationId,
            guild_id: GuildId,
            command_id: ApplicationCommandId,
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
            application_id: ApplicationId,
            guild_id: GuildId,
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
    // Users
    (
        GetUser {
            user_id: UserId,
        },
        method = GET "/users/{user_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [user_id.into_inner().to_u64(), 0],
            )
        },
        response = [json] User,
    ),
    // Guilds
    (
        GetGuild {
            guild_id: GuildId,
        },
        method = GET "/guilds/{guild_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [guild_id.into_inner().to_u64(), 0],
            )
        },
        response = [json] Guild,
    ),
    (
        GetGuildPreview {
            guild_id: GuildId,
        },
        method = GET "/guilds/{guild_id}/preview",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [guild_id.into_inner().to_u64(), 0],
            )
        },
        response = [json] GuildPreview,
    ),
    // TODO: ModifyGuild
    (
        DeleteGuild {
            guild_id: GuildId,
        },
        method = DELETE "/guilds/{guild_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [guild_id.into_inner().to_u64(), 0],
            )
        },
        response = [empty] (),
    ),
    (
        GetGuildChannels {
            guild_id: GuildId,
        },
        method = GET "/guild/{guild_id}/channels",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [guild_id.into_inner().to_u64(), 0],
            )
        },
        response = [json] Vec<Channel>,
    ),
    // TODO: CreateGuildChannel
    // TODO: ModifyGuildChannel
    // TODO: ListActiveThreads
    (
        GetGuildMember {
            guild_id: GuildId,
            user_id: UserId,
        },
        method = GET "/guilds/{guild_id}/members/{user_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [guild_id.into_inner().to_u64(), user_id.into_inner().to_u64()],
            )
        },
        response = [json] GuildMember,
    ),
    // TODO: more guild methods
    (
        /// Returns a list of [Role] objects for the guild.
        ///
        /// [Official docs](https://discord.com/developers/docs/resources/guild#get-guild-roles)
        GetGuildRoles {
            /// The ID of the guild.
            guild_id: GuildId,
        },
        method = GET "/guilds/{guild_id}/roles",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [guild_id.into_inner().to_u64(), 0],
            )
        },
        response = [json] Vec<Role>,
    ),
    // TODO: more guild methods
    // Channels
    (
        GetChannel {
            channel_id: ChannelId,
        },
        method = GET "/channels/{channel_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [channel_id.into_inner().to_u64(), 0]
            )
        },
        response = [json] Channel,
    ),
    (
        ModifyChannel {
            channel_id: ChannelId,
        },
        method = PATCH "/channels/{channel_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [channel_id.into_inner().to_u64(), 0],
            )
        },
        response = [json] Channel,
    ),
    (
        DeleteChannel {
            channel_id: ChannelId,
        },
        method = DELETE "/channels/{channel_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [channel_id.into_inner().to_u64(), 0],
            )
        },
        response = [json] Channel,
    ),
    (
        GetChannelMessages {
            channel_id: ChannelId,
        },
        method = GET "/channels/{channel_id}/messages",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [channel_id.into_inner().to_u64(), 0],
            )
        },
        response = [json] Vec<Message>,
    ),
    (
        GetChannelMessage {
            channel_id: ChannelId,
            message_id: MessageId,
        },
        method = GET "/channels/{channel_id}/messages/{message_id}",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [channel_id.into_inner().to_u64(), 0],
            )
        },
        response = [json] Message,
    ),
    (
        CreateMessage {
            channel_id: ChannelId,
            message: CreateMessageModel,
        },
        body = [json] message,
        method = POST "/channels/{channel_id}/messages",
        info = |method, route| -> DiscordRouteInfo {
            DiscordRouteInfo::with_auth(
                method,
                route,
                [channel_id.into_inner().to_u64(), 0],
            )
        },
        response = [json] Message,
    ),
    // OAuth2
    (
        AuthenticateClientCredentialsGrant {
            scope: ClientCredentialsRequest,
            client_id: ApplicationId,
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
