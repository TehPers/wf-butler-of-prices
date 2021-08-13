use derive_more::{Deref, DerefMut, From, Into};
use ed25519_dalek::PublicKey;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use wfinfo_lib::models::Snowflake;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub app_id: Snowflake,
    pub client_id: Snowflake,
    pub client_secret: String,
    #[serde(default)]
    pub ignore_signature: bool,
    pub discord_public_key: ConfigPublicKey,
    pub admin_public_key: ConfigPublicKey,
    #[serde(rename = "functions_customhandler_port", default = "default_port")]
    pub port: u16,
}

fn default_port() -> u16 {
    3000
}

#[derive(Clone, Debug, From, Into, Deref, DerefMut)]
pub struct ConfigPublicKey(PublicKey);

impl Serialize for ConfigPublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = self.as_bytes();
        let encoded = hex::encode(bytes);
        serializer.serialize_str(&encoded)
    }
}

impl<'de> Deserialize<'de> for ConfigPublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded: Cow<'de, str> = Deserialize::deserialize(deserializer)?;
        let bytes =
            hex::decode(encoded.as_bytes()).map_err(de::Error::custom)?;
        let public_key =
            PublicKey::from_bytes(&bytes).map_err(de::Error::custom)?;
        Ok(public_key.into())
    }
}
