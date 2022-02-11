use crate::models::{GuildMember, Snowflake, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceState {
    /// The guild id this voice state is for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    guild_id: Option<Snowflake>,
    /// The channel id this user is connected to.
    channel_id: Option<Snowflake>,
    /// The user id this voice state is for.
    user_id: Snowflake,
    /// The guild member this voice state is for.
    member: GuildMember,
    /// The session id for this voice state.
    session_id: String,
    /// Whether this user is deafened by the server.
    deaf: bool,
    /// Whether this user is muted by the server.
    mute: bool,
    /// Whether this user is locally deafened.
    self_deaf: bool,
    /// Whether this user is locally muted.
    self_mute: bool,
    /// Whether this user is streaming using "Go Live".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    self_stream: Option<bool>,
    /// Whether this user's camera is enabled.
    self_video: bool,
    /// Whether this user is muted by the current user.
    suppress: bool,
    /// The time at which the user requested to speak.
    request_to_speak_timestamp: Timestamp,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceRegion {
    /// Unique ID for the region.
    id: String,
    /// Name of the region.
    name: String,
    /// True if this is a vip-only server.
    vip: bool,
    /// True for a single server that is closest to the current user's client.
    optimal: bool,
    /// Whether this is a deprecated voice region (avoid switching to these).
    deprecated: bool,
    /// Whether this is a custom voice region (used for events/etc).
    custom: bool,
}
