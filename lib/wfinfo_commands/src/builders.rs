use crate::{
    Choice, CommandCallback, CommandOption, CommandOptionType, SlashCommand,
};
use std::{
    borrow::Cow,
    fmt::{Debug, Formatter},
};

macro_rules! builder {
    (@default_ty $_:ty) => {
        ()
    };
    (@debug_field $fmt:expr, $self:expr, $field:ident,) => {
        $fmt.field(stringify!($field), &$self.$field)
    };
    (@debug_field $fmt:expr, $self:expr, $field:ident, [hide]) => {
        $fmt
    };
    (@debug_finish $fmt:expr,) => {
        $fmt.finish()
    };
    (@debug_finish $fmt:expr, $($_:tt)*) => {
        $fmt.finish_non_exhaustive()
    };
    (@arg_ty $_field_ty:ty, $arg_ty:ty) => {
        $arg_ty
    };
    (@arg_ty $field_ty:ty,) => {
        $field_ty
    };
    (@field_val $arg:expr, $arg_convert:expr) => {
        $arg_convert
    };
    (@field_val $arg:expr,) => {
        Into::into($arg)
    };
    {
        $(#[$($attr:meta),*])*
        $name:ident,
        required = {
            $(
                $([$req_mod:tt])?
                $req_ty_name:ident = $req_field_name:ident : $req_field_ty:ty
            ),*
            $(,)?
        },
        optional = {
            $($([$opt_mod:tt])? $opt_field_name:ident : $opt_field_ty:ty $(as $opt_field_arg_ty:ty = $opt_field_arg_convert:expr)?),*
            $(,)?
        },
        extra = {
            $(
                $([$extra_mod:tt])?
                $extra_field_name:ident : $extra_field_ty:ty
                    = $extra_field_default:expr
            ),*
            $(,)?
        },
        ready = $ready:ident,
        build = |$builder:pat_param| -> $built_ty:ty $build:block
        $(,)?
    } => {
        $(#[$($attr),*])*
        pub struct $name<$($req_ty_name = ()),*> {
            $($req_field_name : $req_ty_name,)*
            $($opt_field_name : Option<$opt_field_ty>,)*
            $($extra_field_name : $extra_field_ty,)*
        }

        #[doc = concat!("[", stringify!($name), "] that is ready to be built.")]
        pub type $ready = $name<$($req_field_ty),*>;

        const _: () = {
            macro_rules! helper {
                (@ty set $ty_name:ident) => {
                    $name<$(helper!(@ty_param set $ty_name, $req_ty_name)),*>
                };
                $(
                    (@ty_param set $req_ty_name, $req_ty_name) => {
                        $req_field_ty
                    };
                )*
                (@ty_param set $_ty_name:ident, $fallback_name:ident) => {
                    $fallback_name
                };
                $(
                    (
                        @field_value $self:expr,
                        $req_field_name = $value:expr,
                        $req_field_name
                    ) => {
                        $value
                    };
                )*
                (
                    @field_value $self:expr,
                    $_field:ident = $_value:expr,
                    $fallback_field:ident
                ) => {
                    $self.$fallback_field
                };
                (@value $self:expr, $field:ident = $value:expr) => {
                    $name {
                        $(
                            $req_field_name: helper!(
                                @field_value $self,
                                $field = $value,
                                $req_field_name
                            ),
                        )*
                        $($opt_field_name: $self.$opt_field_name,)*
                        $($extra_field_name: $self.$extra_field_name,)*
                    }
                };
            }

            impl $name<$(builder!(@default_ty $req_field_ty)),*> {
                #[inline]
                pub fn new() -> Self {
                    Self {
                        $($req_field_name: (),)*
                        $($opt_field_name: None,)*
                        $($extra_field_name: $extra_field_default,)*
                    }
                }
            }

            impl<$($req_ty_name),*> $name<$($req_ty_name),*> {
                $(
                    #[inline]
                    pub fn $req_field_name(
                        self,
                        value: impl Into<$req_field_ty>,
                    ) -> helper!(@ty set $req_ty_name) {
                        helper!(@value self, $req_field_name = value.into())
                    }
                )*

                $(
                    #[inline]
                    pub fn $opt_field_name(
                        mut self,
                        $opt_field_name: builder!(@arg_ty $opt_field_ty, $($opt_field_arg_ty)?),
                        // value: impl Into<$opt_field_ty>,
                    ) -> Self {
                        let value = builder!(@field_val $opt_field_name, $($opt_field_arg_convert)?);
                        self.$opt_field_name = Some(value);
                        self
                    }
                )*
            }

            impl $name<$($req_field_ty),*> {
                #[inline]
                pub fn build(self) -> $built_ty {
                    let $builder = self;
                    $build
                }
            }

            impl Default for $name<$(builder!(@default_ty $req_field_ty)),*> {
                fn default() -> Self {
                    Self::new()
                }
            }

            impl From<$name<$($req_field_ty),*>> for $built_ty {
                fn from(builder: $name<$($req_field_ty),*>) -> Self {
                    builder.build()
                }
            }

            impl<$($req_ty_name: Debug),*> Debug for $name<$($req_ty_name),*> {
                #[allow(unused_mut)]
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    let mut fmt = f.debug_struct(stringify!($name));
                    $(let mut fmt = builder!(@debug_field fmt, self, $req_field_name, $([$req_mod])?);)*
                    $(let mut fmt = builder!(@debug_field fmt, self, $opt_field_name, $([$opt_mod])?);)*
                    $(let mut fmt = builder!(@debug_field fmt, self, $extra_field_name, $([$extra_mod])?);)*
                    builder!(
                        @debug_finish fmt,
                        $($($req_mod)?)*
                        $($($opt_mod)?)*
                        $($($extra_mod)?)*
                    )
                }
            }
        };
    };
}

macro_rules! option_builder {
    {
        impl for $name:tt<$($req_ty_name:ident),* $(,)?>,
        option = |$self:pat_param, $option:pat_param| $set_opt:expr,
        helpers = [
            $($opt_fn:ident($builder_ty:ty)),*
            $(,)?
        ]
        $(,)?
    } => {
        impl<$($req_ty_name),*> $name<$($req_ty_name),*> {
            #[inline]
            pub fn option(self, option: impl Into<CommandOption>) -> Self {
                let $self = self;
                let $option = option.into();
                $set_opt
            }

            $(
                #[inline]
                pub fn $opt_fn<R, F>(
                    self,
                    builder: F,
                ) -> Self
                where
                    R: Into<CommandOption>,
                    F: FnOnce($builder_ty) -> R
                {
                    self.option(builder(Default::default()))
                }
            )*
        }
    };
}

#[macro_export]
macro_rules! create_callback {
    {
        capture: {
            $($field_name:ident : $field_ty:ty = $field_init:expr),*
            $(,)?
        },
        handler: async
            |$interaction_data:pat_param, $command_data:pat_param, $options:pat_param|
            $handler:expr
        $(,)?
    } => {{
        struct Callback {
            $($field_name: $field_ty,)*
        }

        #[async_trait::async_trait]
        impl $crate::CommandCallback for Callback {
            async fn invoke<'a>(
                &self,
                $interaction_data: ::std::sync::Arc<$crate::InteractionData>,
                $command_data: &'a $crate::SlashCommandData,
                $options: $crate::CommandOptionRegistry<'a>,
            ) -> Result<(), $crate::HandleInteractionError> {
                let Self {
                    $(ref $field_name,)*
                } = self;

                $handler?;
                Ok(())
            }
        }

        Callback {
            $($field_name: $field_init,)*
        }
    }};
}

