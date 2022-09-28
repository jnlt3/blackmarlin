// This module is never exposed by the main crate.
#![allow(missing_docs)]

macro_rules! simple_enum {
    (
        $(#[$attr:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_attr:meta])*
                $variant:ident
            ),*
        }
    ) => {
        $(#[$attr])*
        $vis enum $name {
            $(
                $(#[$variant_attr])*
                $variant
            ),*
        }

        impl $name {
            #[doc = concat!("The number of [`", stringify!($name), "`] variants.")]
            pub const NUM: usize = [$(Self::$variant),*].len();
            #[doc = concat!("An array of all [`", stringify!($name), "`] variants.")]
            pub const ALL: [Self; Self::NUM] = [$(Self::$variant),*];

            #[doc = concat!("Checked version of [`", stringify!($name), "::index`].")]
            #[inline(always)]
            pub const fn try_index(index: usize) -> Option<Self> {
                mod variant_indexes {
                    #![allow(non_upper_case_globals, unused)]
                    $(pub const $variant: usize = super::$name::$variant as usize;)*
                }
                #[allow(non_upper_case_globals)]
                match index {
                    $(variant_indexes::$variant => Some(Self::$variant),)*
                    _ => None
                }
            }

            #[doc = concat!(
                "Convert an index to a [`", stringify!($name), "`].\n",
                "# Panics\n",
                "Panic if the index is out of bounds."
            )]
            #[inline(always)]
            pub fn index(index: usize) -> Self {
                Self::try_index(index).unwrap_or_else(|| panic!("Index {} is out of range.", index))
            }

            #[doc = concat!(
                "`const` version of [`", stringify!($name), "::index`].\n",
                "# Panics\n",
                "Panic if the index is out of bounds."
            )]
            #[inline(always)]
            pub const fn index_const(index: usize) -> Self {
                if let Some(value) = Self::try_index(index) {
                    value
                } else {
                    panic!("Index is out of range")
                }
            }
        }
    };
}
pub(crate) use simple_enum;

macro_rules! enum_char_conv {
    (
        $enum:ident, $error:ident {
            $($variant:ident = $char:expr),*
        }
    ) => {
        impl From<$enum> for char {
            fn from(value: $enum) -> Self {
                match value {
                    $($enum::$variant => $char),*
                }
            }
        }

        $crate::simple_error! {
            #[doc = concat!("The value was not a valid [`", stringify!($enum), "`].")]
            pub struct $error = concat!("The value was not a valid `", stringify!($enum), "`.");
        }

        impl core::convert::TryFrom<char> for $enum {
            type Error = $error;

            fn try_from(value: char) -> Result<Self, Self::Error> {
                match value {
                    $($char => Ok(Self::$variant),)*
                    _ => Err($error)
                }
            }
        }

        impl core::str::FromStr for $enum {
            type Err = $error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                use core::convert::TryInto;

                let mut chars = s.chars();
                let c = chars.next().ok_or($error)?;
                if chars.next().is_none() {
                    c.try_into()
                } else {
                    Err($error)
                }
            }
        }

        impl core::fmt::Display for $enum {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                let c: char = (*self).into();
                c.fmt(f)
            }
        }
    };
}
pub(crate) use enum_char_conv;

#[macro_export]
macro_rules! simple_error {
    (
        $(#[$attr:meta])*
        $vis:vis enum $error:ident {
            $($variant:ident = $string:expr),*
        }
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy)]
        $vis enum $error {
            $(#[doc = $string] $variant),*
        }

        impl core::fmt::Display for $error {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                match self {
                    $(Self::$variant => write!(f, $string)),*
                }
            }
        }

        #[cfg(feature = "std")]
        impl std::error::Error for $error {}
    };

    (
        $(#[$attr:meta])*
        $vis:vis struct $error:ident = $string:expr;
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy)]
        $vis struct $error;

        impl core::fmt::Display for $error {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(f, $string)
            }
        }

        #[cfg(feature = "std")]
        impl std::error::Error for $error {}
    };
}
pub use simple_error;
