use crate::{
    middleware::{
        BackoffLayer, ExecuteRequestService, JitterLayer, LimitLayer,
        RestRequestBuilder, RetryLayer, RouteLayer,
        TransientRequestRetryPolicy,
    },
    RequestError, Route,
};
use async_trait::async_trait;
use reqwest::{Client, RequestBuilder, Response};
use std::borrow::Cow;
use tower::{util::BoxLayer, ServiceBuilder, ServiceExt};

#[async_trait]
pub trait RestClient<R>
where
    R: Route,
{
    async fn request(&self, route: R) -> Result<R::Response, RequestError>;
}

pub type RestRequestLayer =
    BoxLayer<ExecuteRequestService, RestRequestBuilder, Response, RequestError>;

#[derive(Clone, Debug)]
pub struct StandardRestClient {
    route_layer: RouteLayer,
    request_layer: RestRequestLayer,
}

impl StandardRestClient {
    pub fn new(client: Client, base_url: impl Into<Cow<'static, str>>) -> Self {
        let base_url = base_url.into();
        let service = ServiceBuilder::new()
            .layer(RetryLayer::new(TransientRequestRetryPolicy::default()))
            .layer(LimitLayer::new(10))
            .layer(BackoffLayer::default())
            .layer(JitterLayer::default())
            .map_request(RequestBuilder::from)
            .map_err(RequestError::from)
            .check_service::<ExecuteRequestService, RestRequestBuilder, Response, RequestError>();

        StandardRestClient::new_from_layers(
            RouteLayer::new(client, base_url),
            BoxLayer::new(service),
        )
    }

    pub fn new_from_layers(
        route_layer: RouteLayer,
        request_layer: RestRequestLayer,
    ) -> Self {
        StandardRestClient {
            route_layer,
            request_layer,
        }
    }
}

#[async_trait]
impl<R> RestClient<R> for StandardRestClient
where
    R: Route,
{
    async fn request(&self, route: R) -> Result<R::Response, RequestError> {
        let service = ServiceBuilder::new()
            .layer(&self.route_layer)
            .layer(&self.request_layer)
            .check_service::<ExecuteRequestService, R, R::Response, RequestError>()
            .service(ExecuteRequestService::default());

        // Execute service
        service.oneshot(route).await
    }
}