builder! {
    CommandBuilder,
    required = {
        Name = name: Cow<'static, str>,
        Desc = description: Cow<'static, str>,
    },
    optional = {
        default_permission: bool,
    },
    extra = {
        options: Vec<CommandOption> = Vec::new(),
        [hide] callback: Option<Box<dyn CommandCallback>> = None,
    },
    ready = ReadyCommandBuilder,
    build = |builder| -> SlashCommand {
        SlashCommand {
            name: builder.name,
            description: builder.description,
            options: builder.options,
            default_permission: builder.default_permission,
            callback: builder.callback,
        }
    }
}

option_builder! {
    impl for CommandBuilder<Name, Desc>,
    option = |mut this, option| {
        this.options.push(option);
        this
    },
    helpers = [
        subcommand_option(SubCommandOptionBuilder),
        subcommand_group_option(SubCommandGroupOptionBuilder),
        string_option(StringOptionBuilder),
        integer_option(IntegerOptionBuilder),
        number_option(NumberOptionBuilder),
        boolean_option(BooleanOptionBuilder),
        user_option(UserOptionBuilder),
        channel_option(ChannelOptionBuilder),
        role_option(RoleOptionBuilder),
        mentionable_option(MentionableOptionBuilder),
    ],
}

impl<Name, Desc> CommandBuilder<Name, Desc> {
    #[inline]
    pub fn callback<C: CommandCallback>(self, callback: C) -> Self {
        self.callback_boxed(Box::new(callback))
    }

