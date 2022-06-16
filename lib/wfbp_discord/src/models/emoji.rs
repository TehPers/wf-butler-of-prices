use crate::{
    models::{RoleId, User},
    snowflake_newtype,
};
use serde::{Deserialize, Serialize};

snowflake_newtype! {
    /// A unique ID for an emoji.
    pub struct EmojiId;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Emoji {
    pub id: Option<EmojiId>,
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<RoleId>>,
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
