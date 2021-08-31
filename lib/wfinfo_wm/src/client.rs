use crate::{middleware::CacheMiddleware, routes::RouteInfo};
use async_trait::async_trait;
use reqwest_middleware::Middleware;
use std::sync::Arc;
use truelayer_extensions::Extensions;
use wfinfo_lib::{
    http::{
        middleware::{BackoffMiddleware, JitterMiddleware, RetryMiddleware},
        RequestError, RestClient, Route, StandardRestClient,
    },
    reqwest::{Client, Response},
};

#[derive(Clone, Debug)]
pub struct WarframeMarketRestClient {
    inner: StandardRestClient,
}

impl WarframeMarketRestClient {
    pub const BASE_URL: &'static str = "https://api.warframe.market/v1";

    pub fn new(raw_client: Client) -> Self {
        let cache = CacheMiddleware::default();
        let retry = RetryMiddleware::default();
        let backoff = BackoffMiddleware::default();
        let jitter = JitterMiddleware::default();
        let middleware: [Arc<dyn Middleware>; 4] = [
            Arc::new(cache),
            Arc::new(retry),
            Arc::new(backoff),
            Arc::new(jitter),
        ];

        // TODO: add caching middleware
        WarframeMarketRestClient {
            inner: StandardRestClient::new_with_middleware(
                raw_client,
                Self::BASE_URL,
                middleware,
            ),
        }
    }
}

#[async_trait]
impl RestClient<RouteInfo> for WarframeMarketRestClient {
    async fn request<R: Route<Info = RouteInfo>>(
        &self,
        route: R,
    ) -> Result<Response, RequestError> {
        let mut extensions = Extensions::new();
        extensions.insert(route.info());

        self.inner.request_with(route, &mut extensions).await
    }
}