    #[inline]
    pub fn callback_boxed(
        mut self,
        callback: Box<dyn CommandCallback>,
    ) -> Self {
        self.callback = Some(callback);
        self
    }
}

builder! {
    SubCommandOptionBuilder,
    required = {
        Name = name: Cow<'static, str>,
        Desc = description: Cow<'static, str>,
    },
    optional = {},
    extra = {
        options: Vec<CommandOption> = Vec::new(),
        [hide] callback: Option<Box<dyn CommandCallback>> = None,
    },
    ready = ReadySubCommandOptionBuilder,
    build = |builder| -> CommandOption {
        CommandOption {
            name: builder.name,
            description: builder.description,
            kind: CommandOptionType::SubCommand {
                options: builder.options,
                callback: builder.callback,
            },
        }
    }
}

option_builder! {
    impl for SubCommandOptionBuilder<Name, Desc>,
    option = |mut this, option| {
        this.options.push(option);
        this
    },
    helpers = [
        string_option(StringOptionBuilder),
        integer_option(IntegerOptionBuilder),
        number_option(NumberOptionBuilder),
        boolean_option(BooleanOptionBuilder),
        user_option(UserOptionBuilder),
        channel_option(ChannelOptionBuilder),
        role_option(RoleOptionBuilder),
        mentionable_option(MentionableOptionBuilder),
    ],
}

impl<Name, Desc> SubCommandOptionBuilder<Name, Desc> {
    #[inline]
    pub fn callback<C: CommandCallback>(self, callback: C) -> Self {
        self.callback_boxed(Box::new(callback))
    }

    #[inline]
    pub fn callback_boxed(
        mut self,
        callback: Box<dyn CommandCallback>,
    ) -> Self {
        self.callback = Some(callback);
        self
    }
}

builder! {
    SubCommandGroupOptionBuilder,
    required = {
        Name = name: Cow<'static, str>,
        Desc = description: Cow<'static, str>,
    },
    optional = {},
    extra = {
        options: Vec<CommandOption> = Vec::new(),
    },
    ready = ReadySubCommandGroupOptionBuilder,
    build = |builder| -> CommandOption {
        CommandOption {
            name: builder.name,
            description: builder.description,
            kind: CommandOptionType::SubCommandGroup {
                options: builder.options,
            },
        }
    }
}

option_builder! {
    impl for SubCommandGroupOptionBuilder<Name, Desc>,
    option = |mut this, option| {
        this.options.push(option);
        this
    },
    helpers = [
        subcommand_option(SubCommandOptionBuilder),
    ],
}

