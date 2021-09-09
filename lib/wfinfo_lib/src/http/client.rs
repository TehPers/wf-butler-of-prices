use crate::http::{
    middleware::{BackoffMiddleware, JitterMiddleware, RetryMiddleware},
    Route,
};
use async_trait::async_trait;
use derive_more::{Display, Error};
use reqwest::{Client, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Middleware};
use std::{
    borrow::Cow,
    fmt::{Debug, Formatter},
    sync::Arc,
};
use truelayer_extensions::Extensions;

#[async_trait]
pub trait RestClient<I> {
    async fn request<R: Route<Info = I>>(
        &self,
        route: R,
    ) -> Result<Response, RequestError>;
}

#[derive(Debug, Display, Error)]
#[non_exhaustive]
pub enum RequestError {
    #[display(fmt = "error building request")]
    RequestBuildFailed(reqwest::Error),
    #[display(fmt = "error sending request")]
    RequestFailed(reqwest_middleware::Error),
    #[display(fmt = "error parsing response")]
    ResponseParseError(reqwest::Error),
    #[display(fmt = "{}", _0)]
    Custom(#[error(ignore)] Cow<'static, str>),
}

#[derive(Clone)]
pub struct StandardRestClient {
    client: ClientWithMiddleware,
    raw_client: Client,
    base_url: Cow<'static, str>,
}

impl StandardRestClient {
    pub fn new(
        raw_client: Client,
        base_url: impl Into<Cow<'static, str>>,
    ) -> Self {
        let retry = RetryMiddleware::default();
        let backoff = BackoffMiddleware::default();
        let jitter = JitterMiddleware::default();
        let middleware: [Arc<dyn Middleware>; 3] =
            [Arc::new(retry), Arc::new(backoff), Arc::new(jitter)];

        StandardRestClient::new_with_middleware(
            raw_client, base_url, middleware,
        )
    }

    pub fn new_with_middleware(
        raw_client: Client,
        base_url: impl Into<Cow<'static, str>>,
        middleware: impl IntoIterator<Item = Arc<dyn Middleware>>,
    ) -> Self {
        let client = middleware
            .into_iter()
            .fold(
                ClientBuilder::new(raw_client.clone()),
                ClientBuilder::with_arc,
            )
            .build();

        StandardRestClient {
            client,
            raw_client,
            base_url: base_url.into(),
        }
    }

    pub async fn request_with<R: Route>(
        &self,
        route: R,
        ext: &mut Extensions,
    ) -> Result<Response, RequestError> {
        let req = route
            .create_request(|method, path| {
                self.raw_client
                    .request(method, format!("{}{}", self.base_url, path))
            })
            .build()
            .map_err(RequestError::RequestBuildFailed)?;
        self.client
            .execute_with_extensions(req, ext)
            .await
            .map_err(RequestError::RequestFailed)
    }
}

#[async_trait]
impl<I> RestClient<I> for StandardRestClient {
    async fn request<R: Route<Info = I>>(
        &self,
        route: R,
    ) -> Result<Response, RequestError> {
        self.request_with(route, &mut Extensions::new()).await
    }
}

impl Debug for StandardRestClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RestClient")
            .field("raw_client", &self.raw_client)
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}
