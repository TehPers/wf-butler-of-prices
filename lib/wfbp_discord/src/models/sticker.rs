use crate::{
    models::{GuildId, Snowflake, User},
    serde_inner_enum, snowflake_newtype,
};
use serde::{Deserialize, Serialize};

snowflake_newtype! {
    /// A unique ID for a sticker.
    pub struct StickerId;
}

/// Represents a sticker that can be sent in messages.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sticker {
    pub id: StickerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pack_id: Option<StickerPackId>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub tags: String,
    #[serde(flatten)]
    pub kind: StickerType,
    pub format_type: StickerFormatType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub available: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_value: Option<u16>,
}

serde_inner_enum! {
    #[derive(Clone, Debug)]
    pub enum StickerType = "type" {
        Standard = 1,
        Guild = 2,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StickerFormatType(pub u16);

impl StickerFormatType {
    pub const PNG: StickerFormatType = StickerFormatType(1);
    pub const APNG: StickerFormatType = StickerFormatType(2);
    pub const LOTTIE: StickerFormatType = StickerFormatType(3);
}

// The smallest amount of data required to render a [`Sticker`]. A partial
// [`Sticker`] object.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StickerItem {
    pub id: StickerId,
    pub name: String,
    pub format_type: StickerFormatType,
}

snowflake_newtype! {
    /// A unique ID for a sticker pack.
    pub struct StickerPackId;
}

snowflake_newtype! {
    /// A unique ID for a sticker pack SKU.
    pub struct StickerPackSkuId;
}

/// Represents a pack of standard [`Sticker`]s.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StickerPack {
    pub id: StickerPackId,
    pub stickers: Vec<Sticker>,
    pub name: String,
    pub sku_id: StickerPackSkuId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_sticker_id: Option<StickerId>,
    pub description: String,
    pub banner_asset_id: Snowflake,
}
