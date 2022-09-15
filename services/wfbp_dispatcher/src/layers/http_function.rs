use crate::{
    models::{InteractionError, InteractionInputData},
    util::body_to_bytes,
};
use axum::{
    body::{Body, Bytes, HttpBody},
    http::Request,
};
use futures::{future::BoxFuture, ready, FutureExt};
use std::task::{Context, Poll};
use tower::{Layer, Service};
use wfbp_azure::functions::{FunctionsInput, RawHttpInput};

/// [`Layer`] which unwraps the body of an HTTP Trigger request into the request
/// that triggered the function.
#[derive(Clone, Debug, Default)]
pub struct HttpFunctionLayer;

impl<S> Layer<S> for HttpFunctionLayer {
    type Service = HttpFunction<S>;

    fn layer(&self, inner: S) -> Self::Service {
        HttpFunction::new(inner)
    }
}

/// [`Service`] which unwraps the body of an HTTP Trigger request into the
/// request that triggered the function.
#[derive(Debug)]
pub struct HttpFunction<S> {
    inner: S,
    prepared: Option<S>,
}

impl<S> HttpFunction<S> {
    pub fn new(inner: S) -> Self {
        HttpFunction {
            inner,
            prepared: None,
        }
    }
}

impl<S> Clone for HttpFunction<S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            prepared: None,
        }
    }
}

impl<S, ReqBody> Service<Request<ReqBody>> for HttpFunction<S>
where
    S: Service<Request<Body>> + Clone + Send + 'static,
    S::Error: From<InteractionError>,
    S::Future: Send + 'static,
    ReqBody: HttpBody<Data = Bytes> + Send + Unpin + 'static,
    anyhow::Error: From<ReqBody::Error>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut Context,
    ) -> Poll<Result<(), Self::Error>> {
        // Check if already ready
        if self.prepared.is_some() {
            return Poll::Ready(Ok(()));
        }

        // Prepare service
        ready!(self.inner.poll_ready(cx))?;
        let cloned = self.inner.clone();
        let prepared = std::mem::replace(&mut self.inner, cloned);
        self.prepared = Some(prepared);
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // Prepare
        let mut prepared = self.prepared.take().unwrap();

        // Execute
        async move {
            // Get request contents
            let (parts, body) = req.into_parts();
            let payload = body_to_bytes(body).await.map_err(|inner| {
                InteractionError::RequestBody {
                    inner: inner.into(),
                }
            })?;

            // Parse body
            let payload: FunctionsInput<InteractionInputData<RawHttpInput>> =
                serde_json::from_slice(&payload)
                    .map_err(InteractionError::ParseBody)?;
            let request = payload.data.request;

            // Construct new body
            let body = Bytes::from(request.body.into_bytes());
            let req = Request::from_parts(parts, Body::from(body));

            // Execute inner service
            prepared.call(req).await
        }
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use tower::{service_fn, ServiceExt};

    #[tokio::test]
    async fn service_unwraps_request() {
        // Build service
        let inner = service_fn(assert_request);
        let service = HttpFunction::new(inner);

        // Build request
        const BODY: &[u8] = br#"
            {
                "Data": {
                    "request": {
                        "Url": "https://example.com",
                        "Method": "GET",
                        "Query": {},
                        "Headers": {},
                        "Params": {},
                        "Body": "Hello, world!"
                    }
                }
            }
        "#;
        let body = Bytes::from(BODY);
        let req = Request::builder()
            .method("GET")
            .header("test", "test-value")
            .body(Body::from(body))
            .expect("failed to build request");

        // Execute
        service.oneshot(req).await.unwrap();

        // Assert
        async fn assert_request(
            req: Request<Body>,
        ) -> Result<(), InteractionError> {
            assert_eq!(
                req.headers().get("test"),
                Some(&HeaderValue::from_static("test-value"))
            );
            let body = body_to_bytes(req.into_body())
                .await
                .expect("failed to read body");
            assert_eq!(body, Bytes::from("Hello, world!"));

            Ok(())
        }
    }
}
