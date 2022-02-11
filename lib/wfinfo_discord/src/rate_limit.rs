use chrono::{DateTime, NaiveDateTime, Utc};
use reqwest::{Method, Response};
use tokio::time::sleep;
use tracing::warn;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct RateLimitBucket {
    pub method: Method,
    pub route: &'static str,
    pub major_parameters: [u64; 2],
}

impl RateLimitBucket {
    pub fn new(
        method: Method,
        route: &'static str,
        major_parameters: [u64; 2],
    ) -> Self {
        RateLimitBucket {
            method,
            route,
            major_parameters,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RateLimiter {
    pub bucket: RateLimitBucket,
    pub limit: u32,
    pub remaining: u32,
    pub reset: DateTime<Utc>,
}

impl RateLimiter {
    pub const RATELIMIT_GLOBAL: &'static str = "x-ratelimit-global";
    pub const RATELIMIT_LIMIT: &'static str = "x-ratelimit-limit";
    pub const RATELIMIT_REMAINING: &'static str = "x-ratelimit-remaining";
    pub const RATELIMIT_RESET: &'static str = "x-ratelimit-reset";
    pub const RATELIMIT_BUCKET: &'static str = "x-ratelimit-bucket";

    pub async fn wait(&mut self) {
        self.remaining = match self.remaining.checked_sub(1) {
            Some(remaining) => remaining,
            None => {
                let delay = self.reset - Utc::now();
                match delay.to_std() {
                    Ok(delay) if !delay.is_zero() => {
                        warn!(
                            ?self,
                            "pre-emptive rate limit hit for {}",
                            self.bucket.route
                        );
                        sleep(delay).await
                    }
                    _ => {}
                }

                self.limit
            }
        }
    }

    pub fn update(&mut self, response: &Response) {
        let limit: Option<u32> = response
            .headers()
            .get(Self::RATELIMIT_LIMIT)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.trim().parse().ok());
        let remaining: Option<u32> = response
            .headers()
            .get(Self::RATELIMIT_REMAINING)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.trim().parse().ok());
        let reset = response
            .headers()
            .get(Self::RATELIMIT_RESET)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.trim().parse().ok())
            .map(|t: f64| {
                NaiveDateTime::from_timestamp(t.ceil().max(0.0) as i64, 0)
            })
            .map(|t| DateTime::from_utc(t, Utc));

        self.limit = limit.unwrap_or(self.limit);
        self.remaining = remaining.unwrap_or(self.remaining);
        self.reset = reset.unwrap_or(self.reset);
    }
}
