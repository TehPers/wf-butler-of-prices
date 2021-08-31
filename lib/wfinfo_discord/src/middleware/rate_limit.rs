use crate::{
    models::RateLimit, routes::DiscordRouteInfo, RateLimitBucket, RateLimiter,
};
use async_trait::async_trait;
use chrono::Utc;
use derive_more::{Display, Error};
use reqwest_middleware::{Middleware, Next};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tracing::{debug, warn};
use truelayer_extensions::Extensions;
use wfinfo_lib::reqwest::{Request, Response, ResponseBuilderExt};

#[derive(Clone, Debug, Default)]
pub struct RateLimitMiddleware {
    rate_limiters: Arc<Mutex<HashMap<RateLimitBucket, RateLimiter>>>,
}

#[async_trait]
impl Middleware for RateLimitMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        debug!("checking pre-emptive rate limit");

        // Get route info
        let info: &DiscordRouteInfo =
            extensions.get().ok_or(MiddlewareError::MissingRouteInfo)?;

        // Get rate limiter for bucket
        let mut limiter_guard = self.rate_limiters.lock().await;
        let limiter =
            limiter_guard
                .entry(info.bucket.clone())
                .or_insert(RateLimiter {
                    bucket: info.bucket.clone(),
                    limit: 1,
                    remaining: 1,
                    reset: Utc::now(),
                });

        // Wait until rate limit is refreshed if needed
        limiter.wait().await;

        // Make request
        let mut response = next.clone().run(req, extensions).await?;

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
                .map_err(MiddlewareError::ReadBodyError)?;

            // Parse body
            let limit: RateLimit = serde_json::from_slice(&body)
                .map_err(MiddlewareError::GlobalRateLimitParseError)?;
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
                .map_err(MiddlewareError::ReconstructResponseError)?
                .into();
        }

        Ok(response)
    }
}

#[derive(Debug, Display, Error)]
#[non_exhaustive]
enum MiddlewareError {
    #[display(fmt = "missing route info for request")]
    MissingRouteInfo,
    #[display(fmt = "{}", _0)]
    GlobalRateLimitParseError(serde_json::Error),
    #[display(fmt = "error reading response body")]
    ReadBodyError(wfinfo_lib::reqwest::Error),
    #[display(fmt = "error reconstructing response")]
    ReconstructResponseError(http::Error),
}

impl From<MiddlewareError> for reqwest_middleware::Error {
    fn from(error: MiddlewareError) -> Self {
        reqwest_middleware::Error::middleware(error)
    }
}
