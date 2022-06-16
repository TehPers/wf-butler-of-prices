use crate::models::{
    ChannelId, GuildId, GuildMember, Timestamp, UserId,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceState {
    /// The guild id this voice state is for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    /// The channel id this user is connected to.
    pub channel_id: Option<ChannelId>,
    /// The user id this voice state is for.
    pub user_id: UserId,
    /// The guild member this voice state is for.
    pub member: GuildMember,
    /// The session id for this voice state.
    pub session_id: String,
    /// Whether this user is deafened by the server.
    pub deaf: bool,
    /// Whether this user is muted by the server.
    pub mute: bool,
    /// Whether this user is locally deafened.
    pub self_deaf: bool,
    /// Whether this user is locally muted.
    pub self_mute: bool,
    /// Whether this user is streaming using "Go Live".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub self_stream: Option<bool>,
    /// Whether this user's camera is enabled.
    pub self_video: bool,
    /// Whether this user is muted by the current user.
    pub suppress: bool,
    /// The time at which the user requested to speak.
    pub request_to_speak_timestamp: Timestamp,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceRegion {
    /// Unique ID for the region.
    pub id: String,
    /// Name of the region.
    pub name: String,
    /// True if this is a vip-only server.
    pub vip: bool,
    /// True for a single server that is closest to the current user's client.
    pub optimal: bool,
    /// Whether this is a deprecated voice region (avoid switching to these).
    pub deprecated: bool,
    /// Whether this is a custom voice region (used for events/etc).
    pub custom: bool,
}
