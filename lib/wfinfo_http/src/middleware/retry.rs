use anyhow::anyhow;
use futures::future::BoxFuture;
use reqwest::{Response, StatusCode};
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::warn;

/// Retries a request indefinitely, halting only if the inner service errors or
/// returns a fatal response.
#[derive(Clone, Debug)]
pub struct RetryLayer<P> {
    policy: P,
}

impl<P> RetryLayer<P> {
    pub fn new(policy: P) -> Self {
        RetryLayer { policy }
    }
}

impl<P, Next> Layer<Next> for RetryLayer<P>
where
    P: Clone,
{
    type Service = RetryService<P, Next>;

    fn layer(&self, next: Next) -> Self::Service {
        RetryService {
            policy: self.policy.clone(),
            next,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RetryService<P, Next> {
    policy: P,
    next: Next,
}

impl<P, Req, Next> Service<Req> for RetryService<P, Next>
where
    P: RetryPolicy<Next::Response> + Clone + Send + 'static,
    Req: Clone + Send + 'static,
    Next: Service<Req> + Clone + Send + 'static,
    Next::Error: From<anyhow::Error>,
    Next::Future: Send + 'static,
{
    type Response = Next::Response;
    type Error = Next::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.next.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        let mut next = self.next.clone();
        let policy = self.policy.clone();

        Box::pin(async move {
            loop {
                // Poll next until it's ready
                futures::future::poll_fn(|cx| next.poll_ready(cx)).await?;

                // Make request
                let req = req.clone();
                let res = next.call(req).await?;

                // Process response
                let response_kind = policy.classify(&res);
                match response_kind {
                    ResponseKind::Success => return Ok(res),
                    ResponseKind::Transient => {
                        warn!("request failed (transient failure)")
                    }
                    ResponseKind::Fatal => {
                        return Err(anyhow!("request failed").into())
                    }
                }
            }
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ResponseKind {
    Success,
    Transient,
    Fatal,
}

pub trait RetryPolicy<Res> {
    fn classify(&self, res: &Res) -> ResponseKind;
}

#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct TransientRequestRetryPolicy {}

impl RetryPolicy<Response> for TransientRequestRetryPolicy {
    fn classify(&self, res: &Response) -> ResponseKind {
        match res.status() {
            status if status.is_success() => ResponseKind::Success,
            status if status.is_server_error() => ResponseKind::Transient,
            StatusCode::REQUEST_TIMEOUT | StatusCode::TOO_MANY_REQUESTS => {
                ResponseKind::Transient
            }
            _ => ResponseKind::Fatal,
        }
    }
}
