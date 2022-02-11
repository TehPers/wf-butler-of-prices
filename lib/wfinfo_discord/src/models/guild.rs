use crate::models::{
    Channel, Emoji, Permissions, Role, Snowflake, StageInstance, Sticker,
    Timestamp, User, VoiceState,
};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Guild {
    id: Snowflake,
    name: String,
    icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    icon_hash: Option<String>,
    splash: Option<String>,
    discovery_splash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner: Option<bool>,
    owner_id: Snowflake,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    permissions: Option<Permissions>,
    afk_channel_id: Option<Snowflake>,
    afk_timeout: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    widget_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    widget_channel_id: Option<Snowflake>,
    verification_level: VerificationLevel,
    default_message_notifications: MessageNotificationsLevel,
    explicit_content_filter: ExplicitContentFilterLevel,
    roles: Vec<Role>,
    emojis: Vec<Emoji>,
    features: Vec<GuildFeature>,
    mfa_level: MultiFactorLevel,
    application_id: Option<Snowflake>,
    system_channel_id: Option<Snowflake>,
    system_channel_flags: SystemChannelFlags,
    rules_channel_id: Option<Snowflake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    joined_at: Option<Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    large: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    unavailable: Option<bool>,
    member_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    voice_states: Option<Vec<VoiceState>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    members: Option<Vec<GuildMember>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    channels: Option<Vec<Channel>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    threads: Option<Vec<Channel>>,
    // TODO: is this needed?
    // #[serde(default, skip_serializing_if = "Option::is_none")]
    // presences: Option<Vec<PresenceUpdate>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    max_presences: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    max_members: Option<u32>,
    vanity_code_url: Option<String>,
    description: Option<String>,
    banner: Option<String>,
    premium_tier: PremiumTier,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    premium_subscription_count: Option<u8>,
    preferred_locale: String,
    public_updates_channel_id: Option<Snowflake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    max_video_channel_users: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    approximate_member_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    approximate_presence_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    welcome_screen: Option<WelcomeScreen>,
    nsfw_level: NsfwLevel,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stage_instances: Option<Vec<StageInstance>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stickers: Option<Vec<Sticker>>,
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
    pub id: Snowflake,
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
    pub channel_id: Option<Snowflake>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuildMember {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    user: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    nick: Option<String>,
    roles: Vec<Snowflake>,
    joined_at: Timestamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    premium_since: Option<Timestamp>,
    deaf: bool,
    mute: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pending: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    permissions: Option<Permissions>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Integration {
    /// Integration id.
    id: Snowflake,
    /// Integration name.
    name: String,
    #[serde(rename = "type")]
    /// Integration type (twitch, youtube, or discord).
    kind: String,
    /// Is this integration enabled.
    enabled: bool,
    /// Is this integration syncing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    syncing: Option<bool>,
    /// Id that this integration uses for "subscribers".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    role_id: Option<Snowflake>,
    /// Whether emoticons should be synced for this integration (twitch only
    /// currently).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    enable_emoticons: Option<bool>,
    /// The behavior of expiring subscribers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expire_behavior: Option<IntegrationExpireBehavior>,
    /// The grace period (in days) before expiring subscribers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expire_grace_period: Option<u32>,
    /// User for this integration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    user: Option<User>,
    /// Integration account information.
    account: IntegrationAccount,
    /// When this integration was last synced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    synced_at: Option<Timestamp>,
    /// How many subscribers this integration has.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    subscriber_count: Option<u32>,
    /// Has this integration been revoked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    revoked: Option<bool>,
    /// The bot/OAuth2 application for discord integrations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    application: Option<IntegrationApplication>,
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
    id: String,
    /// Name of the account.
    name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntegrationApplication {
    /// The id of the app.
    id: Snowflake,
    /// The name of the app.
    name: String,
    /// The icon hash of the app.
    icon: Option<String>,
    /// The description of the app.
    description: String,
    /// The summary of the app.
    summary: String,
    /// The bot associated with this application.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bot: Option<User>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ban {
    /// The reason for the ban.
    reason: Option<String>,
    /// The banned user.
    user: User,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WelcomeScreen {
    /// 	The server description shown in the welcome screen.
    description: Option<String>,
    /// The channels shown in the welcome screen, up to 5.
    welcome_channels: Vec<WelcomeScreenChannel>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WelcomeScreenChannel {
    /// The channel's id.
    channel_id: Snowflake,
    /// The description shown for the channel.
    description: String,
    /// The emoji id, if the emoji is custom.
    emoji_id: Option<Snowflake>,
    /// The emoji name if custom, the unicode character if standard, or null if
    /// no emoji is set.
    emoji_name: Option<String>,
}
