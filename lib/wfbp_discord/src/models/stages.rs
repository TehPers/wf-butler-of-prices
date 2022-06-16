use crate::{
    models::{ChannelId, GuildId},
    snowflake_newtype,
};
use serde::{Deserialize, Serialize};

snowflake_newtype! {
    /// A unique ID for a stage.
    pub struct StageId;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StageInstance {
    /// The id of this Stage instance.
    pub id: StageId,
    /// The guild id of the associated Stage channel.
    pub guild_id: GuildId,
    /// The id of the associated Stage channel.
    pub channel_id: ChannelId,
    /// The topic of the Stage instance (1-120 characters).
    pub topic: String,
    /// The privacy level of the Stage instance.
    pub privacy_level: PrivacyLevel,
    /// Whether or not Stage Discovery is disabled.
    pub discoverable_disabled: bool,
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
