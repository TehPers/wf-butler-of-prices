use crate::{middleware::RestRequestValue, RequestError};
use async_trait::async_trait;
use reqwest::{Method, RequestBuilder, Response};

/// A route within the Discord REST API.
#[async_trait]
pub trait Route: Send + Sync + 'static {
    /// Additional route information.
    type Info: RestRequestValue;

    /// The type of response from the route.
    type Response;

    /// Gets additional route information
    fn info(&self) -> Self::Info;

    /// Creates an HTTP request to this route.
    ///
    /// `request_factory` accepts a HTTP method and URL path and creates a
    /// [`RequestBuilder`] from it.
    fn create_request<F>(&self, request_factory: F) -> RequestBuilder
    where
        F: for<'a> FnOnce(Method, &'a str) -> RequestBuilder;

    /// Maps the HTTP response to a usable response object.
    async fn map_response(
        &self,
        response: Response,
    ) -> Result<Self::Response, RequestError>;
}
