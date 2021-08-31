use async_trait::async_trait;
use std::fmt::{Debug, Formatter};
use wfinfo_discord::models::{
    ApplicationCommandInteractionData, CreateApplicationCommand,
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
    {
        $name:ident,
        required = {
            $($([$req_mod:tt])? $req_ty_name:ident = $req_field_name:ident : $req_field_ty:ty),*
            $(,)?
        },
        optional = {
            $($([$opt_mod:tt])? $opt_field_name:ident : $opt_field_ty:ty),*
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
        build = |$builder:pat_param| -> $built_ty:ty $build:block
        $(,)?
    } => {
        #[allow(non_camel_case_types)]
        pub struct $name<$($req_ty_name = ()),*> {
            $($req_field_name : $req_ty_name,)*
            $($opt_field_name : Option<$opt_field_ty>,)*
            $($extra_field_name : $extra_field_ty,)*
        }

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
                        value: $req_field_ty
                    ) -> helper!(@ty set $req_ty_name) {
                        helper!(@value self, $req_field_name = value)
                    }
                )*

                $(
                    #[inline]
                    pub fn $opt_field_name(
                        mut self,
                        value: $opt_field_ty
                    ) -> Self {
                        self.$opt_field_name = Some(value);
                        self
                    }
                )*
            }

            impl $name<$($req_field_ty),*> {
                #[inline]
                fn build(self) -> $built_ty {
                    let $builder = self;
                    $build
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

pub struct Command {
    pub name: String,
    pub description: String,
    pub options: Vec<CommandOption>,
    pub default_permission: Option<bool>,
    pub callback: Option<Box<dyn CommandCallback>>,
}

impl From<&Command> for CreateApplicationCommand {
    fn from(command: &Command) -> Self {
        CreateApplicationCommand {
            name: command.name.clone(),
            description: command.description.clone(),
            options: todo!(),
            default_permission: command.default_permission,
        }
    }
}

#[async_trait]
pub trait CommandCallback: Send + Sync + 'static {
    fn invoke(&self, args: ApplicationCommandInteractionData);
}

// TODO
type CommandOption = ();

builder! {
    CommandBuilder,
    required = {
        Name = name: String,
        Desc = description: String,
    },
    optional = {
        default_permission: bool,
    },
    extra = {
        options: Vec<CommandOption> = Vec::new(),
        [hide] callback: Option<Box<dyn CommandCallback>> = None,
    },
    build = |builder| -> Command {
        Command {
            name: builder.name,
            description: builder.description,
            options: builder.options,
            default_permission: builder.default_permission,
            callback: builder.callback,
        }
    }
}

impl CommandBuilder {
    #[inline]
    pub fn option<F: FnOnce(OptionBuilder) -> OptionBuilder<String, String>>(
        mut self,
        builder: F,
    ) -> Self {
        let builder = builder(OptionBuilder::new());
        self.options.push(builder.build());
        self
    }

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
    OptionBuilder,
    required = {
        Name = name: String,
        Desc = description: String,
    },
    optional = {
        required: bool,
    },
    extra = {
        choices: Vec<()> = Vec::new(),
        options: Vec<CommandOption> = Vec::new(),
    },
    build = |_builder| -> CommandOption {
        todo!()
    },
}
