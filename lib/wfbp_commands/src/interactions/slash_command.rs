use crate::interactions::{InteractionResponder, SnowflakeResolver};
use async_trait::async_trait;
use derive_more::{Display, Error};
use runtime_injector::{Service, Interface};
use std::{borrow::Cow, collections::HashMap, error::Error, fmt::Debug};
use wfbp_discord::models::{
    ApplicationCommandId, ApplicationCommandInteractionDataOption,
    ApplicationCommandInteractionDataOptionValue, ApplicationCommandOption,
    ChannelId, MentionableId, RoleId, Snowflake, UserId,
};

/// A handler for an invoked slash command.
#[async_trait]
pub trait SlashCommandHandler: Debug + Interface {
    /// Handles an invocation of this slash command.
    async fn handle(&self, args: SlashCommandArgs) -> anyhow::Result<()>;

    /// The name of this command.
    fn name(&self) -> String;

    /// A description of this command.
    fn description(&self) -> String;

    /// The name localizations for this command.
    fn name_localizations(&self) -> Option<HashMap<String, String>> {
        None
    }

    /// The description localizations for this command.
    fn description_localizations(&self) -> Option<HashMap<String, String>> {
        None
    }

    /// Whether this command is usable by default.
    fn default_permission(&self) -> bool {
        true
    }

    /// Gets the options in this command.
    fn options(&self) -> Vec<ApplicationCommandOption>;
}

/// Arguments and resources for the invoked slash command.
#[derive(Clone, Debug)]
pub struct SlashCommandArgs {
    /// The ID of the invoked command.
    pub command_id: ApplicationCommandId,
    /// The options for the invoked command.
    pub opts: SlashCommandOpts,
    /// The snowflake resolver for the invoked command.
    pub resolver: SnowflakeResolver,
    /// The command responder for the invoked command.
    pub responder: InteractionResponder,
}

/// Options for a slash command.
#[derive(Clone, Debug)]
pub struct SlashCommandOpts {
    options: HashMap<String, ApplicationCommandInteractionDataOptionValue>,
}

impl SlashCommandOpts {
    /// Gets a raw option value.
    pub fn get_raw<'s, 'n>(
        &'s self,
        name: &'n str,
    ) -> Option<&'s ApplicationCommandInteractionDataOptionValue> {
        self.options.get(name)
    }

    /// Gets an option value.
    pub fn get<'s, 'n, V>(&'s self, name: &'n str) -> Result<V, GetOptionError>
    where
        V: FromCommandOption<'s>,
    {
        self.options
            .get(name)
            .ok_or(GetOptionError::MissingValue)
            .and_then(|opt| V::from_option(opt))
    }
}

impl FromIterator<ApplicationCommandInteractionDataOption>
    for SlashCommandOpts
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = ApplicationCommandInteractionDataOption>,
    {
        let options =
            iter.into_iter().map(|opt| (opt.name, opt.value)).collect();
        SlashCommandOpts { options }
    }
}

/// Utility trait for converting raw option values to useful types.
pub trait FromCommandOption<'opt>: Sized {
    /// Tries to convert a raw option value to this type.
    fn from_option(
        opt: &'opt ApplicationCommandInteractionDataOptionValue,
    ) -> Result<Self, GetOptionError>;
}

/// Error that can occur when getting an option.
#[derive(Debug, Display, Error)]
pub enum GetOptionError {
    /// The option is missing a value.
    #[display(fmt = "missing value")]
    MissingValue,
    /// The option does not contain a valid value.
    #[display(fmt = "invalid value type")]
    InvalidType,
    /// An error occurred while converting the value.
    #[display(fmt = "error converting value")]
    ConversionError {
        /// The inner error that occurred while converting the value.
        inner: Box<dyn Error + Send + Sync + 'static>,
    },
}

macro_rules! from_cmd_opt {
    (@val [?] $val:expr) => {
        $val.map_err(|err| GetOptionError::ConversionError { inner: Box::new(err) })
    };
    (@val [] $val:expr) => {
        Ok($val)
    };
    (
        <$lt:lifetime> $type:ty,
        $($([$modifier:tt])? $variant:ident { $($field:ident),* } => $val:expr),*
        $(,)?
    ) => {
        impl<$lt> FromCommandOption<$lt> for $type {
            fn from_option(
                opt: &$lt ApplicationCommandInteractionDataOptionValue,
            ) -> Result<Self, GetOptionError> {
                match opt {
                    $(
                        ApplicationCommandInteractionDataOptionValue::$variant {
                            $($field,)*
                            ..
                        } => {
                            from_cmd_opt!(@val [$($modifier)?] $val)
                        },
                    )*
                    _ => Err(GetOptionError::MissingValue),
                }
            }
        }
    };
    (
        $type:ty,
        $($([$modifier:tt])? $variant:ident { $($field:ident),* } => $val:expr),*
        $(,)?
    ) => {
        from_cmd_opt!(
            <'opt> $type,
            $($([$modifier])? $variant { $($field),* } => $val),*
        );
    };
}

from_cmd_opt!(bool, Boolean { value } => *value);

from_cmd_opt!(i8, [?] Integer { value } => (*value).try_into());
from_cmd_opt!(i16, [?] Integer { value } => (*value).try_into());
from_cmd_opt!(i32, [?] Integer { value } => (*value).try_into());
from_cmd_opt!(i64, Integer { value } => *value);
from_cmd_opt!(i128, Integer { value } => (*value).into());
from_cmd_opt!(isize, [?] Integer { value } => (*value).try_into());

from_cmd_opt!(u8, [?] Integer { value } => (*value).try_into());
from_cmd_opt!(u16, [?] Integer { value } => (*value).try_into());
from_cmd_opt!(u32, [?] Integer { value } => (*value).try_into());
from_cmd_opt!(u64, [?] Integer { value } => (*value).try_into());
from_cmd_opt!(u128, [?] Integer { value } => (*value).try_into());
from_cmd_opt!(usize, [?] Integer { value } => (*value).try_into());

from_cmd_opt!(f32, Number { value } => *value as f32);
from_cmd_opt!(f64, Number { value } => *value);

from_cmd_opt!(<'opt> &'opt str, String { value } => value.as_str());
from_cmd_opt!(<'opt> Cow<'opt, str>, String { value } => value.into());

from_cmd_opt!(
    Snowflake,
    User { value } => (*value).into_inner(),
    Channel { value } => (*value).into_inner(),
    Role { value } => (*value).into_inner(),
    Mentionable { value } => (*value).into_inner(),
);

from_cmd_opt!(UserId, User { value } => *value);
from_cmd_opt!(ChannelId, Channel { value } => *value);
from_cmd_opt!(RoleId, Role { value } => *value);
from_cmd_opt!(
    MentionableId,
    User { value } => (*value).into(),
    Role { value } => (*value).into(),
    Mentionable { value } => *value,
);
