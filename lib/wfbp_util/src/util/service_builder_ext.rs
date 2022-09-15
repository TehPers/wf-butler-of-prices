use crate::layers::TryMapRequestBodyLayer;
use tower::{layer::util::Stack, ServiceBuilder};

pub trait ServiceBuilderExt<L> {
    /// Try to apply a transformation to the request body.
    fn try_map_request_body<F>(
        self,
        f: F,
    ) -> ServiceBuilder<Stack<TryMapRequestBodyLayer<F>, L>>;
}

impl<L> ServiceBuilderExt<L> for ServiceBuilder<L> {
    fn try_map_request_body<F>(
        self,
        f: F,
    ) -> ServiceBuilder<Stack<TryMapRequestBodyLayer<F>, L>> {
        self.layer(TryMapRequestBodyLayer::new(f))
    }
}
