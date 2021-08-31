use async_trait::async_trait;
use derive_more::{Display, Error};
use reqwest::{Request, Response, StatusCode};
use reqwest_middleware::{Middleware, Next};
use tracing::warn;
use truelayer_extensions::Extensions;

#[derive(Clone, Debug)]
pub struct RetryMiddleware {
    max_retries: usize,
}

impl RetryMiddleware {
    pub fn new(max_retries: usize) -> Self {
        RetryMiddleware { max_retries }
    }
}

impl Default for RetryMiddleware {
    fn default() -> Self {
        RetryMiddleware::new(10)
    }
}

#[async_trait]
impl Middleware for RetryMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        for _ in 0..self.max_retries {
            // Make request
            let req =
                req.try_clone().ok_or(MiddlewareError::RequestNotClonable)?;
            let res = next.clone().run(req, extensions).await;

            // Process response
            let response_kind = match res.as_ref() {
                Ok(response) => match response.status() {
                    status if status.is_success() => ResponseKind::Success,
                    status if status.is_server_error() => {
                        ResponseKind::Transient
                    }
                    StatusCode::REQUEST_TIMEOUT
                    | StatusCode::TOO_MANY_REQUESTS => ResponseKind::Transient,
                    _ => ResponseKind::Fatal,
                },
                Err(_) => ResponseKind::Fatal,
            };

            // Only retry for transient errors
            if response_kind != ResponseKind::Transient {
                return res;
            }

            warn!(?res, "request failed (transient failure)")
        }

        Err(MiddlewareError::MaxRetriesReached)?
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum ResponseKind {
    Success,
    Transient,
    Fatal,
}

#[derive(Debug, Display, Error)]
enum MiddlewareError {
    #[display(fmt = "request cannot be cloned")]
    RequestNotClonable,
    #[display(fmt = "max allowed retries reached")]
    MaxRetriesReached,
}

impl From<MiddlewareError> for reqwest_middleware::Error {
    fn from(error: MiddlewareError) -> Self {
        reqwest_middleware::Error::middleware(error)
    }
}
