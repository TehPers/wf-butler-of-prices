use serde::{Deserialize, Serialize};
use wfinfo_lib::models::Snowflake;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub app_id: Snowflake,
    pub app_secret: String,
    #[serde(default)]
    pub ignore_signature: bool,
    pub public_key: String,
    #[serde(rename = "functions_customhandler_port", default = "default_port")]
    pub port: u16,
}

fn default_port() -> u16 {
    3000
}
