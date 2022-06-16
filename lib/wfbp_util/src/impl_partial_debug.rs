#[macro_export]
macro_rules! impl_partial_debug {
    {
        struct $name:ident = [
            $($field:tt),*
            $(,)?
        ]
    } => {
        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                let mut f = f.debug_struct(::std::stringify!($name));
                $(f.field(::std::stringify!($field), &self.$field);)*
                f.finish_non_exhaustive()
            }
        }
    };
    (@enum_finish partial $f:expr) => {
        $f.finish_non_exhaustive()
    };
    (@enum_finish $f:expr) => {
        $f.finish()
    };
    {
        enum $name:ident = {
            $(
                $([$modifier:tt])? $variant:ident = [
                    $($field:tt),*
                    $(,)?
                ],
            )*
        }
    } => {
        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                match self {
                    $(
                        &Self::$variant { $(ref $field,)* .. } => {
                            let mut f = f.debug_struct(::std::stringify!($name));
                            $(f.field(::std::stringify!($field), $field);)*
                            $crate::impl_partial_debug!(@enum_finish $($modifier)? f)
                        }
                    )*
                }
            }
        }
    };
}
