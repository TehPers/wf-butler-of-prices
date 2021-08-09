use crate::{
    http::Route,
    models::RateLimit,
    request::{RateLimitBucket, RateLimiter},
};
use anyhow::{bail, Context};
use chrono::Utc;
use rand::distributions::{Distribution, Uniform};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client, Response, StatusCode,
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::sleep};
use tracing::{error, warn};

#[derive(Clone, Debug)]
pub struct WebhookDiscordClient {
    http_client: Client,
    rate_limiters: Arc<Mutex<HashMap<RateLimitBucket, RateLimiter>>>,
}

impl WebhookDiscordClient {
    const MAX_ATTEMPTS: u64 = 10;

    pub fn new(app_secret: String) -> anyhow::Result<Self> {
        let mut headers = HeaderMap::new();
        let auth_value =
            HeaderValue::from_str(&format!("Bearer {}", app_secret))
                .context("invalid token")?;
        headers.insert(AUTHORIZATION, auth_value);

        let http_client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .https_only(true)
            .user_agent(concat!("TEST_BOT/", env!("CARGO_PKG_VERSION")))
            .build()
            .context("error creating HTTP client")?;

        Ok(WebhookDiscordClient {
            http_client,
            rate_limiters: Default::default(),
        })
    }

    pub async fn request(&self, route: Route) -> anyhow::Result<Response> {
        let bucket = route.bucket();
        let mut attempt = 0u64;
        let jitter_dist = Uniform::new_inclusive(0, 30);
        let mut rng = rand::thread_rng();
        loop {
            // Exponential backoff + jitter
            let jitter = Duration::from_millis(jitter_dist.sample(&mut rng));
            let backoff = Duration::from_millis(20 * (1 << attempt));
            let delay = backoff.saturating_add(jitter);
            if delay > Duration::ZERO {
                tokio::time::sleep(delay).await;
            }

            // Rate limit
            let mut limiter_guard = self.rate_limiters.lock().await;
            let limiter =
                limiter_guard.entry(bucket.clone()).or_insert(RateLimiter {
                    bucket: bucket.clone(),
                    limit: 1,
                    remaining: 1,
                    reset: Utc::now(),
                });
            limiter.wait().await;

            // Make request
            let request = route.make_request(&self.http_client);
            let response = request.send().await;

            // Update rate limiter
            let hit_global_limit = if let Ok(response) = response.as_ref() {
                limiter.update(response);

                // Global rate limit
                response
                    .headers()
                    .get(RateLimiter::RATELIMIT_GLOBAL)
                    .and_then(|v| v.to_str().ok())
                    .filter(|v| v.to_ascii_lowercase() == "true")
                    .is_some()
            } else {
                false
            };

            if hit_global_limit {
                let response = response.unwrap();
                let limit: RateLimit = response
                    .json()
                    .await
                    .context("error parsing global rate limit response")?;

                warn!(?limit, "hit global rate limit");
                sleep(Duration::from_secs_f32(limit.retry_after)).await;
                continue;
            }

            // Process response
            match response {
                Ok(response) if response.status().is_success() => {
                    return Ok(response)
                }
                Ok(response)
                    if response.status() == StatusCode::TOO_MANY_REQUESTS =>
                {
                    warn!(
                        ?response,
                        ?bucket,
                        "hit rate limit for route {}",
                        bucket.route
                    );
                }
                Ok(response)
                    if response.status().is_client_error()
                        && response.status() != StatusCode::REQUEST_TIMEOUT =>
                {
                    error!(?response, "request failed (client failure)");
                    response.error_for_status().context("request failed")?;
                    bail!("request failed");
                }
                Ok(response) => {
                    // Transient failure, retry request
                    warn!(?response, "request failed (transient failure)")
                }
                Err(error) => {
                    return Err(error).context("error sending request");
                }
            }

            // Update attempt number
            attempt = attempt.saturating_add(1);
            if attempt >= Self::MAX_ATTEMPTS {
                bail!("request failed");
            }
        }
    }
}
