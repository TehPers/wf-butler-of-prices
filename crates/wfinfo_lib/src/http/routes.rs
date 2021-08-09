use crate::{
    client::WebhookDiscordClient,
    models::{Channel, CreateMessage, Message, Snowflake},
    request::RateLimitBucket,
};
use anyhow::Context;
use reqwest::{Client, Method, RequestBuilder};
use std::fmt::{Display, Formatter};
use tracing::instrument;

macro_rules! routes {
    (@req_body $builder:expr,) => {
        $builder
    };
    (@req_body $builder:expr, $body:expr) => {
        $builder.json($body)
    };
    ($(
        (
            $variant:ident {
                $($url_param:ident : $url_param_type:ty),*
                $(,)?
            },
            $({ body = $body_param:ident : $body_param_type:ty },)?
            $method:ident $route:expr,
            $major_params:expr,
            $helper:ident -> $response:ty
            $(,)?
        )
    ),* $(,)?) => {
        #[derive(Clone, Debug)]
        pub enum Route {
            $(
                $variant {
                    $($url_param: $url_param_type,)*
                    $($body_param: $body_param_type,)?
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
                match self {
                    $(
                        Route::$variant { $($url_param,)* .. } => {
                            format!(
                                concat!("{}", $route),
                                Self::BASE_URL,
                                $($url_param = $url_param),*
                            )
                        },
                    )*
                }
            }

            pub fn bucket(&self) -> RateLimitBucket {
                match self {
                    $(
                        #[allow(unused_variables)]
                        Route::$variant { $($url_param,)* .. } => {
                            RateLimitBucket::new(
                                Method::$method,
                                $route,
                                $major_params,
                            )
                        },
                    )*
                }
            }

            pub fn make_request(&self, client: &Client) -> RequestBuilder {
                match self {
                    $(
                        Route::$variant { $($url_param,)* $($body_param,)? .. } => {
                            let url = format!(
                                concat!("{}", $route),
                                Self::BASE_URL,
                                $($url_param = $url_param),*
                            );
                            let request = client.request(Method::$method, url);
                            let request = routes!(@req_body request, $(&$body_param)?);
                            request
                        }
                    )*
                }
            }

            $(
                #[instrument]
                pub async fn $helper(
                    client: &WebhookDiscordClient
                    $(, $url_param: $url_param_type)*
                    $(, $body_param: $body_param_type)*
                ) -> anyhow::Result<$response> {
                    let route = Self::$variant {
                        $($url_param,)*
                        $($body_param,)?
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
                        Route::$variant { $(ref $url_param),*, .. } => {
                            write!(f, $route, $($url_param = $url_param),*)
                        }
                    )*
                }
            }
        }
    };
}

routes! {
    (
        GetChannel { channel_id: Snowflake },
        GET "/channels/{channel_id}",
        [channel_id.to_u64(), 0],
        get_channel -> Channel,
    ),
    (
        ModifyChannel { channel_id: Snowflake },
        PATCH "/channels/{channel_id}",
        [channel_id.to_u64(), 0],
        modify_channel -> Channel,
    ),
    (
        DeleteChannel { channel_id: Snowflake },
        DELETE "/channels/{channel_id}",
        [channel_id.to_u64(), 0],
        delete_channel -> Channel,
    ),
    (
        GetChannelMessages { channel_id: Snowflake },
        GET "/channels/{channel_id}/messages",
        [channel_id.to_u64(), 0],
        get_channel_messages -> Vec<Message>,
    ),
    (
        GetChannelMessage { channel_id: Snowflake, message_id: Snowflake },
        GET "/channels/{channel_id}/messages/{message_id}",
        [channel_id.to_u64(), 0],
        get_channel_message -> Message,
    ),
    (
        CreateMessage { channel_id: Snowflake },
        { body = message: CreateMessage },
        POST "/channels/{channel_id}/messages",
        [channel_id.to_u64(), 0],
        create_message -> Message,
    ),
}

impl Route {
    /// Base Discord API url (v9).
    pub const BASE_URL: &'static str = "https://discord.com/api/v9";
}
