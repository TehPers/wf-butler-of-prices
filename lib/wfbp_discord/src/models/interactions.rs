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

serde_inner_enum! {
    #[derive(Clone, Debug)]
    pub enum ApplicationCommand = "type" {
        ChatInput = 1 {
            /// Unique id of the command.
            id: Snowflake,
            /// Unique id of the parent application.
            application_id: Snowflake,
            /// Guild id of the command, if not global.
            [?] guild_id: Option<Snowflake>,
            /// 1-32 lowercase character name matching `^[\w-]{1,32}$`.
            name: String,
            /// 1-100 character description.
            description: String,
            /// The parameters for the command.
            options: Vec<ApplicationCommandOption>,
            /// Whether the command is enabled by default when the app is added to a
            /// guild (default `true`).
            [?] default_permission: Option<bool>,
        },
        User = 2 {
            /// Unique id of the command.
            id: Snowflake,
            /// Unique id of the parent application.
            application_id: Snowflake,
            /// Guild id of the command, if not global.
            [?] guild_id: Option<Snowflake>,
            /// 1-32 lowercase character name matching `^[\w-]{1,32}$`.
            name: String,
            /// 1-100 character description.
            description: String,
            /// Whether the command is enabled by default when the app is added to a
            /// guild (default `true`).
            [?] default_permission: Option<bool>,
        },
        Message = 3 {
            /// Unique id of the command.
            id: Snowflake,
            /// Unique id of the parent application.
            application_id: Snowflake,
            /// Guild id of the command, if not global.
            [?] guild_id: Option<Snowflake>,
            /// 1-32 lowercase character name matching `^[\w-]{1,32}$`.
            name: String,
            /// 1-100 character description.
            description: String,
            /// Whether the command is enabled by default when the app is added to a
            /// guild (default `true`).
            [?] default_permission: Option<bool>,
        },
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
    /// The type of the command.
    #[serde(flatten)]
    pub kind: InteractionType,
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
            /// The guild it was sent from.
            [?] guild_id: Option<Snowflake>,
            /// The channel it was sent from.
            channel_id: Snowflake,
            /// Guild member data for the invoking user, including permissions.
            [?] member: Option<GuildMember>,
            /// User object for the invoking user, if invoked in a DM.
            [?] user: Option<User>,
            /// The command data payload.
            data: ApplicationCommandInteractionData,
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
            /// The type of the component.
            component_type: ComponentType,
        },
        Autocomplete = 4 {
            /// The guild it was sent from.
            [?] guild_id: Option<Snowflake>,
            /// The channel it was sent from.
            channel_id: Snowflake,
            /// Guild member data for the invoking user, including permissions.
            [?] member: Option<GuildMember>,
            /// User object for the invoking user, if invoked in a DM.
            [?] user: Option<User>,
        },
        ModalSubmit = 5 {
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
            // /// TODO:  The command data payload.
            // data: ApplicationCommandInteractionData,
        },
    }
}

serde_inner_enum! {
    #[derive(Clone, Debug)]
    pub enum ApplicationCommandInteractionData = "type" {
        SlashCommand = 1 {
            /// The ID of the invoked command.
            id: Snowflake,
            /// The name of the invoked command.
            name: String,
            /// Converted users + roles + channels.
            [?] resolved: Option<ApplicationCommandInteractionDataResolved>,
            /// The params + values from the user.
            [?] options: Option<Vec<ApplicationCommandInteractionDataOption>>,
        },
        User = 2 {
            /// The ID of the invoked command.
            id: Snowflake,
            /// The name of the invoked command.
            name: String,
            /// ID the of user.
            target_id: Snowflake,
        },
        Message = 3 {
            /// The ID of the invoked command.
            id: Snowflake,
            /// The name of the invoked command.
            name: String,
            /// ID the of message.
            target_id: Snowflake,
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
            value: Snowflake,
        },
        Channel = 7 {
            value: Snowflake,
        },
        Role = 8 {
            value: Snowflake,
        },
        Mentionable = 9 {
            value: Snowflake,
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

serde_inner_enum! {
    #[derive(Clone, Debug)]
    pub enum CreateApplicationCommand = "type" {
        ChatInput = 1 {
            /// 1-32 lowercase character name matching `^[\w-]{1,32}$`.
            name: String,
            /// 1-100 character description.
            description: String,
            /// The parameters for the command.
            [?] options: Option<Vec<ApplicationCommandOption>>,
            /// Whether the command is enabled by default when the app is added to a
            /// guild (default `true`).
            [?] default_permission: Option<bool>,
        },
        User = 2 {
            /// 1-32 lowercase character name matching `^[\w-]{1,32}$`.
            name: String,
            /// Whether the command is enabled by default when the app is added to a
            /// guild (default `true`).
            [?] default_permission: Option<bool>,
        },
        Message = 3 {
            /// 1-32 lowercase character name matching `^[\w-]{1,32}$`.
            name: String,
            /// Whether the command is enabled by default when the app is added to a
            /// guild (default `true`).
            [?] default_permission: Option<bool>,
        },
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
    pub id: Snowflake,
    /// The permissions for the command in the guild.
    pub permissions: Vec<ApplicationCommandPermission>,
}
