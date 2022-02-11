use crate::models::{Snowflake, User};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Emoji {
    pub id: Option<Snowflake>,
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<Snowflake>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_colons: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub managed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animated: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub available: Option<bool>,
}
