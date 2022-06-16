use crate::{models::Integration, snowflake_newtype};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

snowflake_newtype! {
    /// A unique ID for a user.
    pub struct UserId;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    id: UserId,
    username: String,
    discriminator: String,
    avatar: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bot: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    system: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mfa_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    locale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    verified: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    flags: Option<UserFlags>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    premium_type: Option<PremiumType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    public_flags: Option<UserFlags>,
}

bitflags! {
    #[derive(Default, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct UserFlags: u32 {
        const DISCORD_EMPLOYEE = 1 << 0;
        const PARTNERED_SERVER_OWNER = 1 << 1;
        const HYPESQUAD_EVENTS = 1 << 2;
        const BUG_HUNTER_LEVEL_1 = 1 << 3;
        const HOUSE_BRAVERY = 1 << 6;
        const HOUSE_BRILLIANCE = 1 << 7;
        const HOUSE_BALANCE = 1 << 8;
        const EARLY_SUPPORTER = 1 << 9;
        const TEAM_USER = 1 << 10;
        const BUG_HUNTER_LEVEL_2 = 1 << 14;
        const VERIFIED_BOT = 1 << 16;
        const EARLY_VERIFIED_BOT_DEVELOPER = 1 << 17;
        const DISCORD_CERTIFIED_MODERATOR = 1 << 18;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PremiumType(pub u16);

impl PremiumType {
    pub const NONE: PremiumType = PremiumType(0);
    pub const NITRO_CLASSIC: PremiumType = PremiumType(1);
    pub const NITRO: PremiumType = PremiumType(2);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Connection {
    id: String,
    name: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    revoked: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    integrations: Option<Vec<Integration>>,
    verified: bool,
    friend_sync: bool,
    show_activity: bool,
    visibility: VisibilityType,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VisibilityType(pub u8);

impl VisibilityType {
    pub const NONE: VisibilityType = VisibilityType(0);
    pub const EVERYONE: VisibilityType = VisibilityType(1);
}
