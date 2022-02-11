use futures::future::BoxFuture;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
    Rng, SeedableRng,
};
use std::{
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::sync::Mutex;
use tower::{Layer, Service};
use tracing::trace;

/// Adds random jitter to requests.
#[derive(Clone, Debug)]
pub struct JitterLayer<D, R> {
    dist: Arc<D>,
    rng: Arc<Mutex<R>>,
}

impl<D, R> JitterLayer<D, R> {
    pub fn new(dist: D, rng: R) -> Self {
        JitterLayer {
            dist: Arc::new(dist),
            rng: Arc::new(Mutex::new(rng)),
        }
    }
}

impl Default for JitterLayer<Uniform<u64>, StdRng> {
    fn default() -> Self {
        Self::new(Uniform::new_inclusive(0, 30), StdRng::from_entropy())
    }
}

impl<D, R, Next> Layer<Next> for JitterLayer<D, R> {
    type Service = JitterService<D, R, Next>;

    fn layer(&self, next: Next) -> Self::Service {
        JitterService {
            dist: self.dist.clone(),
            rng: self.rng.clone(),
            next,
        }
    }
}

#[derive(Clone, Debug)]
pub struct JitterService<D, R, Next> {
    dist: Arc<D>,
    rng: Arc<Mutex<R>>,
    next: Next,
}

impl<D, R, Req, Next> Service<Req> for JitterService<D, R, Next>
where
    D: Distribution<u64> + Send + Sync + 'static,
    R: Rng + Send + 'static,
    Next: Service<Req>,
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
        let rng = self.rng.clone();
        let dist = self.dist.clone();
        let next_fut = self.next.call(req);

        Box::pin(async move {
            let delay = {
                let mut rng = rng.lock().await;
                Duration::from_millis(dist.sample(&mut *rng))
            };

            if delay > Duration::ZERO {
                trace!(?delay, "applying jitter");
                tokio::time::sleep(delay).await;
            }

            next_fut.await
        })
    }
}
