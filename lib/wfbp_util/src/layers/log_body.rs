use crate::{errors::ReadBodyError, util::body_to_bytes};
use axum::{
    body::{Body, Bytes, HttpBody},
    http::{Request, Response, StatusCode},
};
use futures::{future::BoxFuture, ready, FutureExt};
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::info;

#[derive(Clone, Debug)]
pub struct LogBodyLayer;

impl<S> Layer<S> for LogBodyLayer {
    type Service = LogBody<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LogBody {
            inner,
            prepared: None,
        }
    }
}

#[derive(Debug)]
pub struct LogBody<S> {
    inner: S,
    prepared: Option<S>,
}

impl<S> Clone for LogBody<S>
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

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for LogBody<S>
where
    S: Service<Request<Body>, Response = Response<ResBody>>
        + Clone
        + Send
        + 'static,
    S::Error: From<ReadBodyError>,
    S::Future: Send + 'static,
    ReqBody: HttpBody<Data = Bytes> + Send + Unpin + 'static,
    ReqBody::Error: Into<anyhow::Error>,
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
        let mut inner = self.prepared.take().expect("service not ready");

        // Execute
        async move {
            // Get request contents
            let (parts, body) = req.into_parts();
            let payload = body_to_bytes(body).await?;

            // Reconstruct request
            let body = Bytes::from(payload.clone());
            let body = Body::from(body);
            let req = Request::from_parts(parts, body);

            // Call inner service
            let res = inner.call(req).await?;

            // Log body if response status is 400
            if res.status() == StatusCode::BAD_REQUEST {
                info!(?payload);
            }

            Ok(res)
        }
        .boxed()
    }
}
