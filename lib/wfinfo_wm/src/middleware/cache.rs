use anyhow::Context as _;
use async_trait::async_trait;
use futures::{future::BoxFuture, FutureExt};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::Debug,
    future::Future,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tower::{Layer, Service};
use wfinfo_http::Route;

#[derive(Debug, Default)]
pub struct CacheLayer<S> {
    storage: Arc<S>,
}

impl<S> CacheLayer<S> {
    pub fn new(storage: S) -> Self {
        CacheLayer {
            storage: Arc::new(storage),
        }
    }
}

impl<S> Clone for CacheLayer<S> {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
        }
    }
}

impl<S, Next> Layer<Next> for CacheLayer<S> {
    type Service = CacheService<S, Next>;

    fn layer(&self, next: Next) -> Self::Service {
        CacheService {
            storage: self.storage.clone(),
            next,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CacheService<S, Next> {
    storage: Arc<S>,
    next: Next,
}

impl<S, Req, Next> Service<Req> for CacheService<S, Next>
where
    S: CacheStorage,
    Req: Route,
    Req::Info: AsCacheInfo,
    Next: Service<Req>,
    Next::Response: Serialize + DeserializeOwned,
    Next::Error: From<anyhow::Error>,
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
        let info = req.info();
        let next_fut = self.next.call(req);
        let storage = self.storage.clone();
        let (key, expiry_secs) = match info.cache_info() {
            Some(cache_info) => {
                (cache_info.bucket.to_string(), cache_info.expiry_secs)
            }
            None => return next_fut.boxed(),
        };

        Box::pin(async move {
            let val =
                storage.get_or_insert(&key, expiry_secs, next_fut).await?;
            Ok(val)
        })
    }
}

pub trait AsCacheInfo {
    fn cache_info(&self) -> Option<CacheInfo<'_>>;
}

#[derive(Clone, Debug)]
pub struct CacheInfo<'a> {
    pub bucket: Cow<'a, str>,
    pub expiry_secs: u64,
}

impl<'a> AsCacheInfo for CacheInfo<'a> {
    fn cache_info(&self) -> Option<CacheInfo<'_>> {
        Some(self.clone())
    }
}

impl<'a> AsCacheInfo for Option<CacheInfo<'a>> {
    fn cache_info(&self) -> Option<CacheInfo<'_>> {
        self.clone()
    }
}

#[async_trait]
pub trait CacheStorage: Send + Sync + 'static {
    async fn get_or_insert<T, E, F>(
        &self,
        key: &str,
        expiry_secs: u64,
        f: F,
    ) -> Result<T, E>
    where
        T: Serialize + DeserializeOwned,
        E: From<anyhow::Error>,
        F: Send + Future<Output = Result<T, E>>;
}

#[async_trait]
impl CacheStorage for RwLock<HashMap<String, (String, Instant)>> {
    async fn get_or_insert<T, E, F>(
        &self,
        key: &str,
        expiry_secs: u64,
        f: F,
    ) -> Result<T, E>
    where
        T: Serialize + DeserializeOwned,
        E: From<anyhow::Error>,
        F: Send + Future<Output = Result<T, E>>,
    {
        // Fast path - cache hit
        let guard = self.read().await;
        if let Some(value) = guard.get(key) {
            let now = Instant::now();
            if now < value.1 {
                let value = serde_json::from_str(&value.0)
                    .context("error deserializing cached value")?;
                return Ok(value);
            }
        }
        drop(guard);

        // Slow path - cache miss
        let mut guard = self.write().await;
        let value = f.await?;
        let serialized = serde_json::to_string(&value)
            .context("error serializing value for cache")?;
        guard.insert(
            key.into(),
            (
                serialized,
                Instant::now() + Duration::from_secs(expiry_secs),
            ),
        );
        Ok(value)
    }
}

pub type LocalCacheStorage = RwLock<HashMap<String, (String, Instant)>>;
