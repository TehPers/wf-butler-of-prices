use serde::{de, ser::SerializeStruct, Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RawHttpInput {
    pub url: String,
    pub method: String,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, Vec<String>>,
    pub params: HashMap<String, String>,
    pub body: String,
}

#[derive(Clone, Debug)]
pub struct HttpInput<T> {
    pub url: String,
    pub method: String,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, Vec<String>>,
    pub params: HashMap<String, String>,
    pub body: T,
}

impl<'de, T: 'static + for<'d> Deserialize<'d>> Deserialize<'de>
    for HttpInput<T>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let intermediate: RawHttpInput =
            Deserialize::deserialize(deserializer)?;

        Ok(HttpInput {
            url: intermediate.url,
            method: intermediate.method,
            query: intermediate.query,
            headers: intermediate.headers,
            params: intermediate.params,
            body: serde_json::from_str(&intermediate.body)
                .map_err(de::Error::custom)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct HttpOutput<T> {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: T,
}

impl<T: Serialize> Serialize for HttpOutput<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("HttpOutput", 3)?;
        s.serialize_field("StatusCode", &self.status_code)?;
        s.serialize_field("Headers", &self.headers)?;
        let body = serde_json::to_string(&self.body)
            .map_err(serde::ser::Error::custom)?;
        s.serialize_field("body", &body)?;
        s.end()
    }
}
