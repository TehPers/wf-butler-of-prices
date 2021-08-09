use crate::{
    models::{
        Application, Component, Emoji, GuildMember, Nonce, Permissions,
        Snowflake, Sticker, StickerItem, Timestamp, User,
    },
    serde_inner_enum,
};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Channel {
    pub id: Snowflake,
    #[serde(rename = "type")]
    pub kind: ChannelType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<Snowflake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission_overwrites: Option<Vec<Overwrite>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_message_id: Option<Snowflake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_limit_per_user: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipients: Option<Vec<User>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<Snowflake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_id: Option<Snowflake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<Snowflake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_pin_timestamp: Option<Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rtc_region: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_quality_mode: Option<VideoQualityMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_metadata: Option<ThreadMetadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member: Option<ThreadMember>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_auto_archive_duration: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permissions>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ChannelType(pub u8);

impl ChannelType {
    /// A text channel within a server.
    pub const GUILD_TEXT: ChannelType = ChannelType(0);
    /// A direct message between users.
    pub const DM: ChannelType = ChannelType(1);
    /// A voice channel within a server.
    pub const GUILD_VOICE: ChannelType = ChannelType(2);
    /// A direct message between multiple users.
    pub const GROUP_DM: ChannelType = ChannelType(3);
    /// An organizational category that contains up to 50 channels.
    pub const GUILD_CATEGORY: ChannelType = ChannelType(4);
    /// A channel that users can follow and crosspost into their own server.
    pub const GUILD_NEWS: ChannelType = ChannelType(5);
    /// A channel in which game developers can sell their game on Discord.
    pub const GUILD_STORE: ChannelType = ChannelType(6);
    /// A temporary sub-channel within a GUILD_NEWS channel.
    pub const GUILD_NEWS_THREAD: ChannelType = ChannelType(10);
    /// A temporary sub-channel within a GUILD_TEXT channel.
    pub const GUILD_PUBLIC_THREAD: ChannelType = ChannelType(11);
    /// A temporary sub-channel within a GUILD_TEXT channel that is only viewable by those invited and those with the MANAGE_THREADS permission.
    pub const GUILD_PRIVATE_THREAD: ChannelType = ChannelType(12);
    /// A voice channel for hosting events with an audience.
    pub const GUILD_STAGE_VOICE: ChannelType = ChannelType(13);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VideoQualityMode(pub u8);

impl VideoQualityMode {
    /// Discord chooses the quality for optimal performance.
    pub const AUTO: VideoQualityMode = VideoQualityMode(1);
    /// 720p.
    pub const FULL: VideoQualityMode = VideoQualityMode(2);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: Snowflake,
    pub channel_id: Snowflake,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<Snowflake>,
    /// Only a valid user when generated by a user, webhooks are invalid users
    pub author: User,
    /// Only used in responses
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member: Option<GuildMember>,
    pub content: String,
    pub timestamp: Timestamp,
    pub edited_timestamp: Option<Timestamp>,
    pub tts: bool,
    pub mention_everyone: bool,
    // TODO: verify this type
    pub mentions: Vec<User>,
    pub mention_roles: Vec<Snowflake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mention_channels: Option<Vec<ChannelMention>>,
    pub attachments: Vec<Attachment>,
    pub embeds: Vec<Embed>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reactions: Option<Vec<Reaction>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<Nonce>,
    pub pinned: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webhook_id: Option<Snowflake>,
    #[serde(flatten)]
    pub kind: MessageType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity: Option<MessageActivity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application: Option<Application>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_id: Option<Snowflake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_reference: Option<MessageReference>,
    pub flags: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread: Option<Channel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<Component>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sticker_items: Option<Vec<StickerItem>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stickers: Option<Vec<Sticker>>,
}

serde_inner_enum! {
    #[derive(Clone, Debug)]
    pub enum MessageType = "type" {
        Default = 0,
        RecipientAdd = 1,
        RecipientRemove = 2,
        Call = 3,
        ChannelNameChange = 4,
        ChannelIconChange = 5,
        ChannelPinnedMessage = 6,
        GuildMemberJoin = 7,
        UserPremiumGuildSubscription = 8,
        UserPremiumGuildSubscriptionTier1 = 9,
        UserPremiumGuildSubscriptionTier2 = 10,
        UserPremiumGuildSubscriptionTier3 = 11,
        ChannelFollowAdd = 12,
        GuildDiscoveryDisqualified = 14,
        GuildDiscoveryRequalified = 15,
        GuildDiscoveryGracePeriodInitialWarning = 16,
        GuildDiscoveryGracePeriodFinalWarning = 17,
        ThreadCreated = 18,
        /// Only in v8
        Reply = 19 {
            [?] referenced_message: Option<Box<Message>>,
        },
        /// Only in v8
        ApplicationCommand = 20,
        /// Only in v9
        ThreadStarterMessage = 21 {
            [?] referenced_message: Option<Box<Message>>,
        },
        GuildInviteReminder = 22,
    }
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::Default
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageActivity {
    #[serde(rename = "type")]
    pub kind: MessageActivityKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub party_id: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MessageActivityKind(pub u8);

impl MessageActivityKind {
    pub const JOIN: MessageActivityKind = MessageActivityKind(1);
    pub const SPECTATE: MessageActivityKind = MessageActivityKind(2);
    pub const LISTEN: MessageActivityKind = MessageActivityKind(3);
    pub const JOIN_REQUEST: MessageActivityKind = MessageActivityKind(5);
}

bitflags! {
    #[derive(Default, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct MessageFlags: u32 {
        /// This message has been published to subscribed channels.
        const CROSSPOSTED = 1 << 0;
        /// This message originated from a message in another channel.
        const IS_CROSSPOST = 1 << 1;
        /// Do not include any embeds when serializing this message.
        const SUPPRESS_EMBEDS = 1 << 2;
        /// The source message for this crosspost has been deleted.
        const SOURCE_MESSAGE_DELETED = 1 << 3;
        /// This message came from the urgent message system.
        const URGENT = 1 << 4;
        /// This message has an associated thread, with the same ID as the message.
        const HAS_THREAD = 1 << 5;
        /// This message is only visible to the user who invoked the interaction.
        const EPHEMERAL = 1 << 6;
        /// This message is an interaction response and the bot is "thinking".
        const LOADING =  1 << 7;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageReference {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<Snowflake>,
    /// Optional when creating reply, always avaiable from response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<Snowflake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<Snowflake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fail_if_not_exists: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FollowedChannel {
    /// Source channel ID.
    pub channel_id: Snowflake,
    /// Created target webhook ID.
    pub webhook_id: Snowflake,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reaction {
    pub count: u32,
    pub me: bool,
    pub emoji: Emoji,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Overwrite {
    /// Role or user ID.
    pub id: Snowflake,
    /// Either 0 (role) or 1 (member).
    #[serde(rename = "type")]
    pub kind: OverwriteType,
    /// Permission bit set.
    pub allow: String,
    /// Permission bit set.
    pub deny: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OverwriteType(pub u8);

impl OverwriteType {
    pub const ROLE: OverwriteType = OverwriteType(0);
    pub const MEMBER: OverwriteType = OverwriteType(1);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThreadMetadata {
    /// Whether the thread is archived.
    pub archived: bool,
    /// Duration in minutes to automatically archive the thread after recent activity.
    pub auto_archive_duration: u32,
    /// Timestamp when the thread's archive status was last changed, used for calculating recent activity.
    pub archive_timestamp: Timestamp,
    /// Whether the thread is locked; when a thread is locked, only users with MANAGE_THREADS can unarchive it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThreadMember {
    /// The ID of the thread.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Snowflake>,
    /// The ID of the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Snowflake>,
    /// The time the current user last joined the thread.
    pub join_timestamp: Timestamp,
    /// Any user-thread settings.
    pub flags: ThreadMemberFlags,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ThreadMemberFlags(pub u32);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Embed {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub footer: Option<EmbedFooter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<EmbedImage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<EmbedThumbnail>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video: Option<EmbedVideo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<EmbedProvider>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<EmbedAuthor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<EmbedField>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbedThumbnail {
    /// Source url of thumbnail (only supports http(s) and attachments).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// A proxied url of the thumbnail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy_url: Option<String>,
    /// Height of thumbnail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// Width of thumbnail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbedVideo {
    /// Source url of video.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// A proxied url of the video.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy_url: Option<String>,
    /// Height of video.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// Width of video.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbedImage {
    /// Source url of image (only supports http(s) and attachments).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// A proxied url of the image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy_url: Option<String>,
    /// Height of image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// Width of image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbedProvider {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbedAuthor {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy_icon_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbedFooter {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy_icon_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbedField {
    pub name: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attachment {
    pub id: Snowflake,
    pub filename: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// Size of the file in bytes.
    pub size: usize,
    pub url: String,
    pub proxy_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelMention {
    pub id: Snowflake,
    pub guild_id: Snowflake,
    #[serde(rename = "type")]
    pub kind: ChannelType,
    pub name: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CreateMessage {
    /// The message contents (up to 2000 characters).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// `true` if this is a TTS message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tts: Option<bool>,
    // TODO
    // /// The contents of the file being sent.
    // file: file contents,
    /// Embedded rich content (up to 6000 characters).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<Embed>>,
    /// JSON encoded body of non-file params.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_json: Option<String>,
    /// Allowed mentions for the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_mentions: Option<AllowedMentions>,
    /// Include to make your message a reply.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_reference: Option<MessageReference>,
    /// The components to include with the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<Component>>,
    /// IDs of up to 3 stickers in the server to send in the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sticker_ids: Option<Vec<Snowflake>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AllowedMentions {
    /// An array of allowed mention types to parse from the content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse: Option<Vec<AllowedMentionType>>,
    /// Array of role_ids to mention (Max size of 100).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<Snowflake>>,
    /// Array of user_ids to mention (Max size of 100).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<Snowflake>>,
    /// For replies, whether to mention the author of the message being replied to (default `false`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replied_user: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AllowedMentionType(pub Cow<'static, str>);

impl AllowedMentionType {
    /// Controls role mentions.
    pub const ROLE: AllowedMentionType =
        AllowedMentionType(Cow::Borrowed("roles"));
    /// Controls user mentions.
    pub const USER: AllowedMentionType =
        AllowedMentionType(Cow::Borrowed("users"));
    /// Controls @everyone and @here mentions.
    pub const EVERYONE: AllowedMentionType =
        AllowedMentionType(Cow::Borrowed("everyone"));
}
