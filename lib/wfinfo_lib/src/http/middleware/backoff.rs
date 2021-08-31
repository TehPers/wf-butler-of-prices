use async_trait::async_trait;
use derive_more::{Display, Error};
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next};
use std::time::Duration;
use tracing::trace;
use truelayer_extensions::Extensions;

#[derive(Clone, Debug)]
pub struct BackoffMiddleware {
    base: u64,
}

impl BackoffMiddleware {
    pub fn new(base: u64) -> Self {
        BackoffMiddleware { base }
    }
}

impl Default for BackoffMiddleware {
    fn default() -> Self {
        BackoffMiddleware::new(20)
    }
}

#[async_trait]
impl Middleware for BackoffMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        let attempt: &mut BackoffAttempt = match extensions.get_mut() {
            Some(attempt) => attempt,
            None => {
                extensions.insert(BackoffAttempt::default());
                extensions.get_mut().unwrap()
            }
        };

        // Get next attempt
        let next_attempt = match attempt.0.checked_add(1) {
            Some(next_attempt) => next_attempt,
            None => return Err(MiddlewareError::MaxRetriesReached)?,
        };

        // Delay
        let delay = Duration::from_millis(self.base * (1 << attempt.0));
        trace!(?attempt, ?delay, "applying exponential backoff");
        tokio::time::sleep(delay).await;
        attempt.0 = next_attempt;

        // Execute request
        next.run(req, extensions).await
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub struct BackoffAttempt(pub u8);

#[derive(Debug, Display, Error)]
#[non_exhaustive]
enum MiddlewareError {
    #[display(fmt = "max supported retries reached for backoff")]
    MaxRetriesReached,
}

impl From<MiddlewareError> for reqwest_middleware::Error {
    fn from(error: MiddlewareError) -> Self {
        reqwest_middleware::Error::middleware(error)
    }
}
