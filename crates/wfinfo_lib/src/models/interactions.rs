use crate::{
    models::{
        AllowedMentions, Channel, Embed, Emoji, GuildMember, Message, Role,
        Snowflake, User,
    },
    serde_inner_enum,
};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TODO: internally tagged enum instead, if possible
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplicationCommand {
    /// Unique id of the command.
    pub id: Snowflake,
    /// Unique id of the parent application.
    pub application_id: Snowflake,
    /// Guild id of the command, if not global.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<Snowflake>,
    /// 1-32 lowercase character name matching `^[\w-]{1,32}$`.
    pub name: String,
    /// 1-100 character description.
    pub description: String,
    /// The parameters for the command.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<ApplicationCommandOption>>,
    /// Whether the command is enabled by default when the app is added to a guild (default `true`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_permission: Option<bool>,
}

serde_inner_enum! {
    #[derive(Clone, Debug)]
    pub enum ApplicationCommandOption = "type" {
        SubCommand = 1 {
            /// 1-32 lowercase character name matching `^[\w-]{1,32}$`.
            name: String,
            /// 1-100 character description.
            description: String,
            /// Nested options.
            [?] options: Option<Vec<ApplicationCommandOption>>,
        },
        SubCommandGroup = 2 {
            /// 1-32 lowercase character name matching `^[\w-]{1,32}$`.
            name: String,
            /// 1-100 character description.
            description: String,
            /// Nested options.
            [?] options: Option<Vec<ApplicationCommandOption>>,
        },
        String = 3 {
            /// 1-32 lowercase character name matching `^[\w-]{1,32}$`.
            name: String,
            /// 1-100 character description.
            description: String,
            /// If the parameter is required or optional (default `false`).
            [?] required: Option<bool>,
            /// Choices for the user to pick from.
            [?] choices: Option<Vec<ApplicationCommandOptionChoice<String>>>,
        },
        /// Any integer between -2^53 and 2^53.
        Integer = 4 {
            /// 1-32 lowercase character name matching `^[\w-]{1,32}$`.
            name: String,
            /// 1-100 character description.
            description: String,
            /// If the parameter is required or optional (default `false`).
            [?] required: Option<bool>,
            /// Choices for the user to pick from.
            [?] choices: Option<Vec<ApplicationCommandOptionChoice<i64>>>,
        },
        Boolean = 5 {
            /// 1-32 lowercase character name matching `^[\w-]{1,32}$`.
            name: String,
            /// 1-100 character description.
            description: String,
            /// If the parameter is required or optional (default `false`).
            [?] required: Option<bool>,
        },
        User = 6 {
            /// 1-32 lowercase character name matching `^[\w-]{1,32}$`.
            name: String,
            /// 1-100 character description.
            description: String,
            /// If the parameter is required or optional (default `false`).
            [?] required: Option<bool>,
        },
        /// Includes all channel types + categories.
        Channel = 7 {
            /// 1-32 lowercase character name matching `^[\w-]{1,32}$`.
            name: String,
            /// 1-100 character description.
            description: String,
            /// If the parameter is required or optional (default `false`).
            [?] required: Option<bool>,
        },
        Role = 8 {
            /// 1-32 lowercase character name matching `^[\w-]{1,32}$`.
            name: String,
            /// 1-100 character description.
            description: String,
            /// If the parameter is required or optional (default `false`).
            [?] required: Option<bool>,
        },
        /// Includes users and roles.
        Mentionable = 9 {
            /// 1-32 lowercase character name matching `^[\w-]{1,32}$`.
            name: String,
            /// 1-100 character description.
            description: String,
            /// If the parameter is required or optional (default `false`).
            [?] required: Option<bool>,
        },
        /// Any double between -2^53 and 2^53
        Number = 10 {
            /// 1-32 lowercase character name matching `^[\w-]{1,32}$`.
            name: String,
            /// 1-100 character description.
            description: String,
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
    #[serde(flatten)]
    pub value: T,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuildApplicationCommandPermissions {
    /// The id of the command.
    pub id: Snowflake,
    /// The id of the application the command belongs to.
    pub application_id: Snowflake,
    /// The id of the guild.
    pub guild_id: Snowflake,
    /// The permissions for the command in the guild.
    pub permissions: Vec<ApplicationCommandPermission>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplicationCommandPermission {
    /// The id of the role or user.
    pub id: Snowflake,
    /// Role or user.
    #[serde(rename = "type")]
    pub kind: ApplicationCommandPermissionType,
    /// `true` to allow, `false` to disallow.
    pub permission: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ApplicationCommandPermissionType(pub u8);

impl ApplicationCommandPermissionType {
    pub const ROLE: ApplicationCommandPermissionType =
        ApplicationCommandPermissionType(1);
    pub const USER: ApplicationCommandPermissionType =
        ApplicationCommandPermissionType(2);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Interaction {
    /// Id of the interaction.
    pub id: Snowflake,
    /// Id of the application this interaction is for.
    pub application_id: Snowflake,
    /// A continuation token for responding to the interaction.
    pub token: String,
    /// Read-only property, always 1.
    pub version: u8,
    #[serde(flatten)]
    pub kind: InteractionType,
}

serde_inner_enum! {
    #[derive(Clone, Debug)]
    pub enum InteractionType = "type" {
        Ping = 1,
        ApplicationCommand = 2 {
            /// The command data payload.
            data: ApplicationCommandInteractionData,
            /// The guild it was sent from.
            [?] guild_id: Option<Snowflake>,
            /// The channel it was sent from.
            channel_id: Snowflake,
            /// Guild member data for the invoking user, including permissions.
            [?] member: Option<GuildMember>,
            /// User object for the invoking user, if invoked in a DM.
            [?] user: Option<User>,
        },
        MessageComponent = 3 {
            /// The guild it was sent from.
            [?] guild_id: Option<Snowflake>,
            /// The channel it was sent from.
            channel_id: Snowflake,
            /// Guild member data for the invoking user, including permissions.
            [?] member: Option<GuildMember>,
            /// User object for the invoking user, if invoked in a DM.
            [?] user: Option<User>,
            /// The message the component was attached to.
            message: Message,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplicationCommandInteractionData {
    /// The ID of the invoked command.
    pub id: Snowflake,
    /// The name of the invoked command.
    pub name: String,
    /// For components, the `custom_id` of the component.
    pub custom_id: String,
    /// For components, the type of the component.
    #[serde(flatten)]
    pub component_type: ComponentType,
}

serde_inner_enum! {
    #[derive(Clone, Debug)]
    pub enum ComponentType = "component_type" {
        ActionRow = 1,
        Button = 2 {
            /// Converted users + roles + channels.
            [?] resolved: Option<ApplicationCommandInteractionDataResolved>,
            // /// The params + values from the user.
            // TODO [?] options: Option<Vec<ApplicationCommandInteractionDataOption>>,
        },
        SelectMenu = 3 {
            /// Converted users + roles + channels.
            [?] resolved: Option<ApplicationCommandInteractionDataResolved>,
            // /// The params + values from the user.
            // TODO [?] options: Option<Vec<ApplicationCommandInteractionDataOption>>,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplicationCommandInteractionDataResolved {
    /// The IDs and [User] objects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub users: Option<HashMap<Snowflake, User>>,
    /// The IDs and partial [GuildMember] objects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub members: Option<HashMap<Snowflake, GuildMember>>,
    /// The IDs and [Role] objects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roles: Option<HashMap<Snowflake, Role>>,
    /// The IDs and partial [Channel] objects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channels: Option<HashMap<Snowflake, Channel>>,
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
        /// Only the user receiving the message can see it.
        const EPHEMERAL = 1 << 6;
    }
}
