use crate::{
    middleware::{CacheLayer, LocalCacheStorage},
    routes::WmRouteInfo,
};
use async_trait::async_trait;
use reqwest::{Client, RequestBuilder, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use tower::{util::BoxLayer, ServiceBuilder, ServiceExt};
use wfbp_http::{
    middleware::{
        BackoffLayer, ExecuteRequestService, JitterLayer, LimitLayer,
        RestRequestBuilder, RetryLayer, RouteLayer,
        TransientRequestRetryPolicy,
    },
    RequestError, RestClient, RestRequestLayer, Route,
};

#[derive(Clone, Debug)]
pub struct WmRestClient {
    cache_layer: CacheLayer<LocalCacheStorage>,
    route_layer: RouteLayer,
    request_layer: RestRequestLayer,
}

impl WmRestClient {
    pub const BASE_URL: &'static str = "https://api.warframe.market/v1";

    pub fn new(client: Client) -> Self {
        let cache_layer = CacheLayer::new(LocalCacheStorage::default());
        let route_layer = RouteLayer::new(client, Self::BASE_URL.into());
        let request_layer = ServiceBuilder::new()
            .layer(RetryLayer::new(TransientRequestRetryPolicy::default()))
            .layer(LimitLayer::new(10))
            .layer(BackoffLayer::default())
            .layer(JitterLayer::default())
            .map_request(RequestBuilder::from)
            .map_err(RequestError::from)
            .check_service::<ExecuteRequestService, RestRequestBuilder, Response, RequestError>();

        Self {
            cache_layer,
            route_layer,
            request_layer: BoxLayer::new(request_layer),
        }
    }
}

#[async_trait]
impl<R> RestClient<R> for WmRestClient
where
    R: Route<Info = WmRouteInfo>,
    <R as Route>::Response:
        Serialize + DeserializeOwned + Send + Sync + 'static,
{
    async fn request(&self, route: R) -> Result<R::Response, RequestError> {
        let service = ServiceBuilder::new()
            .layer(&self.cache_layer)
            .layer(&self.route_layer)
            .layer(&self.request_layer)
            .check_service::<ExecuteRequestService, R, R::Response, RequestError>()
            .service(ExecuteRequestService::default());

        // Execute service
        service.oneshot(route).await
    }
}
