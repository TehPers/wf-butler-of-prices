use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AdminCommand {
    #[serde(rename = "register_commands")]
    RegisterCommands,
}
