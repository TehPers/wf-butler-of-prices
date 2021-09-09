use crate::{
    middleware::{AuthenticationMiddleware, RateLimitMiddleware},
    models::Snowflake,
    routes::DiscordRouteInfo,
};
use async_trait::async_trait;
use reqwest_middleware::Middleware;
use std::sync::Arc;
use truelayer_extensions::Extensions;
use wfinfo_lib::{
    http::{
        middleware::{
            BackoffMiddleware, JitterMiddleware, RetryMiddleware,
            ToErrorMiddleware,
        },
        RequestError, RestClient, Route, StandardRestClient,
    },
    reqwest::{Client, Response},
};

#[derive(Clone, Debug)]
pub struct DiscordRestClient {
    inner: StandardRestClient,
}

impl DiscordRestClient {
    pub const BASE_URL: &'static str = "https://discord.com/api/v9";

    pub fn new(
        raw_client: Client,
        client_id: Snowflake,
        client_secret: Arc<String>,
    ) -> Self {
        let to_error = ToErrorMiddleware::default();
        let retry = RetryMiddleware::default();
        let auth = AuthenticationMiddleware::new(
            raw_client.clone(),
            Self::BASE_URL,
            client_id,
            client_secret,
        );
        let backoff = BackoffMiddleware::default();
        let rate_limit = RateLimitMiddleware::default();
        let jitter = JitterMiddleware::default();
        let middleware: [Arc<dyn Middleware>; 6] = [
            Arc::new(to_error),
            Arc::new(retry),
            Arc::new(auth),
            Arc::new(backoff),
            Arc::new(rate_limit),
            Arc::new(jitter),
        ];

        DiscordRestClient {
            inner: StandardRestClient::new_with_middleware(
                raw_client,
                Self::BASE_URL,
                middleware,
            ),
        }
    }
}

#[async_trait]
impl RestClient<DiscordRouteInfo> for DiscordRestClient {
    async fn request<R: Route<Info = DiscordRouteInfo>>(
        &self,
        route: R,
    ) -> Result<Response, RequestError> {
        let mut extensions = Extensions::new();
        extensions.insert(route.info());

        self.inner.request_with(route, &mut extensions).await
    }
}
