use anyhow::anyhow;
use futures::future::BoxFuture;
use std::{
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
    task::{Context, Poll},
    time::Duration,
};
use tower::{Layer, Service};
use tracing::trace;

/// Adds exponential backoff to requests. The resulting service is intended to
/// be reused for the same request, assuming there is retry logic in place.
#[derive(Clone, Debug)]
pub struct BackoffLayer {
    base: u64,
}

impl BackoffLayer {
    pub fn new(base: u64) -> Self {
        BackoffLayer { base }
    }
}

impl<Next> Layer<Next> for BackoffLayer {
    type Service = BackoffService<Next>;

    fn layer(&self, next: Next) -> Self::Service {
        BackoffService {
            base: self.base,
            next,
            attempt: Default::default(),
        }
    }
}

impl Default for BackoffLayer {
    fn default() -> Self {
        BackoffLayer::new(20)
    }
}

#[derive(Clone, Debug)]
pub struct BackoffService<Next> {
    base: u64,
    next: Next,
    attempt: Arc<AtomicU8>,
}

impl<Req, Next> Service<Req> for BackoffService<Next>
where
    Next: Service<Req>,
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
        let attempt =
            self.attempt.fetch_add(1, Ordering::Relaxed).checked_add(1);
        let delay_millis = attempt.and_then(|attempt| {
            let factor = 1u64.checked_shl(attempt.into())?;
            self.base.checked_mul(factor)
        });

        match delay_millis {
            None => Box::pin(async { Err(anyhow!("max retries reached"))? }),
            Some(delay_millis) => {
                let delay = Duration::from_millis(delay_millis);
                let next_fut = self.next.call(req);
                Box::pin(async move {
                    // Delay
                    trace!(?attempt, ?delay, "applying exponential backoff");
                    tokio::time::sleep(delay).await;

                    // Execute request
                    next_fut.await
                })
            }
        }
    }
}
