use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PayloadResponse<T> {
    pub payload: T,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemPayload<T> {
    pub item: T,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemsPayload<T> {
    pub items: Vec<T>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemShort {
    pub id: String,
    pub url_name: String,
    pub thumb: String,
    pub item_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    PC,
    XBox,
    PS4,
    Switch,
}

impl Platform {
    pub fn name(&self) -> &'static str {
        match self {
            Platform::PC => "pc",
            Platform::XBox => "xbox",
            Platform::PS4 => "ps4",
            Platform::Switch => "switch",
        }
    }
}

impl Default for Platform {
    fn default() -> Self {
        Platform::PC
    }
}
