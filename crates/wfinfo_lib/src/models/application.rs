use crate::models::{Snowflake, Team, User};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Application {
    /// The id of the app.
    pub id: Snowflake,
    /// The name of the app.
    pub name: String,
    /// The icon hash of the app.
    pub icon: Option<String>,
    /// The description of the app.
    pub description: String,
    /// An array of rpc origin urls, if rpc is enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rpc_origins: Option<Vec<String>>,
    /// When `false` only app owner can join the app's bot to guilds.
    pub bot_public: bool,
    /// When `true` the app's bot will only join upon completion of the full oauth2 code grant flow.
    pub bot_require_code_grant: bool,
    /// The url of the app's terms of service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terms_of_service_url: Option<String>,
    /// The url of the app's privacy policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub privacy_policy_url: Option<String>,
    /// Partial user object containing info on the owner of the application.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<User>,
    /// If this application is a game sold on Discord, this field will be the summary field for the store page of its primary sku.
    pub summary: String,
    /// The hex encoded key for verification in interactions and the GameSDK's GetTicket.
    pub verify_key: String,
    /// If the application belongs to a team, this will be a list of the members of that team.
    pub team: Option<Team>,
    /// If this application is a game sold on Discord, this field will be the guild to which it has been linked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<Snowflake>,
    /// If this application is a game sold on Discord, this field will be the id of the "Game SKU" that is created, if exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_sku_id: Option<Snowflake>,
    /// If this application is a game sold on Discord, this field will be the URL slug that links to the store page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// The application's default rich presence invite cover image hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_image: Option<String>,
    /// The application's public flags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flags: Option<ApplicationFlags>,
}

bitflags! {
    #[derive(Default, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct ApplicationFlags: u32 {
        const GATEWAY_PRESENCE = 1 << 12;
        const GATEWAY_PRESENCE_LIMITED = 1 << 13;
        const GATEWAY_GUILD_MEMBERS = 1 << 14;
        const GATEWAY_GUILD_MEMBERS_LIMITED = 1 << 15;
        const VERIFICATION_PENDING_GUILD_LIMIT = 1 << 16;
        const EMBEDDED = 1 << 17;
    }
}
