use crate::{
    http::Route,
    models::{ClientCredentialsRequest, RateLimit, Snowflake},
    request::{RateLimitBucket, RateLimiter},
};
use anyhow::{bail, Context};
use async_recursion::async_recursion;
use chrono::Utc;
use rand::{
    distributions::{Distribution, Uniform},
    prelude::StdRng,
    SeedableRng,
};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client, Response, StatusCode,
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    sync::{Mutex, RwLock},
    time::sleep,
};
use tracing::{debug, error, instrument, warn};

#[derive(Clone, Debug)]
pub struct DiscordRestClient {
    http_client: Client,
    rate_limiters: Arc<Mutex<HashMap<RateLimitBucket, RateLimiter>>>,
    access_token: Arc<RwLock<Option<String>>>,
    client_id: Snowflake,
    client_secret: Arc<String>,
    rng: Arc<Mutex<StdRng>>,
}

impl DiscordRestClient {
    const MAX_ATTEMPTS: u64 = 10;

    pub fn new(
        client_id: Snowflake,
        client_secret: String,
    ) -> anyhow::Result<Self> {
        let mut headers = HeaderMap::new();
        let auth_value =
            HeaderValue::from_str(&format!("Bearer {}", client_secret))
                .context("invalid token")?;
        headers.insert(AUTHORIZATION, auth_value);

        let http_client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .https_only(true)
            .user_agent(concat!("TEST_BOT/", env!("CARGO_PKG_VERSION")))
            .build()
            .context("error creating HTTP client")?;

        Ok(DiscordRestClient {
            http_client,
            rate_limiters: Default::default(),
            access_token: Arc::new(RwLock::new(None)),
            client_id,
            client_secret: Arc::new(client_secret),
            rng: Arc::new(Mutex::new(StdRng::from_entropy())),
        })
    }

    #[async_recursion]
    #[instrument]
    pub async fn request(&self, route: Route) -> anyhow::Result<Response> {
        debug!(?route, "outgoing request");
        // debug!(url=?route.url(), "outgoing request");

        let bucket = route.bucket();
        let mut attempt = 0u64;
        let jitter_dist = Uniform::new_inclusive(0, 30);
        loop {
            // Exponential backoff + jitter
            let jitter = {
                let mut rng = self.rng.lock().await;
                Duration::from_millis(jitter_dist.sample(&mut *rng))
            };
            let backoff = Duration::from_millis(20 * (1 << attempt));
            let delay = backoff.saturating_add(jitter);
            if delay > Duration::ZERO {
                debug!(?delay, "applying jitter + backoff");
                tokio::time::sleep(delay).await;
            }

            // Authentication
            let auth_header = if route.is_auth() {
                None
            } else {
                debug!("checking for access token");
                let access_token_guard = self.access_token.read().await;
                let auth_header = if let Some(access_token) =
                    access_token_guard.as_ref()
                {
                    // Fast path - no need to update access token
                    HeaderValue::from_str(&format!("Bearer {}", access_token))
                        .context("invalid token")?
                } else {
                    // Slow path - write lock + verify access token again
                    debug!("checking again for access token");
                    drop(access_token_guard);
                    let mut access_token_guard =
                        self.access_token.write().await;
                    if let Some(access_token) = access_token_guard.as_ref() {
                        HeaderValue::from_str(&format!(
                            "Bearer {}",
                            access_token
                        ))
                        .context("invalid token")?
                    } else {
                        debug!("fetching credentials");
                        let credentials =
                            Route::authenticate_client_credentials_grant(
                                self,
                                ClientCredentialsRequest {
                                    grant_type: "client_credentials".to_owned(),
                                    scope: "applications.commands".to_owned(),
                                },
                                self.client_id,
                                self.client_secret.clone(),
                            )
                            .await?;
                        let access_token =
                            access_token_guard.insert(credentials.access_token);
                        HeaderValue::from_str(&format!(
                            "Bearer {}",
                            access_token
                        ))
                        .context("invalid token")?
                    }
                };

                Some(auth_header)
            };

            // Rate limit
            debug!("checking pre-emptive rate limit");
            let mut limiter_guard = self.rate_limiters.lock().await;
            let limiter =
                limiter_guard.entry(bucket.clone()).or_insert(RateLimiter {
                    bucket: bucket.clone(),
                    limit: 1,
                    remaining: 1,
                    reset: Utc::now(),
                });
            limiter.wait().await;

            // Build request
            let mut request = route.make_request(&self.http_client);
            if let Some(auth_header) = auth_header {
                request = request.header(AUTHORIZATION, auth_header);
            }

            // Make request
            debug!(?request, "making request");
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

            // Unlock mutex
            drop(limiter_guard);

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
                    debug!(?response, "success");
                    return Ok(response);
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
                    if response.status() == StatusCode::UNAUTHORIZED =>
                {
                    // TODO: refresh oauth2 token
                    self.access_token.write().await.take();
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
