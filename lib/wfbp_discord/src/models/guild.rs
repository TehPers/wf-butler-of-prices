use crate::{
    models::{
        ApplicationId, Channel, ChannelId, Emoji, EmojiId, Permissions, Role,
        RoleId, StageInstance, Sticker, Timestamp, User, UserId, VoiceState,
    },
    snowflake_newtype,
};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

snowflake_newtype! {
    /// A unique ID for a guild.
    pub struct GuildId;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Guild {
    pub id: GuildId,
    pub name: String,
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_hash: Option<String>,
    pub splash: Option<String>,
    pub discovery_splash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<bool>,
    pub owner_id: UserId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permissions>,
    pub afk_channel_id: Option<ChannelId>,
    pub afk_timeout: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub widget_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub widget_channel_id: Option<UserId>,
    pub verification_level: VerificationLevel,
    pub default_message_notifications: MessageNotificationsLevel,
    pub explicit_content_filter: ExplicitContentFilterLevel,
    pub roles: Vec<Role>,
    pub emojis: Vec<Emoji>,
    pub features: Vec<GuildFeature>,
    pub mfa_level: MultiFactorLevel,
    pub application_id: Option<ApplicationId>,
    pub system_channel_id: Option<ChannelId>,
    pub system_channel_flags: SystemChannelFlags,
    pub rules_channel_id: Option<ChannelId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub joined_at: Option<Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub large: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unavailable: Option<bool>,
    pub member_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice_states: Option<Vec<VoiceState>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<GuildMember>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<Channel>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threads: Option<Vec<Channel>>,
    // TODO: is this needed?
    // #[serde(default, skip_serializing_if = "Option::is_none")]
    // pub presences: Option<Vec<PresenceUpdate>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_presences: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_members: Option<u32>,
    pub vanity_code_url: Option<String>,
    pub description: Option<String>,
    pub banner: Option<String>,
    pub premium_tier: PremiumTier,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub premium_subscription_count: Option<u8>,
    pub preferred_locale: String,
    pub public_updates_channel_id: Option<ChannelId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_video_channel_users: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approximate_member_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approximate_presence_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub welcome_screen: Option<WelcomeScreen>,
    pub nsfw_level: NsfwLevel,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_instances: Option<Vec<StageInstance>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stickers: Option<Vec<Sticker>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MessageNotificationsLevel(pub u8);

impl MessageNotificationsLevel {
    pub const ALL_MESSAGES: MessageNotificationsLevel =
        MessageNotificationsLevel(0);
    pub const ONLY_MENTIONS: MessageNotificationsLevel =
        MessageNotificationsLevel(1);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExplicitContentFilterLevel(pub u8);

impl ExplicitContentFilterLevel {
    /// Media content will not be scanned.
    pub const DISABLED: ExplicitContentFilterLevel =
        ExplicitContentFilterLevel(0);
    /// Media content sent by members without roles will be scanned.
    pub const MEMBERS_WITHOUT_ROLES: ExplicitContentFilterLevel =
        ExplicitContentFilterLevel(1);
    /// Media content sent by all members will be scanned.
    pub const ALL_MEMBERS: ExplicitContentFilterLevel =
        ExplicitContentFilterLevel(2);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MultiFactorLevel(pub u8);

impl MultiFactorLevel {
    /// Guild has no MFA/2FA requirement for moderation actions.
    pub const NONE: MultiFactorLevel = MultiFactorLevel(0);
    /// Guild has a 2FA requirement for moderation actions.
    pub const ELEVATED: MultiFactorLevel = MultiFactorLevel(1);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VerificationLevel(pub u8);

impl VerificationLevel {
    /// Unrestricted.
    pub const NONE: VerificationLevel = VerificationLevel(0);
    /// Must have verified email on account.
    pub const LOW: VerificationLevel = VerificationLevel(1);
    /// Must be registered on Discord for longer than 5 minutes.
    pub const MEDIUM: VerificationLevel = VerificationLevel(2);
    /// Must be a member of the server for longer than 10 minutes.
    pub const HIGH: VerificationLevel = VerificationLevel(3);
    /// Must have a verified phone number.
    pub const VERY_HIGH: VerificationLevel = VerificationLevel(4);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NsfwLevel(pub u8);

impl NsfwLevel {
    pub const DEFAULT: NsfwLevel = NsfwLevel(0);
    pub const EXPLICIT: NsfwLevel = NsfwLevel(1);
    pub const SAFE: NsfwLevel = NsfwLevel(2);
    pub const AGE_RESTRICTED: NsfwLevel = NsfwLevel(3);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PremiumTier(pub u8);

impl PremiumTier {
    /// Guild has not unlocked any Server Boost perks.
    pub const NONE: PremiumTier = PremiumTier(0);
    /// Guild has unlocked Server Boost level 1 perks.
    pub const TIER_1: PremiumTier = PremiumTier(1);
    /// Guild has unlocked Server Boost level 2 perks.
    pub const TIER_2: PremiumTier = PremiumTier(2);
    /// Guild has unlocked Server Boost level 3 perks.
    pub const TIER_3: PremiumTier = PremiumTier(3);
}

bitflags! {
    #[derive(Default, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct SystemChannelFlags: u32 {
        /// Suppress member join notifications.
        const SUPPRESS_JOIN_NOTIFICATIONS =	1 << 0;
        /// Suppress server boost notifications.
        const SUPPRESS_PREMIUM_SUBSCRIPTIONS =	1 << 1;
        /// Suppress server setup tips.
        const SUPPRESS_GUILD_REMINDER_NOTIFICATIONS =	1 << 2;
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GuildFeature(pub Cow<'static, str>);

impl GuildFeature {
    /// Guild has access to set an animated guild icon.
    pub const ANIMATED_ICON: GuildFeature =
        GuildFeature(Cow::Borrowed("ANIMATED_ICON"));
    /// Guild has access to set a guild banner image.
    pub const BANNER: GuildFeature = GuildFeature(Cow::Borrowed("BANNER"));
    /// Guild has access to use commerce features (i.e. create store channels).
    pub const COMMERCE: GuildFeature = GuildFeature(Cow::Borrowed("COMMERCE"));
    /// Guild can enable welcome screen, Membership Screening, stage channels
    /// and discovery, and receives community updates.
    pub const COMMUNITY: GuildFeature =
        GuildFeature(Cow::Borrowed("COMMUNITY"));
    /// Guild is able to be discovered in the directory.
    pub const DISCOVERABLE: GuildFeature =
        GuildFeature(Cow::Borrowed("DISCOVERABLE"));
    /// Guild is able to be featured in the directory.
    pub const FEATURABLE: GuildFeature =
        GuildFeature(Cow::Borrowed("FEATURABLE"));
    /// Guild has access to set an invite splash background.
    pub const INVITE_SPLASH: GuildFeature =
        GuildFeature(Cow::Borrowed("INVITE_SPLASH"));
    /// Guild has enabled Membership Screening.
    pub const MEMBER_VERIFICATION_GATE_ENABLED: GuildFeature =
        GuildFeature(Cow::Borrowed("MEMBER_VERIFICATION_GATE_ENABLED"));
    /// Guild has access to create news channels.
    pub const NEWS: GuildFeature = GuildFeature(Cow::Borrowed("NEWS"));
    /// Guild is partnered.
    pub const PARTNERED: GuildFeature =
        GuildFeature(Cow::Borrowed("PARTNERED"));
    /// Guild can be previewed before joining via Membership Screening or the
    /// directory.
    pub const PREVIEW_ENABLED: GuildFeature =
        GuildFeature(Cow::Borrowed("PREVIEW_ENABLED"));
    /// Guild has access to set a vanity URL.
    pub const VANITY_URL: GuildFeature =
        GuildFeature(Cow::Borrowed("VANITY_URL"));
    /// Guild is verified.
    pub const VERIFIED: GuildFeature = GuildFeature(Cow::Borrowed("VERIFIED"));
    /// Guild has access to set 384kbps bitrate in voice (previously VIP voice
    /// servers).
    pub const VIP_REGIONS: GuildFeature =
        GuildFeature(Cow::Borrowed("VIP_REGIONS"));
    /// Guild has enabled the welcome screen.
    pub const WELCOME_SCREEN_ENABLED: GuildFeature =
        GuildFeature(Cow::Borrowed("WELCOME_SCREEN_ENABLED"));
    /// Guild has enabled ticketed events.
    pub const TICKETED_EVENTS_ENABLED: GuildFeature =
        GuildFeature(Cow::Borrowed("TICKETED_EVENTS_ENABLED"));
    /// Guild has enabled monetization.
    pub const MONETIZATION_ENABLED: GuildFeature =
        GuildFeature(Cow::Borrowed("MONETIZATION_ENABLED"));
    /// Guild has increased custom sticker slots.
    pub const MORE_STICKERS: GuildFeature =
        GuildFeature(Cow::Borrowed("MORE_STICKERS"));
    /// Guild has access to the three day archive time for threads.
    pub const THREE_DAY_THREAD_ARCHIVE: GuildFeature =
        GuildFeature(Cow::Borrowed("THREE_DAY_THREAD_ARCHIVE"));
    /// Guild has access to the seven day archive time for threads.
    pub const SEVEN_DAY_THREAD_ARCHIVE: GuildFeature =
        GuildFeature(Cow::Borrowed("SEVEN_DAY_THREAD_ARCHIVE"));
    /// Guild has access to create private threads.
    pub const PRIVATE_THREADS: GuildFeature =
        GuildFeature(Cow::Borrowed("PRIVATE_THREADS"));
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuildPreview {
    /// Guild id.
    pub id: GuildId,
    /// Guild name (2-100 characters).
    pub name: String,
    /// Icon hash.
    pub icon: Option<String>,
    /// Splash hash.
    pub splash: Option<String>,
    /// Discovery splash hash.
    pub discovery_splash: Option<String>,
    /// Custom guild emojis.
    pub emojis: Vec<Emoji>,
    /// Enabled guild features.
    pub features: Vec<GuildFeature>,
    /// Approximate number of members in this guild.
    pub approximate_member_count: u32,
    /// Approximate number of online members in this guild.
    pub approximate_presence_count: u32,
    /// The description for the guild, if the guild is discoverable.
    pub description: Option<String>,
}

pub struct GuildWidget {
    pub enabled: bool,
    pub channel_id: Option<ChannelId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuildMember {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nick: Option<String>,
    pub roles: Vec<RoleId>,
    pub joined_at: Timestamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub premium_since: Option<Timestamp>,
    pub deaf: bool,
    pub mute: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permissions>,
}

snowflake_newtype! {
    /// A unique ID for an integration.
    pub struct IntegrationId;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Integration {
    /// Integration id.
    pub id: IntegrationId,
    /// Integration name.
    pub name: String,
    #[serde(rename = "type")]
    /// Integration type (twitch, youtube, or discord).
    pub kind: String,
    /// Is this integration enabled.
    pub enabled: bool,
    /// Is this integration syncing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub syncing: Option<bool>,
    /// Id that this integration uses for "subscribers".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role_id: Option<RoleId>,
    /// Whether emoticons should be synced for this integration (twitch only
    /// currently).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_emoticons: Option<bool>,
    /// The behavior of expiring subscribers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expire_behavior: Option<IntegrationExpireBehavior>,
    /// The grace period (in days) before expiring subscribers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expire_grace_period: Option<u32>,
    /// User for this integration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    /// Integration account information.
    pub account: IntegrationAccount,
    /// When this integration was last synced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synced_at: Option<Timestamp>,
    /// How many subscribers this integration has.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscriber_count: Option<u32>,
    /// Has this integration been revoked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked: Option<bool>,
    /// The bot/OAuth2 application for discord integrations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application: Option<IntegrationApplication>,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IntegrationExpireBehavior(pub u8);

impl IntegrationExpireBehavior {
    pub const REMOVE_ROLE: IntegrationExpireBehavior =
        IntegrationExpireBehavior(0);
    pub const KICK: IntegrationExpireBehavior = IntegrationExpireBehavior(1);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntegrationAccount {
    /// Id of the account.
    pub id: String,
    /// Name of the account.
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntegrationApplication {
    /// The id of the app.
    pub id: ApplicationId,
    /// The name of the app.
    pub name: String,
    /// The icon hash of the app.
    pub icon: Option<String>,
    /// The description of the app.
    pub description: String,
    /// The summary of the app.
    pub summary: String,
    /// The bot associated with this application.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bot: Option<User>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ban {
    /// The reason for the ban.
    pub reason: Option<String>,
    /// The banned user.
    pub user: User,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WelcomeScreen {
    /// 	The server description shown in the welcome screen.
    pub description: Option<String>,
    /// The channels shown in the welcome screen, up to 5.
    pub welcome_channels: Vec<WelcomeScreenChannel>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WelcomeScreenChannel {
    /// The channel's id.
    pub channel_id: ChannelId,
    /// The description shown for the channel.
    pub description: String,
    /// The emoji id, if the emoji is custom.
    pub emoji_id: Option<EmojiId>,
    /// The emoji name if custom, the unicode character if standard, or null if
    /// no emoji is set.
    pub emoji_name: Option<String>,
}
