use chrono::{DateTime, FixedOffset, NaiveDateTime};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use std::{
    fmt::{Display, Formatter},
    num::TryFromIntError,
};

/// A unique ID for a Discord entity (user, role, channel, guild, etc).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Snowflake(u64);

impl Snowflake {
    pub const TIMESTAMP_MASK: u64 = 0xFFFFFFFFFFC00000;
    pub const WORKER_ID_MASK: u64 = 0x00000000003E0000;
    pub const PROCESS_ID_MASK: u64 = 0x000000000001F000;
    pub const INCREMENT_MASK: u64 = 0x0000000000000FFF;
    pub const DISCORD_EPOCH: u64 = 1420070400000;

    pub const fn new(value: u64) -> Self {
        Snowflake(value)
    }

    pub const fn to_u64(self) -> u64 {
        self.0
    }

    pub fn timestamp(self) -> Result<Timestamp, TryFromIntError> {
        let timestamp = (self.0 >> 22) + Self::DISCORD_EPOCH;
        let timestamp = timestamp.try_into()?;
        let naive = NaiveDateTime::from_timestamp(timestamp, 0);
        let datetime = DateTime::from_utc(naive, FixedOffset::east(0));
        Ok(Timestamp(datetime))
    }

    pub fn worker_id(self) -> u8 {
        ((self.0 & Self::WORKER_ID_MASK) >> 17) as u8
    }

    pub fn process_id(self) -> u8 {
        ((self.0 & Self::PROCESS_ID_MASK) >> 12) as u8
    }

    pub fn increment(self) -> u16 {
        (self.0 & Self::INCREMENT_MASK) as u16
    }
}

impl Display for Snowflake {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for Snowflake {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for Snowflake {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SnowflakeVisitor;
        impl<'de> Visitor<'de> for SnowflakeVisitor {
            type Value = Snowflake;

            fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
                write!(f, "a string containing a parseable u64")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map(Snowflake).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(SnowflakeVisitor)
    }
}

/// A timestamp attached to a [`Snowflake`].
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Hash,
    Serialize,
    Deserialize,
)]
#[serde(transparent)]
pub struct Timestamp(pub DateTime<FixedOffset>);

#[derive(Clone, Debug)]
pub enum Nonce {
    Integer(u32),
    String(String),
}

impl Serialize for Nonce {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            &Nonce::Integer(value) => serializer.serialize_u32(value),
            &Nonce::String(ref value) => serializer.serialize_str(&value),
        }
    }
}

impl<'de> Deserialize<'de> for Nonce {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NonceVisitor;
        impl<'de> Visitor<'de> for NonceVisitor {
            type Value = Nonce;

            fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
                write!(f, "a string or unsigned integer")
            }

            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Nonce::Integer(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Nonce::String(v.to_owned()))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Nonce::String(v))
            }
        }

        deserializer.deserialize_any(NonceVisitor)
    }
}

#[macro_export]
macro_rules! snowflake_newtype {
    {
        $(#[$attr:meta])*
        $visibility:vis struct $name:ident;
    } => {
        $(#[$attr])*
        #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
        #[derive($crate::serde::Serialize, $crate::serde::Deserialize)]
        #[derive(
            $crate::derive_more::From,
            $crate::derive_more::Into,
            $crate::derive_more::Display,
        )]
        #[serde(transparent)]
        $visibility struct $name(pub $crate::models::Snowflake);

        impl $name {
            /// Converts into the inner value.
            pub fn into_inner(self) -> $crate::models::Snowflake {
                self.0
            }
        }
    };
}
