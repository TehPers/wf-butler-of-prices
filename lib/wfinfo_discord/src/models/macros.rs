#[macro_export]
macro_rules! serde_inner_enum {
    (@count $field_name:ident ?) => {
        if $field_name.is_some() { 1 } else { 0 }
    };
    (@count $field_name:ident) => {
        1
    };
    (@ser $serializer:expr, $field_name:ident ?) => {
        if $field_name.is_some() {
            $serializer.serialize_field(stringify!($field_name), $field_name)
        } else {
            Ok(())
        }
    };
    (@ser $serializer:expr, $field_name:ident) => {
        $serializer.serialize_field(stringify!($field_name), $field_name)
    };
    (@de $entries:expr, $field_name:ident ?) => {
        $entries
            .remove(stringify!($field_name))
            .map(|value| {
                Deserialize::deserialize(value)
                    .map_err(|error| DeError::custom(error.to_string()))
            })
            .transpose()
    };
    (@de $entries:expr, $field_name:ident) => {
        $entries
            .remove(stringify!($field_name))
            .ok_or_else(|| DeError::missing_field(stringify!($field_name)))
            .and_then(|value| {
                Deserialize::deserialize(value)
                    .map_err(|error| DeError::custom(error.to_string()))
            })
    };
    (
        $(#[$($attr:meta),*])*
        $v:vis enum $name:ident = $tag_name:literal {
            $(
                $(#[$($variant_attr:meta),*])*
                $variant_name:ident = $tag_value:literal $({
                    $(
                        $(#[$($field_attr:meta),*])*
                        $([$mod:tt])?
                        $field_name:ident : $field_type:ty
                    ),*
                    $(,)?
                })?
            ),*
            $(,)?
        }
    ) => {
        $(#[$($attr),*])*
        $v enum $name {
            $(
                $(#[$($variant_attr),*])*
                $variant_name $({
                    $(
                        $(#[$($field_attr),*])*
                        $field_name: $field_type,
                    )*
                })?,
            )*
        }

        const _: () = {
            use serde::{
                de::Error as DeError,
                ser::SerializeStruct,
                Deserialize, Deserializer, Serialize, Serializer
            };
            use serde_json::Value;
            use std::collections::HashMap;

            impl Serialize for $name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    match self {
                        $(
                            $name::$variant_name $({ $(ref $field_name,)* })? => {
                                // Calculate length
                                #[allow(unused_mut)]
                                let mut len = 1;
                                $($(len += $crate::serde_inner_enum!(@count $field_name $($mod)?);)*)?

                                // Serialize fields
                                let mut s = serializer.serialize_struct(stringify!($name), len)?;
                                s.serialize_field($tag_name, &$tag_value)?;
                                $($($crate::serde_inner_enum!(@ser s, $field_name $($mod)?)?;)*)?
                                s.end()
                            },
                        )*
                    }
                }
            }

            impl<'de> Deserialize<'de> for $name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    // Collect into a map
                    let mut entries: HashMap<String, Value> =
                        Deserialize::deserialize(deserializer)?;

                    // Get the tag
                    let tag = entries
                        .remove($tag_name)
                        .ok_or_else(|| DeError::missing_field($tag_name))?;
                    let tag: u8 = tag
                        .as_u64()
                        .and_then(|tag| tag.try_into().ok())
                        .ok_or_else(|| DeError::custom(concat!("invalid value for field '", $tag_name, "'")))?;

                    // Deserialize variant
                    match tag {
                        $(
                            $tag_value => {
                                // Deserialize fields
                                $($(let $field_name = $crate::serde_inner_enum!(@de entries, $field_name $($mod)?)?;)*)?

                                Ok($name::$variant_name $({
                                    $($field_name,)*
                                })?)
                            },
                        )*
                        _ => Err(DeError::custom(format!("invalid tag: '{}'", tag))),
                    }
                }
            }
        };
    };
}
