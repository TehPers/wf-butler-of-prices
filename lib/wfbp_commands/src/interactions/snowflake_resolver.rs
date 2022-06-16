use anyhow::Context;
use std::collections::{hash_map::Entry, HashMap};
use wfbp_discord::{
    models::{
        ApplicationCommandInteractionDataResolved, Attachment, AttachmentId,
        Channel, ChannelId, GuildId, GuildMember, Message, MessageId, Role,
        RoleId, User, UserId,
    },
    routes::{
        GetChannel, GetChannelMessage, GetGuildMember, GetGuildRoles, GetUser,
    },
    DiscordRestClient,
};

/// Resolves IDs into entities, optionally by calling the Discord REST API.
#[derive(Clone, Debug, Default)]
pub struct SnowflakeResolver {
    users: HashMap<UserId, User>,
    members: HashMap<UserId, GuildMember>,
    roles: HashMap<RoleId, Role>,
    channels: HashMap<ChannelId, Channel>,
    messages: HashMap<MessageId, Message>,
    attachments: HashMap<AttachmentId, Attachment>,
}

impl SnowflakeResolver {
    /// Gets a resolved user.
    pub fn get_user(&self, id: UserId) -> Option<&User> {
        self.users.get(&id)
    }

    /// Gets a resolved guild member.
    pub fn get_member(&self, id: UserId) -> Option<&GuildMember> {
        self.members.get(&id)
    }

    /// Gets a resolved role.
    pub fn get_role(&self, id: RoleId) -> Option<&Role> {
        self.roles.get(&id)
    }

    /// Gets a resolved channel.
    pub fn get_channel(&self, id: ChannelId) -> Option<&Channel> {
        self.channels.get(&id)
    }

    /// Gets a resolved message.
    pub fn get_message(&self, id: MessageId) -> Option<&Message> {
        self.messages.get(&id)
    }

    /// Gets a resolved attachment.
    pub fn get_attachment(&self, id: AttachmentId) -> Option<&Attachment> {
        self.attachments.get(&id)
    }

    /// Fetches a user if needed.
    pub async fn fetch_user(
        &mut self,
        id: UserId,
        client: DiscordRestClient,
    ) -> anyhow::Result<&User> {
        let entry = self.users.entry(id);
        let entry = match entry {
            Entry::Occupied(entry) => return Ok(entry.into_mut()),
            Entry::Vacant(entry) => entry,
        };

        let user = GetUser::execute(&client, id).await?;
        Ok(entry.insert(user))
    }

    /// Fetches a guild member if needed.
    pub async fn fetch_member(
        &mut self,
        guild_id: GuildId,
        user_id: UserId,
        client: DiscordRestClient,
    ) -> anyhow::Result<&GuildMember> {
        let entry = self.members.entry(user_id);
        let entry = match entry {
            Entry::Occupied(entry) => return Ok(entry.into_mut()),
            Entry::Vacant(entry) => entry,
        };

        let member =
            GetGuildMember::execute(&client, guild_id, user_id).await?;
        Ok(entry.insert(member))
    }

    /// Fetches a role if needed.
    pub async fn fetch_role<'s>(
        &'s mut self,
        guild_id: GuildId,
        role_id: RoleId,
        client: DiscordRestClient,
    ) -> anyhow::Result<&'s Role> {
        if !self.roles.contains_key(&role_id) {
            let new_roles = GetGuildRoles::execute(&client, guild_id).await?;
            self.roles.extend(
                new_roles.into_iter().map(|role: Role| (role.id, role)),
            );
        }

        self.roles.get(&role_id).context("Role not found")
    }

    /// Fetches a channel if needed.
    pub async fn fetch_channel(
        &mut self,
        channel_id: ChannelId,
        client: DiscordRestClient,
    ) -> anyhow::Result<&Channel> {
        let entry = self.channels.entry(channel_id);
        let entry = match entry {
            Entry::Occupied(entry) => return Ok(entry.into_mut()),
            Entry::Vacant(entry) => entry,
        };

        let channel = GetChannel::execute(&client, channel_id).await?;
        Ok(entry.insert(channel))
    }

    /// Fetches a message if needed.
    pub async fn fetch_message(
        &mut self,
        channel_id: ChannelId,
        message_id: MessageId,
        client: DiscordRestClient,
    ) -> anyhow::Result<&Message> {
        let entry = self.messages.entry(message_id);
        let entry = match entry {
            Entry::Occupied(entry) => return Ok(entry.into_mut()),
            Entry::Vacant(entry) => entry,
        };

        let message =
            GetChannelMessage::execute(&client, channel_id, message_id).await?;
        Ok(entry.insert(message))
    }
}

impl From<ApplicationCommandInteractionDataResolved> for SnowflakeResolver {
    fn from(resolved: ApplicationCommandInteractionDataResolved) -> Self {
        let ApplicationCommandInteractionDataResolved {
            users,
            members,
            roles,
            channels,
            messages,
            attachments,
        } = resolved;

        SnowflakeResolver {
            users: users.unwrap_or_default(),
            members: members.unwrap_or_default(),
            roles: roles.unwrap_or_default(),
            channels: channels.unwrap_or_default(),
            messages: messages.unwrap_or_default(),
            attachments: attachments.unwrap_or_default(),
        }
    }
}
