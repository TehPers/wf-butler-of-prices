use crate::{
    models::RateLimit, routes::DiscordRouteInfo, RateLimitBucket, RateLimiter,
};
use anyhow::anyhow;
use chrono::Utc;
use derive_more::{Display, Error, From};
use futures::future::BoxFuture;
use reqwest::{Response, ResponseBuilderExt};
use std::{
    collections::HashMap,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::sync::Mutex;
use tower::{Layer, Service};
use tracing::warn;
use wfinfo_http::{middleware::RestRequestBuilder, RequestError};

#[derive(Clone, Debug, Default)]
pub struct RateLimitLayer {
    rate_limiters: Arc<Mutex<HashMap<RateLimitBucket, RateLimiter>>>,
}

impl<Next> Layer<Next> for RateLimitLayer {
    type Service = RateLimitService<Next>;

    fn layer(&self, next: Next) -> Self::Service {
        RateLimitService {
            rate_limiters: self.rate_limiters.clone(),
            next,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RateLimitService<Next> {
    rate_limiters: Arc<Mutex<HashMap<RateLimitBucket, RateLimiter>>>,
    next: Next,
}

impl<Next> Service<RestRequestBuilder> for RateLimitService<Next>
where
    Next: Service<RestRequestBuilder, Response = Response>
        + Send
        + Sync
        + 'static,
    Next::Error: From<RateLimitError>,
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

    fn call(&mut self, req: RestRequestBuilder) -> Self::Future {
        // Get route info
        let info: &DiscordRouteInfo = match req.get() {
            Some(info) => info,
            None => {
                return Box::pin(async move {
                    Err(RateLimitError::MissingRouteInfo)?
                })
            }
        };

        let bucket = info.bucket.clone();
        let next_fut = self.next.call(req);
        let rate_limiters = self.rate_limiters.clone();
        Box::pin(async move {
            // Get rate limiter for bucket
            let mut limiter_guard = rate_limiters.lock().await;
            let limiter =
                limiter_guard.entry(bucket.clone()).or_insert(RateLimiter {
                    bucket: bucket.clone(),
                    limit: 1,
                    remaining: 1,
                    reset: Utc::now(),
                });

            // Wait until rate limit is refreshed if needed
            limiter.wait().await;

            // Execute request
            let mut response = next_fut.await?;

            // Process response
            limiter.update(&response);

            // Check for global rate limit
            let global_limit_hit = response
                .headers()
                .get(RateLimiter::RATELIMIT_GLOBAL)
                .and_then(|v| v.to_str().ok())
                .filter(|v| v.to_ascii_lowercase() == "true")
                .is_some();
            if global_limit_hit {
                let status = response.status();
                let headers = response.headers().clone();
                let url = response.url().clone();
                let body = response
                    .bytes()
                    .await
                    .map_err(RateLimitError::ReadBodyError)?;

                // Parse body
                let limit: RateLimit = serde_json::from_slice(&body)
                    .map_err(RateLimitError::GlobalRateLimitParseError)?;
                warn!(?limit, "hit global rate limit");
                tokio::time::sleep(Duration::from_secs_f32(limit.retry_after))
                    .await;

                // Reconstruct response
                let mut builder = http::Response::builder();
                *builder.headers_mut().unwrap() = headers;
                response = builder
                    .status(status)
                    .url(url)
                    .body(body)
                    .map_err(RateLimitError::ReconstructResponseError)?
                    .into();
            }

            Ok(response)
        })
    }
}

#[derive(Debug, Display, Error, From)]
#[non_exhaustive]
pub enum RateLimitError {
    #[display(fmt = "missing route info for request")]
    MissingRouteInfo,
    #[display(fmt = "{_0}")]
    GlobalRateLimitParseError(serde_json::Error),
    #[display(fmt = "error reading response body")]
    ReadBodyError(reqwest::Error),
    #[display(fmt = "error reconstructing response")]
    ReconstructResponseError(http::Error),
}

impl From<RateLimitError> for RequestError {
    fn from(err: RateLimitError) -> Self {
        match err {
            RateLimitError::ReadBodyError(err) => {
                RequestError::ReqwestError(err)
            }
            RateLimitError::MissingRouteInfo => anyhow!("{err}").into(),
            RateLimitError::GlobalRateLimitParseError(_) => {
                anyhow!("{err}").into()
            }
            RateLimitError::ReconstructResponseError(_) => {
                anyhow!("{err}").into()
            }
        }
    }
}
