use downcast_rs::{impl_downcast, DowncastSync};
use dyn_clone::{clone_trait_object, DynClone};
use futures::{future::BoxFuture, FutureExt};
use reqwest::{RequestBuilder, Response};
use std::{
    any::TypeId,
    collections::HashMap,
    fmt::Debug,
    task::{Context, Poll},
};
use tower::Service;

#[derive(Clone, Debug, Default)]
pub struct ExecuteRequestService;

impl Service<RequestBuilder> for ExecuteRequestService {
    type Response = Response;
    type Error = reqwest::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: RequestBuilder) -> Self::Future {
        req.send().boxed()
    }
}

#[derive(Debug)]
pub struct RestRequestBuilder {
    inner: RequestBuilder,
    values: HashMap<TypeId, Box<dyn RestRequestValue>>,
}

impl RestRequestBuilder {
    #[inline]
    pub fn new(inner: &RequestBuilder) -> Option<Self> {
        inner.try_clone().map(|inner| RestRequestBuilder {
            inner,
            values: HashMap::new(),
        })
    }

    #[inline]
    pub fn request(&self) -> &RequestBuilder {
        &self.inner
    }

    #[inline]
    pub fn request_mut(&mut self) -> &mut RequestBuilder {
        &mut self.inner
    }

    #[inline]
    pub fn into_request(self) -> RequestBuilder {
        self.inner
    }

    #[inline]
    pub fn with_modified_request<F>(self, f: F) -> Self
    where
        F: FnOnce(RequestBuilder) -> RequestBuilder,
    {
        RestRequestBuilder {
            inner: f(self.inner),
            values: self.values,
        }
    }

    #[inline]
    pub fn insert<T: RestRequestValue>(&mut self, value: T) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.values
            .insert(type_id, Box::new(value))
            .and_then(|old| old.downcast().ok())
            .map(|x| *x)
    }

    #[inline]
    pub fn get<T: RestRequestValue>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.values.get(&type_id).and_then(|x| x.downcast_ref())
    }

    #[inline]
    pub fn get_mut<T: RestRequestValue>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.values.get_mut(&type_id).and_then(|x| x.downcast_mut())
    }

    #[inline]
    pub fn remove<T: RestRequestValue>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.values
            .remove(&type_id)
            .and_then(|old| old.downcast().ok())
            .map(|x| *x)
    }
}

impl Clone for RestRequestBuilder {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.try_clone().expect("error cloning inner request"),
            values: self.values.clone(),
        }
    }
}

impl From<RestRequestBuilder> for RequestBuilder {
    #[inline]
    fn from(req: RestRequestBuilder) -> Self {
        req.into_request()
    }
}

/// Automatically implemented for all types that can be stored in a
/// [`RestRequestBuilder`].
pub trait RestRequestValue: DynClone + DowncastSync + Debug {}
impl<T: DynClone + DowncastSync + Debug> RestRequestValue for T {}

clone_trait_object!(RestRequestValue);
impl_downcast!(sync RestRequestValue);
