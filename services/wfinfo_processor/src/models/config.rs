use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use wfinfo_discord::{middleware::ClientSecret, models::Snowflake};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub app_id: Snowflake,
    pub client_id: Snowflake,
    pub client_secret: ClientSecret,
    #[serde(rename = "functions_customhandler_port", default = "default_port")]
    pub port: u16,
}

fn default_port() -> u16 {
    3000
}
