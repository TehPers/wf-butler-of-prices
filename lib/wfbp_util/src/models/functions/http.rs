use axum::http::{HeaderMap, Method, StatusCode, Uri};
use derive_more::From;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HttpInput {
    #[serde(with = "http_serde::uri")]
    pub url: Uri,
    #[serde(with = "http_serde::method")]
    pub method: Method,
    #[serde(with = "http_serde::header_map")]
    pub headers: HeaderMap,
    pub query: FunctionsHttpQueryParams,
    pub params: FunctionsHttpRouteParams,
    pub body: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HttpOutput {
    pub body: String,
    #[serde(with = "http_serde::status_code")]
    pub status: StatusCode,
    #[serde(with = "http_serde::header_map")]
    pub headers: HeaderMap,
}

#[derive(Clone, Debug, Default, From, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FunctionsHttpRouteParams(pub HashMap<String, String>);

#[derive(Clone, Debug, Default, From, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FunctionsHttpQueryParams(pub HashMap<String, String>);
