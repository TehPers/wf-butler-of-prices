use crate::models::functions::FunctionsHttpInput;
use axum::{
    body::{Body, Bytes},
    http::{Request, Uri},
};
use futures::ready;
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// [`Layer`] which converts request bodies from [`FunctionsHttpInput`] to
/// [`Body`].
#[derive(Clone, Debug, Default)]
pub struct HttpFunctionLayer;

impl<S> Layer<S> for HttpFunctionLayer {
    type Service = HttpFunctionToBody<S>;

    fn layer(&self, inner: S) -> Self::Service {
        HttpFunctionToBody::new(inner)
    }
}

/// [`Service`] which converts a request body from [`FunctionsHttpInput`] to
/// [`Body`].
#[derive(Debug)]
pub struct HttpFunctionToBody<S> {
    inner: S,
    prepared: Option<S>,
}

impl<S> HttpFunctionToBody<S> {
    pub fn new(inner: S) -> Self {
        HttpFunctionToBody {
            inner,
            prepared: None,
        }
    }
}

impl<S> Clone for HttpFunctionToBody<S>
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

impl<S> Service<Request<FunctionsHttpInput>> for HttpFunctionToBody<S>
where
    S: Service<Request<Body>> + Clone + Send + 'static,
    S::Error: From<serde_json::Error>,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

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

    fn call(&mut self, req: Request<FunctionsHttpInput>) -> Self::Future {
        // Deconstruct request
        let (mut parts, body) = req.into_parts();

        // Construct new request
        parts.method = body.method;
        parts.headers = body.headers;
        let body = Bytes::from(body.body.into_bytes());
        let req = Request::from_parts(parts, Body::from(body));

        // Execute inner service
        self.inner.call(req)
    }
}
