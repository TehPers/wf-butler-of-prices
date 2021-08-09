use crate::models::Snowflake;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StageInstance {
    /// The id of this Stage instance.
    id: Snowflake,
    /// The guild id of the associated Stage channel.
    guild_id: Snowflake,
    /// The id of the associated Stage channel.
    channel_id: Snowflake,
    /// The topic of the Stage instance (1-120 characters).
    topic: String,
    /// The privacy level of the Stage instance.
    privacy_level: PrivacyLevel,
    /// Whether or not Stage Discovery is disabled.
    discoverable_disabled: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PrivacyLevel(pub u8);

impl PrivacyLevel {
    /// The Stage instance is visible publicly, such as on Stage Discovery.
    pub const PUBLIC: PrivacyLevel = PrivacyLevel(1);
    /// The Stage instance is visible to only guild members.
    pub const GUILD_ONLY: PrivacyLevel = PrivacyLevel(2);
}
