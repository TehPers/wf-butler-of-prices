use async_trait::async_trait;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
    SeedableRng,
};
use reqwest_middleware::Middleware;
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tracing::trace;

pub struct JitterMiddleware<D: Distribution<u64> + Send + Sync + 'static> {
    dist: D,
    rng: Arc<Mutex<StdRng>>,
}

impl<D: Distribution<u64> + Send + Sync + 'static> JitterMiddleware<D> {
    pub fn new(dist: D) -> Self {
        JitterMiddleware {
            dist,
            rng: Arc::new(Mutex::new(StdRng::from_entropy())),
        }
    }
}

impl Default for JitterMiddleware<Uniform<u64>> {
    fn default() -> Self {
        JitterMiddleware::new(Uniform::new_inclusive(0, 30))
    }
}

#[async_trait]
impl<D: Distribution<u64> + Send + Sync + 'static> Middleware
    for JitterMiddleware<D>
{
    async fn handle(
        &self,
        req: reqwest::Request,
        extensions: &mut truelayer_extensions::Extensions,
        next: reqwest_middleware::Next<'_>,
    ) -> reqwest_middleware::Result<reqwest::Response> {
        let delay = {
            let mut rng = self.rng.lock().await;
            Duration::from_millis(self.dist.sample(&mut *rng))
        };

        if delay > Duration::ZERO {
            trace!(?delay, "applying jitter");
            tokio::time::sleep(delay).await;
        }

        next.run(req, extensions).await
    }
}
