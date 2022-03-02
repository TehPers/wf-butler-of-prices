use std::collections::HashSet;

use wfbp_discord::models::{
    ApplicationCommandInteractionData, GuildMember, Snowflake, User,
};

use crate::interactions::{ResponseData, SlashCommandHandler};

#[derive(Debug)]
pub struct ApplicationCommandHandler {
    slash_commands: HashSet<String, Box<dyn SlashCommandHandler>>,
}

impl ApplicationCommandHandler {
    pub async fn handle(
        &self,
        response_data: &ResponseData,
    ) -> anyhow::Result<()> {
    }
}

pub struct ApplicationCommandData {
    pub guild_id: Option<Snowflake>,
    pub channel_id: Snowflake,
    pub member: Option<GuildMember>,
    pub user: Option<User>,
    pub data: ApplicationCommandInteractionData,
}
