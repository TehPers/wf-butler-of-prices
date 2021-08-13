use crate::{
    client::DiscordRestClient,
    models::{
        ApplicationCommand, BatchEditGuildApplicationCommandPermissions,
        Channel, ClientCredentials, ClientCredentialsRequest,
        CreateApplicationCommand, CreateGuildApplicationCommandPermissions,
        CreateMessage, CreateWebhookMessage, EditWebhookMessage,
        GuildApplicationCommandPermissions, InteractionResponse, Message,
        Snowflake,
    },
    request::RateLimitBucket,
};
use anyhow::Context;
use reqwest::{Client, Method, RequestBuilder};
use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};
use tracing::instrument;

macro_rules! routes {
    (@req_body $builder:expr, [$body_type:ident] $body:expr) => {
        $builder.$body_type($body)
    };
    (@req_body $builder:expr, $body:expr) => {
        routes!(@req_body $builder, [json] $body)
    };
    (@req_body $builder:expr,) => {
        $builder
    };
    (@major_params $params:expr) => {
        $params
    };
    (@major_params) => {
        Default::default()
    };
    (@query $builder:expr, $query:expr) => {
        $builder.query($query)
    };
    (@query $builder:expr,) => {
        $builder
    };
    (@is_auth $is_auth:expr) => {
        $is_auth
    };
    (@is_auth) => {
        false
    };
    ($(
        (
            $variant:ident {
                $($url_param:ident : $url_param_type:ty),*
                $(,)?
            },
            $(body = $([$body_type:ident])? $body_param:ident : $body_param_type:ty,)?
            $(extra = {
                $($extra_field:ident : $extra_field_type:ty),*
                $(,)?
            },)?
            method = $method:ident $route:expr,
            $(query = $query:expr,)?
            $(major_params = $major_params:expr,)?
            $(processor = |$req:ident| $processor:expr,)?
            $(is_auth = $is_auth:expr,)?
            helper = $helper:ident -> $response:ty
            $(,)?
        )
    ),* $(,)?) => {
        #[derive(Clone, Debug)]
        pub enum Route {
            $(
                $variant {
                    $($url_param: $url_param_type,)*
                    $($body_param: $body_param_type,)?
                    $($($extra_field: $extra_field_type,)*)?
                },
            )*
        }

        impl Route {
            pub fn method(&self) -> Method {
                match self {
                    $(Route::$variant { .. } => Method::$method,)*
                }
            }

            pub fn url(&self) -> String {
                format!("{}{}", Self::BASE_URL, self)
            }

            pub fn bucket(&self) -> RateLimitBucket {
                match self {
                    $(
                        #[allow(unused_variables)]
                        Route::$variant { $($url_param,)* .. } => {
                            RateLimitBucket::new(
                                Method::$method,
                                $route,
                                routes!(@major_params $($major_params)?),
                            )
                        },
                    )*
                }
            }

            pub fn is_auth(&self) -> bool {
                match self {
                    $(
                        Route::$variant { .. } => {
                            routes!(@is_auth $($is_auth)?)
                        },
                    )*
                }
            }

            pub fn make_request(&self, client: &Client) -> RequestBuilder {
                match self {
                    $(
                        Route::$variant { $($url_param,)* $($body_param,)? $($($extra_field,)*)? } => {
                            // Build request URL
                            let url = format!(
                                concat!("{}", $route),
                                Self::BASE_URL,
                                $($url_param = $url_param),*
                            );

                            // Build request
                            let request = client.request(Method::$method, url);

                            // Body
                            let request = routes!(@req_body request, $($([$body_type])? &$body_param)?);

                            // Query string
                            let request = routes!(@query request, $($query)?);

                            // Processor
                            $(
                                let $req = request;
                                let request = $processor;
                            )?

                            request
                        }
                    )*
                }
            }

            $(
                #[instrument]
                pub async fn $helper(
                    client: &DiscordRestClient
                    $(, $url_param: $url_param_type)*
                    $(, $body_param: $body_param_type)?
                    $($(, $extra_field: $extra_field_type)*)?
                ) -> anyhow::Result<$response> {
                    let route = Self::$variant {
                        $($url_param,)*
                        $($body_param,)?
                        $($($extra_field,)*)?
                    };

                    let response = client.request(route)
                        .await
                        .context("error sending request")?
                        .json()
                        .await
                        .context("error parsing response")?;

                    Ok(response)
                }
            )*
        }

        impl Display for Route {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Route::$variant { $(ref $url_param,)* .. } => {
                            write!(f, $route, $($url_param = $url_param),*)
                        }
                    )*
                }
            }
        }
    };
}

impl Route {
    /// Base Discord API url (v9).
    pub const BASE_URL: &'static str = "https://discord.com/api/v9";
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
        body = message: CreateMessage,
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
        is_auth = true,
        helper = authenticate_client_credentials_grant -> ClientCredentials,
    ),
}
