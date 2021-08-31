use crate::routes::{CacheBucket, RouteInfo};
use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use derive_more::{Display, Error};
use http::{HeaderMap, StatusCode};
use reqwest_middleware::{Middleware, Next};
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};
use tokio::sync::RwLock;
use tracing::debug;
use truelayer_extensions::Extensions;
use wfinfo_lib::reqwest::{Request, Response, ResponseBuilderExt, Url};

#[derive(Clone, Debug, Default)]
pub struct CacheMiddleware {
    buckets: Arc<RwLock<HashMap<CacheBucket, CacheEntry>>>,
}

#[async_trait]
impl Middleware for CacheMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        // Get route info
        let info: &RouteInfo =
            extensions.get().ok_or(MiddlewareError::MissingRouteInfo)?;
        let bucket = match info.bucket.as_ref() {
            Some(bucket) => bucket,
            None => return next.run(req, extensions).await,
        };
        let cache_time = info.cache_time;

        // Check if cached
        let now = Utc::now();
        let buckets_guard = self.buckets.read().await;
        let entry = buckets_guard
            .get(bucket)
            .filter(|entry| !entry.expired(now));

        // Early return if cached
        if let Some(entry) = entry {
            debug!(?bucket, "returning cached entry");
            return entry.response();
        }

        // Make request
        drop(buckets_guard);
        let bucket = bucket.clone();
        let response = next.run(req, extensions).await?;

        // Cache response
        let expires = cache_time.map(|cache_time| Utc::now() + cache_time);
        let mut buckets_guard = self.buckets.write().await;
        match buckets_guard.entry(bucket) {
            Entry::Occupied(mut entry) => {
                entry.insert(
                    CacheEntry::from_response(response, expires).await?,
                );
                entry.get().response()
            }
            Entry::Vacant(entry) => entry
                .insert(CacheEntry::from_response(response, expires).await?)
                .response(),
        }
    }
}

#[derive(Clone, Debug)]
struct CacheEntry {
    expires: Option<DateTime<Utc>>,
    status: StatusCode,
    headers: HeaderMap,
    url: Url,
    body: Bytes,
}

impl CacheEntry {
    pub async fn from_response(
        response: Response,
        expires: Option<DateTime<Utc>>,
    ) -> reqwest_middleware::Result<Self> {
        let status = response.status();
        let headers = response.headers().clone();
        let url = response.url().clone();
        let body = response
            .bytes()
            .await
            .map_err(MiddlewareError::ReadBodyError)?;

        Ok(CacheEntry {
            expires,
            status,
            headers,
            url,
            body,
        })
    }

    pub fn expired(&self, now: DateTime<Utc>) -> bool {
        match self.expires {
            None => false,
            Some(expires) => now >= expires,
        }
    }

    pub fn response(&self) -> reqwest_middleware::Result<Response> {
        // Reconstruct response
        let mut builder = http::Response::builder();
        *builder.headers_mut().unwrap() = self.headers.clone();
        let response = builder
            .status(self.status)
            .url(self.url.clone())
            .body(self.body.clone())
            .map_err(MiddlewareError::ReconstructResponseError)?
            .into();
        Ok(response)
    }
}

#[derive(Debug, Display, Error)]
#[non_exhaustive]
enum MiddlewareError {
    #[display(fmt = "missing route info for request")]
    MissingRouteInfo,
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
