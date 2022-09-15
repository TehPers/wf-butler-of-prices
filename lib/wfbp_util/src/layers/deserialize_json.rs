use axum::{
    http::{Request, Response},
    BoxError,
};
use futures::{future::BoxFuture, ready, FutureExt};
use std::{
    future::Future,
    task::{Context, Poll},
};
use tower::{Layer, Service};

/// Try to apply a transformation to the request body.
#[derive(Clone, Debug)]
pub struct TryMapRequestBodyLayer<F> {
    f: F,
}

impl<F> TryMapRequestBodyLayer<F> {
    /// Create a new [`TryMapRequestBodyLayer`].
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<S, F> Layer<S> for TryMapRequestBodyLayer<F>
where
    F: Clone,
{
    type Service = TryMapRequestBody<S, F>;

    fn layer(&self, inner: S) -> Self::Service {
        TryMapRequestBody::new(inner, self.f.clone())
    }
}

/// Try to apply a transformation to the request body.
#[derive(Debug)]
pub struct TryMapRequestBody<S, F> {
    inner: S,
    f: F,
    prepared: Option<S>,
}

impl<S, F> TryMapRequestBody<S, F> {
    /// Create a new [`TryMapRequestBody`].
    pub fn new(inner: S, f: F) -> Self {
        Self {
            inner,
            f,
            prepared: None,
        }
    }
}

impl<S, F> Clone for TryMapRequestBody<S, F>
where
    S: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            f: self.f.clone(),
            prepared: None,
        }
    }
}

impl<S, F, Fut, ReqBody, ResBody, NewReqBody, E> Service<Request<ReqBody>>
    for TryMapRequestBody<S, F>
where
    S: Service<Request<NewReqBody>, Response = Response<ResBody>>
        + Clone
        + Send
        + 'static,
    S::Error: Into<BoxError>,
    S::Future: Send,
    F: FnOnce(ReqBody) -> Fut + Clone,
    Fut: Future<Output = Result<NewReqBody, E>> + Send + 'static,
    NewReqBody: Send,
    E: Into<BoxError>,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut Context,
    ) -> Poll<Result<(), Self::Error>> {
        // Check if already prepared
        if self.prepared.is_some() {
            return Poll::Ready(Ok(()));
        }

        // Prepare service
        ready!(self.inner.poll_ready(cx)).map_err(Into::into)?;
        let cloned = self.inner.clone();
        let prepared = std::mem::replace(&mut self.inner, cloned);
        self.prepared = Some(prepared);
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // Prepare
        let mut inner = self.prepared.take().expect("service not ready");
        let (parts, body) = req.into_parts();
        let new_body = (self.f.clone())(body);

        // Execute
        async move {
            let new_body = new_body.await.map_err(Into::into)?;
            let req = Request::from_parts(parts, new_body);
            inner.call(req).await.map_err(Into::into)
        }
        .boxed()
    }
}
