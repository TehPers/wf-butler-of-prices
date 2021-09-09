use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PayloadResponse<T, I = ()> {
    pub payload: T,
    #[serde(default = "Option::default")]
    pub include: Option<I>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemPayload {
    pub item: ItemSet,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemsPayload<T> {
    pub items: Vec<T>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemOrdersPayload {
    pub orders: Vec<ItemOrder>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemShort {
    pub id: String,
    pub url_name: String,
    pub thumb: String,
    pub item_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemOrder {
    pub id: String,
    pub platinum: u32,
    pub quantity: u32,
    pub order_type: OrderType,
    pub platform: Platform,
    pub creation_date: DateTime<FixedOffset>,
    pub last_update: DateTime<FixedOffset>,
    pub user: UserShort,
    #[serde(flatten)]
    pub rank: Option<ItemRank>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ItemRank {
    ModOrArcane { mod_rank: u8 },
    Relic { subtype: RelicSubtype },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RelicSubtype {
    Intact,
    Exceptional,
    Flawless,
    Radiant,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemSet {
    pub id: String,
    pub items_in_set: Vec<ItemFull>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemFull {
    pub id: String,
    pub url_name: String,
    pub icon: String,
    pub thumb: String,
    #[serde(default)]
    pub sub_icon: Option<String>,
    // #[serde(alias = "max_rank")]
    // pub mod_max_rank: u8,
    #[serde(default)]
    pub ducats: Option<u16>,
    #[serde(default)]
    pub set_root: Option<bool>,
    #[serde(default)]
    pub mastery_rank: Option<u8>,
    #[serde(default)]
    pub rarity: Option<ModRarity>,
    #[serde(default)]
    pub trading_tax: Option<u32>,
    pub en: LangInItem,
    // TODO: there are other languages too
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModRarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
    Peculiar,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LangInItem {
    pub item_name: String,
    pub description: String,
    #[serde(default)]
    pub wiki_link: Option<String>,
    // TODO: pub drop: Vec<()>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserShort {
    pub id: String,
    pub ingame_name: String,
    pub status: UserStatus,
    pub region: String,
    pub reputation: i32,
    pub avatar: Option<String>,
    pub last_seen: Option<DateTime<FixedOffset>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    InGame,
    Online,
    Offline,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Sell,
    Buy,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
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
