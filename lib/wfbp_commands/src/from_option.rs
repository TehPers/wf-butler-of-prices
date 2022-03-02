use anyhow::Context;
use derive_more::{Display, Error};
use wfbp_discord::models::{
    ApplicationCommandInteractionDataOption,
    ApplicationCommandInteractionDataOptionValue, Snowflake,
};

#[non_exhaustive]
#[derive(Debug, Display, Error)]
pub enum FromOptionError {
    #[display(fmt = "the option is the wrong type")]
    InvalidType,
    #[display(fmt = "error parsing input: {}", _0)]
    ParseError(#[error(ignore)] String),
    #[display(fmt = "{}", _0)]
    Custom(#[error(ignore)] anyhow::Error),
}

pub trait FromOption<'a>: Sized {
    fn from_option(
        option: &'a ApplicationCommandInteractionDataOption,
    ) -> Result<Self, FromOptionError>;
}

macro_rules! from_option {
    (@branch $kind:expr, $variant:ident => |$value:pat_param| $result:expr) => {
        if let ApplicationCommandInteractionDataOptionValue::$variant {
            ref value
        } = $kind {
            return match value {
                $value => $result
            };
        }
    };
    (@branch $kind:expr, $variant:ident) => {
        if let ApplicationCommandInteractionDataOptionValue::$variant {
            value
        } = $kind {
            return Ok(value);
        }
    };
    ($target:ty = $variant:ident $(=> |$value:pat_param| $result:expr)?) => {
        from_option!(
            $target = {
                $variant $(=> |$value| $result)?,
            }
        );
    };
    (
        $target:ty = {
            $($variant:ident $(=> |$value:pat_param| $result:expr)?),*
            $(,)?
        }
    ) => {
        impl<'a> FromOption<'a> for $target {
            fn from_option(
                option: &'a ApplicationCommandInteractionDataOption,
            ) -> Result<Self, FromOptionError> {
                $(from_option!(@branch option.value, $variant $(=> |$value| $result)?);)*
                Err(FromOptionError::InvalidType)
            }
        }
    };
}

from_option!(i64 = Integer);
from_option!(bool = Boolean);
from_option!(f64 = Number);
from_option!(Snowflake = {
    User,
    Channel,
    Role,
    Mentionable,
});
from_option!(String = {
    String => |value| Ok(value.clone()),
});

from_option!(i8 = Integer => |&value| {
    value.try_into()
        .context("error converting integer")
        .map_err(FromOptionError::Custom)
});
from_option!(i16 = Integer => |&value| {
    value.try_into()
        .context("error converting integer")
        .map_err(FromOptionError::Custom)
});
from_option!(i32 = Integer => |&value| {
    value.try_into()
        .context("error converting integer")
        .map_err(FromOptionError::Custom)
});
from_option!(i128 = Integer => |&value| Ok(value.into()));

from_option!(u8 = Integer => |&value| {
    value.try_into()
        .context("error converting integer")
        .map_err(FromOptionError::Custom)
});
from_option!(u16 = Integer => |&value| {
    value.try_into()
        .context("error converting integer")
        .map_err(FromOptionError::Custom)
});
from_option!(u32 = Integer => |&value| {
    value.try_into()
        .context("error converting integer")
        .map_err(FromOptionError::Custom)
});
from_option!(u64 = Integer => |&value| {
    value.try_into()
        .context("error converting integer")
        .map_err(FromOptionError::Custom)
});
from_option!(u128 = Integer => |&value| {
    value.try_into()
        .context("error converting integer")
        .map_err(FromOptionError::Custom)
});

from_option!(f32 = Number => |&value| Ok(value as f32));

impl<'a> FromOption<'a> for &'a str {
    fn from_option(
        option: &'a ApplicationCommandInteractionDataOption,
    ) -> Result<Self, FromOptionError> {
        match option.value {
            ApplicationCommandInteractionDataOptionValue::String {
                ref value,
            } => Ok(value),
            _ => Err(FromOptionError::InvalidType),
        }
    }
}