builder! {
    StringOptionBuilder,
    required = {
        Name = name: Cow<'static, str>,
        Desc = description: Cow<'static, str>,
    },
    optional = {
        required: bool,
        // TODO
        choices: Vec<Choice<Cow<'static, str>>>, // as impl IntoIterator<Item = Choice<Cow<'static, str>>> = choices.into_iter().collect(),
    },
    extra = {},
    ready = ReadyStringOptionBuilder,
    build = |builder| -> CommandOption {
        CommandOption {
            name: builder.name,
            description: builder.description,
            kind: CommandOptionType::String {
                required: builder.required,
                choices: builder.choices,
            },
        }
    }
}

builder! {
    IntegerOptionBuilder,
    required = {
        Name = name: Cow<'static, str>,
        Desc = description: Cow<'static, str>,
    },
    optional = {
        required: bool,
        // TODO: rust-analyzer panics if I uncomment the code on the next line
        choices: Vec<Choice<i64>>, // as impl IntoIterator<Item = Choice<i64>> = choices.into_iter().collect(),
    },
    extra = {},
    ready = ReadyIntegerOptionBuilder,
    build = |builder| -> CommandOption {
        CommandOption {
            name: builder.name,
            description: builder.description,
            kind: CommandOptionType::Integer {
                required: builder.required,
                choices: builder.choices,
            },
        }
    }
}

builder! {
    NumberOptionBuilder,
    required = {
        Name = name: Cow<'static, str>,
        Desc = description: Cow<'static, str>,
    },
    optional = {
        required: bool,
        // TODO
        choices: Vec<Choice<f64>>, // as impl IntoIterator<Item = Choice<f64>> = choices.into_iter().collect(),
    },
    extra = {},
    ready = ReadyNumberOptionBuilder,
    build = |builder| -> CommandOption {
        CommandOption {
            name: builder.name,
            description: builder.description,
            kind: CommandOptionType::Number {
                required: builder.required,
                choices: builder.choices,
            },
        }
    }
}

builder! {
    BooleanOptionBuilder,
    required = {
        Name = name: Cow<'static, str>,
        Desc = description: Cow<'static, str>,
    },
    optional = {
        required: bool,
    },
    extra = {},
    ready = ReadyBooleanOptionBuilder,
    build = |builder| -> CommandOption {
        CommandOption {
            name: builder.name,
            description: builder.description,
            kind: CommandOptionType::Boolean {
                required: builder.required,
            },
        }
    }
}

builder! {
    UserOptionBuilder,
    required = {
        Name = name: Cow<'static, str>,
        Desc = description: Cow<'static, str>,
    },
    optional = {
        required: bool,
    },
    extra = {},
    ready = ReadyUserOptionBuilder,
    build = |builder| -> CommandOption {
        CommandOption {
            name: builder.name,
            description: builder.description,
            kind: CommandOptionType::User {
                required: builder.required,
            },
        }
    }
}

builder! {
    ChannelOptionBuilder,
    required = {
        Name = name: Cow<'static, str>,
        Desc = description: Cow<'static, str>,
    },
    optional = {
        required: bool,
    },
    extra = {},
    ready = ReadyChannelOptionBuilder,
    build = |builder| -> CommandOption {
        CommandOption {
            name: builder.name,
            description: builder.description,
            kind: CommandOptionType::Channel {
                required: builder.required,
            },
        }
    }
}

builder! {
    RoleOptionBuilder,
    required = {
        Name = name: Cow<'static, str>,
        Desc = description: Cow<'static, str>,
    },
    optional = {
        required: bool,
    },
    extra = {},
    ready = ReadyRoleOptionBuilder,
    build = |builder| -> CommandOption {
        CommandOption {
            name: builder.name,
            description: builder.description,
            kind: CommandOptionType::Role {
                required: builder.required,
            },
        }
    }
}

builder! {
    MentionableOptionBuilder,
    required = {
        Name = name: Cow<'static, str>,
        Desc = description: Cow<'static, str>,
    },
    optional = {
        required: bool,
    },
    extra = {},
    ready = ReadyMentionableOptionBuilder,
    build = |builder| -> CommandOption {
        CommandOption {
            name: builder.name,
            description: builder.description,
            kind: CommandOptionType::Mentionable {
                required: builder.required,
            },
        }
    }
}
