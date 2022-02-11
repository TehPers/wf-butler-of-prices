use anyhow::anyhow;
use futures::future::BoxFuture;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    task::{Context, Poll},
};
use tower::{Layer, Service};

/// For each request, limits the maximum number of times the request can be
/// executed.
#[derive(Clone, Debug)]
pub struct LimitLayer {
    max_tries: usize,
}

impl LimitLayer {
    pub fn new(max_tries: usize) -> Self {
        LimitLayer { max_tries }
    }
}

impl<Next> Layer<Next> for LimitLayer {
    type Service = LimitService<Next>;

    fn layer(&self, next: Next) -> Self::Service {
        LimitService {
            remaining_tries: Arc::new(AtomicUsize::new(self.max_tries)),
            next,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LimitService<Next> {
    remaining_tries: Arc<AtomicUsize>,
    next: Next,
}

impl<Req, Next> Service<Req> for LimitService<Next>
where
    Next: Service<Req>,
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
        let next_fut = self.next.call(req);
        let remaining_tries = self.remaining_tries.clone();

        Box::pin(async move {
            loop {
                let last_remaining = remaining_tries.load(Ordering::Acquire);
                let new_remaining = match last_remaining.checked_sub(1) {
                    Some(new_remaining) => new_remaining,
                    None => return Err(anyhow!("max tries reached"))?,
                };

                if remaining_tries
                    .compare_exchange_weak(
                        last_remaining,
                        new_remaining,
                        Ordering::Release,
                        Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    break next_fut.await;
                }
            }
        })
    }
}
