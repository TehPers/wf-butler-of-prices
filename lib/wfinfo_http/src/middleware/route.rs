use crate::{middleware::RestRequestBuilder, RequestError, Route};
use anyhow::anyhow;
use futures::future::BoxFuture;
use reqwest::{Client, Response};
use std::{
    borrow::Cow,
    task::{Context, Poll},
};
use tower::{Layer, Service};

#[derive(Clone, Debug)]
pub struct RouteLayer {
    client: Client,
    base_url: Cow<'static, str>,
}

impl RouteLayer {
    pub fn new(client: Client, base_url: Cow<'static, str>) -> Self {
        RouteLayer { client, base_url }
    }
}

impl<Next> Layer<Next> for RouteLayer {
    type Service = RouteService<Next>;

    fn layer(&self, next: Next) -> Self::Service {
        RouteService {
            next,
            client: self.client.clone(),
            base_url: self.base_url.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RouteService<Next> {
    next: Next,
    client: Client,
    base_url: Cow<'static, str>,
}

impl<Req, Next> Service<Req> for RouteService<Next>
where
    Req: Route,
    Next: Service<RestRequestBuilder, Response = Response>,
    Next::Future: Send + 'static,
    RequestError: From<Next::Error>,
{
    type Response = Req::Response;
    type Error = RequestError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.next.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        // Create HTTP request
        let http_req = req.create_request(|method, path| {
            let url = format!("{}{}", self.base_url, path);
            self.client.request(method, url)
        });

        // Convert to REST request builder
        let mut http_req = match RestRequestBuilder::new(&http_req) {
            Some(http_req) => http_req,
            None => {
                return Box::pin(async move {
                    Err(RequestError::Custom(anyhow!(
                        "request can't be cloned"
                    )))
                })
            }
        };

        // Embed route info into request builder
        http_req.insert(req.info());

        // Create the future
        let fut = self.next.call(http_req);
        Box::pin(async move {
            let http_res = fut.await?;
            req.map_response(http_res).await
        })
    }
}
