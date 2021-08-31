use reqwest::{Method, RequestBuilder};
use std::fmt::Display;

/// A route within the Discord REST API.
///
/// The route's [`Display`] implementation determines the path in the Discord
/// REST API.
pub trait Route: Display + Send {
    /// Additional route information
    type Info;

    /// Gets additional route information
    fn info(&self) -> Self::Info;

    /// Creates an HTTP request to this route.
    ///
    /// `request_factory` accepts a HTTP method and URL path and creates a
    /// [`RequestBuilder`] from it.
    fn create_request<F>(&self, request_factory: F) -> RequestBuilder
    where
        F: for<'a> FnOnce(Method, &'a str) -> RequestBuilder;
}
