use axum::http::{HeaderMap, Method, Uri};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FunctionsHttpInput {
    #[serde(with = "http_serde::uri")]
    pub url: Uri,
    #[serde(with = "http_serde::method")]
    pub method: Method,
    #[serde(with = "http_serde::header_map")]
    pub headers: HeaderMap,
    pub query: HashMap<String, String>,
    pub params: HashMap<String, String>,
    pub body: String,
}
