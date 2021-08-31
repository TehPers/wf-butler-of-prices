use serde::{Deserialize, Serialize};
use wfinfo_discord::models::Snowflake;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub app_id: Snowflake,
    pub client_id: Snowflake,
    pub client_secret: String,
    #[serde(rename = "functions_customhandler_port", default = "default_port")]
    pub port: u16,
}

fn default_port() -> u16 {
    3000
}
