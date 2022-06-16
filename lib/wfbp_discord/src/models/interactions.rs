use crate::{
    models::{
        AllowedMentions, ApplicationId, Attachment, AttachmentId, Channel,
        ChannelId, Embed, Emoji, GuildId, GuildMember, Message, MessageId,
        Role, RoleId, User, UserId,
    },
    serde_inner_enum, snowflake_newtype,
};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

serde_inner_enum! {
    #[derive(Clone, Debug)]
    pub enum Component = "type" {
        ActionRow = 1 {
            components: Vec<Component>,
        },
        Button = 2 {
            style: ButtonStyle,
            [?] label: Option<String>,
            [?] emoji: Option<Emoji>,
            [?] custom_id: Option<String>,
            [?] url: Option<String>,
            [?] disabled: Option<bool>,
        },
        SelectMenu = 3 {
            custom_id: String,
            options: Vec<SelectOption>,
            [?] placeholder: Option<String>,
            [?] min_values: Option<u8>,
            [?] max_values: Option<u8>,
            [?] disabled: Option<bool>,
        },
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ButtonStyle(pub u8);

impl ButtonStyle {
    pub const PRIMARY: ButtonStyle = ButtonStyle(1);
    pub const SECONDARY: ButtonStyle = ButtonStyle(2);
    pub const SUCCESS: ButtonStyle = ButtonStyle(3);
    pub const DANGER: ButtonStyle = ButtonStyle(4);
    pub const LINK: ButtonStyle = ButtonStyle(5);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelectOption {
    pub label: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emoji: Option<Emoji>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,
}

snowflake_newtype! {
    /// A unique ID for an application command.
    pub struct ApplicationCommandId;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplicationCommand {
    /// Unique id of the command.
    pub id: ApplicationCommandId,
    /// The type of the command.
    #[serde(flatten)]
    pub kind: ApplicationCommandType,
    /// Unique id of the parent application.
    pub application_id: ApplicationId,
    /// Guild id of the command, if not global.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    /// 1-32 lowercase character name matching `^[\w-]{1,32}$`.
    pub name: String,
    /// 1-100 character description.
    pub description: String,
    /// Whether the command is enabled by default when the app is added to a
    /// guild (default `true`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_permission: Option<bool>,
}

serde_inner_enum! {
    /// An application command type.
    #[derive(Clone, Debug)]
    pub enum ApplicationCommandType = "type" {
        /// Slash commands; a text-based command that shows up when a user types `/`.
        ChatInput = 1 {
            /// The parameters for the command.
            ///
            /// *Note: Required options must be listed before optional options.*
            options: Vec<ApplicationCommandOption>,
        },
        /// A UI-based command that shows up when you right click or tap on a user.
        User = 2,
        /// A UI-based command that shows up when you right click or tap on a message.
        Message = 3,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplicationCommandOption {
    /// 1-32 lowercase character name matching `^[\w-]{1,32}$`.
    pub name: String,
    /// 1-100 character description.
    pub description: String,
    #[serde(flatten)]
    pub kind: ApplicationCommandOptionType,
}

serde_inner_enum! {
    #[derive(Clone, Debug)]
    pub enum ApplicationCommandOptionType = "type" {
        SubCommand = 1 {
            /// Nested options.
            [?] options: Option<Vec<ApplicationCommandOption>>,
        },
        SubCommandGroup = 2 {
            /// Nested options.
            [?] options: Option<Vec<ApplicationCommandOption>>,
        },
        String = 3 {
            /// If the parameter is required or optional (default `false`).
            [?] required: Option<bool>,
            /// Choices for the user to pick from.
            [?] choices: Option<Vec<ApplicationCommandOptionChoice<String>>>,
        },
        /// Any integer between -2^53 and 2^53.
        Integer = 4 {
            /// If the parameter is required or optional (default `false`).
            [?] required: Option<bool>,
            /// Choices for the user to pick from.
            [?] choices: Option<Vec<ApplicationCommandOptionChoice<i64>>>,
        },
        Boolean = 5 {
            /// If the parameter is required or optional (default `false`).
            [?] required: Option<bool>,
        },
        User = 6 {
            /// If the parameter is required or optional (default `false`).
            [?] required: Option<bool>,
        },
        /// Includes all channel types + categories.
        Channel = 7 {
            /// If the parameter is required or optional (default `false`).
            [?] required: Option<bool>,
        },
        Role = 8 {
            /// If the parameter is required or optional (default `false`).
            [?] required: Option<bool>,
        },
        /// Includes users and roles.
        Mentionable = 9 {
            /// If the parameter is required or optional (default `false`).
            [?] required: Option<bool>,
        },
        /// Any double between -2^53 and 2^53
        Number = 10 {
            /// If the parameter is required or optional (default `false`).
            [?] required: Option<bool>,
            /// Choices for the user to pick from.
            [?] choices: Option<Vec<ApplicationCommandOptionChoice<f64>>>,
        },
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplicationCommandOptionChoice<T> {
    /// 1-100 character choice name.
    pub name: String,
    /// Value of the choice, up to 100 characters if string.
    pub value: T,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuildApplicationCommandPermissions {
    /// The id of the command.
    pub id: ApplicationCommandId,
    /// The id of the application the command belongs to.
    pub application_id: ApplicationId,
    /// The id of the guild.
    pub guild_id: GuildId,
    /// The permissions for the command in the guild.
    pub permissions: Vec<ApplicationCommandPermission>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplicationCommandPermission {
    /// The type of application command permission.
    #[serde(flatten)]
    pub kind: ApplicationCommandPermissionType,
    /// `true` to allow, `false` to disallow.
    pub permission: bool,
}

serde_inner_enum! {
    #[derive(Clone, Debug)]
    pub enum ApplicationCommandPermissionType = "type" {
        Role = 1 {
            /// The ID of the role.
            id: RoleId,
        },
        User = 2 {
            /// The ID of the user.
            id: UserId,
        },
    }
}

snowflake_newtype! {
    /// A unique ID for an interaction.
    pub struct InteractionId;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Interaction {
    /// Id of the interaction.
    pub id: InteractionId,
    /// Id of the application this interaction is for.
    pub application_id: ApplicationId,
    /// The type of the command.
    #[serde(flatten)]
    pub kind: InteractionType,
    /// The guild it was sent from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    /// The channel it was sent from.
    pub channel_id: ChannelId,
    /// Guild member data for the invoking user, including permissions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member: Option<GuildMember>,
    /// User object for the invoking user, if invoked in a DM.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    /// A continuation token for responding to the interaction.
    pub token: String,
    /// Auto-incrementing version identifier updated during substantial record
    /// changes.
    pub version: u8,
}

serde_inner_enum! {
    #[derive(Clone, Debug)]
    pub enum InteractionType = "type" {
        Ping = 1,
        ApplicationCommand = 2 {
            /// The command data payload.
            data: ApplicationCommandInteractionData,
            /// The selected language of the invoking user.
            locale: String,
        },
        MessageComponent = 3 {
            /// The message the component was attached to.
            message: Message,
            /// The type of the component.
            component_type: ComponentType,
            /// The selected language of the invoking user.
            locale: String,
        },
        Autocomplete = 4 {
            /// The selected language of the invoking user.
            locale: String,
        },
        ModalSubmit = 5 {
            /// The message the component was attached to.
            message: Message,
            // TODO: /// The command data payload.
            // data: ApplicationCommandInteractionData,
            /// The selected language of the invoking user.
            locale: String,
        },
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplicationCommandInteractionData {
    /// The ID of the invoked command.
    pub id: ApplicationCommandId,
    /// The name of the invoked command.
    pub name: String,
    /// The type of the invoked command.
    pub kind: ApplicationCommandInteractionDataType,
}

serde_inner_enum! {
    #[derive(Clone, Debug)]
    pub enum ApplicationCommandInteractionDataType = "type" {
        SlashCommand = 1 {
            /// Converted users + roles + channels.
            [?] resolved: Option<ApplicationCommandInteractionDataResolved>,
            /// The params + values from the user.
            [?] options: Option<Vec<ApplicationCommandInteractionDataOption>>,
        },
        User = 2 {
            /// ID the of user.
            target_id: UserId,
        },
        Message = 3 {
            /// ID the of message.
            target_id: MessageId,
        },
    }
}

serde_inner_enum! {
    #[derive(Clone, Debug)]
    pub enum ComponentType = "type" {
        ActionRow = 1,
        Button = 2 {
            /// The `custom_id` of the component.
            custom_id: String,
            /// Converted users + roles + channels.
            [?] resolved: Option<ApplicationCommandInteractionDataResolved>,
            /// The params + values from the user.
            [?] options: Option<Vec<ApplicationCommandInteractionDataOption>>,
        },
        SelectMenu = 3 {
            /// The `custom_id` of the component.
            custom_id: String,
            /// Converted users + roles + channels.
            [?] resolved: Option<ApplicationCommandInteractionDataResolved>,
            /// The params + values from the user.
            [?] options: Option<Vec<ApplicationCommandInteractionDataOption>>,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplicationCommandInteractionDataOption {
    /// The name of the parameter.
    pub name: String,
    #[serde(flatten)]
    pub value: ApplicationCommandInteractionDataOptionValue,
}

snowflake_newtype! {
    /// A unique ID for a user or role.
    pub struct MentionableId;
}

impl From<MentionableId> for UserId {
    fn from(id: MentionableId) -> Self {
        id.0.into()
    }
}

impl From<UserId> for MentionableId {
    fn from(id: UserId) -> Self {
        id.0.into()
    }
}

impl From<MentionableId> for RoleId {
    fn from(id: MentionableId) -> Self {
        id.0.into()
    }
}

impl From<RoleId> for MentionableId {
    fn from(id: RoleId) -> Self {
        id.0.into()
    }
}

serde_inner_enum! {
    #[derive(Clone, Debug)]
    pub enum ApplicationCommandInteractionDataOptionValue = "type" {
        SubCommand = 1 {
            [?] options: Option<Vec<ApplicationCommandInteractionDataOption>>,
        },
        SubCommandGroup = 2 {
            [?] options: Option<Vec<ApplicationCommandInteractionDataOption>>,
        },
        String = 3 {
            value: String,
        },
        Integer = 4 {
            value: i64,
        },
        Boolean = 5 {
            value: bool,
        },
        User = 6 {
            value: UserId,
        },
        Channel = 7 {
            value: ChannelId,
        },
        Role = 8 {
            value: RoleId,
        },
        Mentionable = 9 {
            value: MentionableId,
        },
        Number = 10 {
            value: f64,
        },
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ApplicationCommandInteractionDataResolved {
    /// The IDs and [User] objects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub users: Option<HashMap<UserId, User>>,
    /// The IDs and partial [GuildMember] objects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub members: Option<HashMap<UserId, GuildMember>>,
    /// The IDs and [Role] objects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roles: Option<HashMap<RoleId, Role>>,
    /// The IDs and partial [Channel] objects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channels: Option<HashMap<ChannelId, Channel>>,
    /// The IDs and partial [Message] objects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub messages: Option<HashMap<MessageId, Message>>,
    /// The IDs and [Attachment] objects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachments: Option<HashMap<AttachmentId, Attachment>>,
}

serde_inner_enum! {
    #[derive(Clone, Debug)]
    pub enum InteractionResponse = "type" {
        Pong = 1,
        ChannelMessageWithSource = 4 {
            data: InteractionApplicationCommandCallbackData,
        },
        DeferredChannelMessageWithSource = 5 {
            data: InteractionApplicationCommandCallbackData,
        },
        /// Only valid for component-based interactions.
        DeferredUpdateMessage = 6 {
            data: InteractionApplicationCommandCallbackData,
        },
        /// Only valid for component-based interactions.
        UpdateMessage = 7 {
            data: InteractionApplicationCommandCallbackData,
        },
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InteractionApplicationCommandCallbackData {
    /// Is the response TTS?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tts: Option<bool>,
    /// Message content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Supports up to 10 embeds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<Embed>>,
    /// Allowed mentions object.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_mentions: Option<AllowedMentions>,
    /// Interaction application command callback data flags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flags: Option<InteractionResponseDataFlags>,
    /// Message components.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<Component>>,
}

bitflags! {
    #[derive(Default, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct InteractionResponseDataFlags: u32 {
        /// Do not include any embeds when serializing this message.
        const SUPPRESS_EMBEDS = 1 << 2;
        /// Only the user receiving the message can see it.
        const EPHEMERAL = 1 << 6;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateApplicationCommand {
    /// 1-32 lowercase character name matching `^[\w-]{1,32}$`.
    pub name: String,
    /// Localization dictionary for the `name` field. Values follow the same
    /// restrictions as `name`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_localizations: Option<HashMap<String, String>>,
    /// 1-100 character description.
    pub description: String,
    /// Localization dictionary for the `description` field. Values follow the
    /// same restrictions as `description`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description_localizations: Option<HashMap<String, String>>,
    /// Whether the command is enabled by default when the app is added to a
    /// guild (default `true`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_permission: Option<bool>,
    /// Type of command.
    pub kind: CreateApplicationCommandType,
}

serde_inner_enum! {
    #[derive(Clone, Debug)]
    pub enum CreateApplicationCommandType = "type" {
        ChatInput = 1 {
            /// The parameters for the command.
            [?] options: Option<Vec<ApplicationCommandOption>>,
        },
        User = 2,
        Message = 3,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateGuildApplicationCommandPermissions {
    /// The permissions for the command in the guild.
    pub permissions: Vec<ApplicationCommandPermission>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchEditGuildApplicationCommandPermissions {
    /// The id of the command.
    pub id: ApplicationCommandId,
    /// The permissions for the command in the guild.
    pub permissions: Vec<ApplicationCommandPermission>,
}
