use crate::models::{Snowflake, User};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sticker {
    pub id: Snowflake,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pack_id: Option<Snowflake>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub tags: String,
    #[serde(rename = "type")]
    pub kind: StickerType,
    pub format_type: StickerFormatType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub available: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<Snowflake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_value: Option<u16>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StickerType(pub u16);

impl StickerType {
    pub const STANDARD: StickerType = StickerType(1);
    pub const GUILD: StickerType = StickerType(2);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StickerFormatType(pub u16);

impl StickerFormatType {
    pub const PNG: StickerFormatType = StickerFormatType(1);
    pub const APNG: StickerFormatType = StickerFormatType(2);
    pub const LOTTIE: StickerFormatType = StickerFormatType(3);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StickerItem {
    pub id: Snowflake,
    pub name: String,
    pub format_type: StickerFormatType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StickerPack {
    pub id: Snowflake,
    pub stickers: Vec<Sticker>,
    pub name: String,
    pub sku_id: Snowflake,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_sticker_id: Option<Snowflake>,
    pub description: String,
    pub banner_asset_id: Snowflake,
}
