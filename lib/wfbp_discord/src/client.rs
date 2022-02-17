use crate::{
    middleware::{AuthenticationLayer, ClientSecret, RateLimitLayer},
    models::Snowflake,
    routes::DiscordRouteInfo,
};
use async_trait::async_trait;
use reqwest::{Client, RequestBuilder, Response};
use std::{fmt::Debug, sync::Arc};
use tower::ServiceBuilder;
use wfbp_http::{
    middleware::{
        BackoffLayer, ExecuteRequestService, JitterLayer, LimitLayer,
        RestRequestBuilder, RetryLayer, RouteLayer,
        TransientRequestRetryPolicy,
    },
    RequestError, RestClient, RestRequestLayer, Route, StandardRestClient,
};

#[derive(Clone, Debug)]
pub struct DiscordRestClient {
    inner: StandardRestClient,
}

impl DiscordRestClient {
    pub const BASE_URL: &'static str = "https://discord.com/api/v9";

    pub fn new(
        client: Client,
        client_id: Snowflake,
        client_secret: Arc<ClientSecret>,
    ) -> Self {
        let auth_client =
            StandardRestClient::new(client.clone(), Self::BASE_URL);

        let request_layer = ServiceBuilder::new()
            .layer(RetryLayer::new(TransientRequestRetryPolicy::default()))
            .layer(LimitLayer::new(10))
            .layer(AuthenticationLayer::new(auth_client, client_id, client_secret))
            .layer(BackoffLayer::default())
            .layer(RateLimitLayer::default())
            .layer(JitterLayer::default())
            .map_request(RequestBuilder::from)
            .map_err(RequestError::from)
            .check_service::<ExecuteRequestService, RestRequestBuilder, Response, RequestError>();
        let inner = StandardRestClient::new_from_layers(
            RouteLayer::new(client, Self::BASE_URL.into()),
            RestRequestLayer::new(request_layer),
        );
        DiscordRestClient { inner }
    }
}

#[async_trait]
impl<R> RestClient<R> for DiscordRestClient
where
    R: Route<Info = DiscordRouteInfo>,
{
    async fn request(&self, route: R) -> Result<R::Response, RequestError> {
        self.inner.request(route).await
    }
}
