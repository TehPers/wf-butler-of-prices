use crate::models::Snowflake;
use bitflags::bitflags;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;

bitflags! {
    #[derive(Default, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct Permissions: u64 {
        const CREATE_INSTANT_INVITE = 1 << 0;
        const KICK_MEMBERS = 1 << 1;
        const BAN_MEMBERS = 1 << 2;
        const ADMINISTRATOR = 1 << 3;
        const MANAGE_CHANNELS = 1 << 4;
        const MANAGE_GUILD = 1 << 5;
        const ADD_REACTIONS = 1 << 6;
        const VIEW_AUDIT_LOG = 1 << 7;
        const PRIORITY_SPEAKER = 1 << 8;
        const STREAM = 1 << 9;
        const VIEW_CHANNEL = 1 << 10;
        const SEND_MESSAGES = 1 << 11;
        const SEND_TTS_MESSAGES = 1 << 12;
        const MANAGE_MESSAGES = 1 << 13;
        const EMBED_LINKS = 1 << 14;
        const ATTACH_FILES = 1 << 15;
        const READ_MESSAGE_HISTORY = 1 << 16;
        const MENTION_EVERYONE = 1 << 17;
        const USE_EXTERNAL_EMOJIS = 1 << 18;
        const VIEW_GUILD_INSIGHTS = 1 << 19;
        const CONNECT = 1 << 20;
        const SPEAK = 1 << 21;
        const MUTE_MEMBERS = 1 << 22;
        const DEAFEN_MEMBERS = 1 << 23;
        const MOVE_MEMBERS = 1 << 24;
        const USE_VAD = 1 << 25;
        const CHANGE_NICKNAME = 1 << 26;
        const MANAGE_NICKNAMES = 1 << 27;
        const MANAGE_ROLES = 1 << 28;
        const MANAGE_WEBHOOKS = 1 << 29;
        const MANAGE_EMOJIS_AND_STICKERS = 1 << 30;
        const USE_SLASH_COMMANDS = 1 << 31;
        const REQUEST_TO_SPEAK = 1 << 32;
        const MANAGE_THREADS = 1 << 34;
        const USE_PUBLIC_THREADS = 1 << 35;
        const USE_PRIVATE_THREADS = 1 << 36;
        const USE_EXTERNAL_STICKERS = 1 << 37;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Role {
    pub id: Snowflake,
    pub name: String,
    pub color: u32,
    pub hoist: bool,
    pub position: u16,
    pub permissions: Permissions,
    pub managed: bool,
    pub mentionable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<RoleTags>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoleTags {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bot_id: Option<Snowflake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integration_id: Option<Snowflake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub premium_subscriber: Option<PremiumSubscriber>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub struct PremiumSubscriber;

impl Serialize for PremiumSubscriber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
    }
}

impl<'de> Deserialize<'de> for PremiumSubscriber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PremiumSubscriberVisitor;
        impl<'de> Visitor<'de> for PremiumSubscriberVisitor {
            type Value = PremiumSubscriber;

            fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
                write!(f, "a None value")
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PremiumSubscriber)
            }
        }

        deserializer.deserialize_option(PremiumSubscriberVisitor)
    }
}
