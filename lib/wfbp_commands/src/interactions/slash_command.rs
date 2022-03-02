use async_trait::async_trait;
use std::{collections::HashMap, fmt::Debug};
use wfbp_discord::models::{
    ApplicationCommandInteractionDataOption,
    ApplicationCommandInteractionDataOptionValue,
};

#[async_trait]
pub trait SlashCommandHandler: Debug + Send + Sync + 'static {
    async fn handle(&self, opts: SlashCommandOpts) -> anyhow::Result<()>;
}

#[derive(Clone, Debug)]
pub struct SlashCommandOpts {
    options: HashMap<String, ApplicationCommandInteractionDataOptionValue>,
}

impl SlashCommandOpts {
    pub fn new<I>(options: I) -> Self
    where
        I: IntoIterator<Item = ApplicationCommandInteractionDataOption>,
    {
        Self {
            options: options
                .into_iter()
                .map(|opt| (opt.name, opt.value))
                .collect(),
        }
    }
}
