//! Module for the `raw_type` macro.

/// Defines a pair of an ABI-compatible raw newtype and a corresponding
/// convenient, high-level open-set enum, along with all conversions between
/// the newtype, the enum, and the underlying integer.
///
/// The newtype behaves like the plain integer (`Copy`, comparisons, and
/// conversions) but carries the semantics of the enum: [`Debug`] prints the
/// variant name together with the raw value (e.g. `Foo(0)`) and [`Display`]
/// prints just the variant name (e.g. `Foo`); values without a specified
/// semantic print as `Custom(x)`. It is safe to use in `#[repr(C)]`
/// structures parsed from raw memory, as every bit pattern is valid for it. The enum assigns each specified value to a
/// variant; all other values are mapped to the automatically added `Custom`
/// variant, which carries the raw integer. By convention, the newtype
/// carries the name of the enum plus a `Raw` suffix.
///
/// [`Debug`]: core::fmt::Debug
/// [`Display`]: core::fmt::Display
///
/// # Example
///
/// ```
/// multiboot2_common::raw_type! {
///     /// Binary representation of a demo type.
///     pub struct DemoTypeRaw(u32);
///
///     /// High-level abstraction of the possible demo types.
///     pub enum DemoType {
///         /// The first defined type.
///         Foo = 0,
///         /// The second defined type.
///         Bar = 1,
///     }
/// }
///
/// let raw = DemoTypeRaw::new(1);
/// assert_eq!(raw, DemoType::Bar);
/// assert_eq!(DemoType::from(DemoTypeRaw::new(42)), DemoType::Custom(42));
/// ```
#[macro_export]
macro_rules! raw_type {
    (
        $(#[$raw_attr:meta])*
        $raw_vis:vis struct $Raw:ident($int:ty);

        $(#[$enum_attr:meta])*
        $enum_vis:vis enum $Enum:ident {
            $(
                $(#[$variant_attr:meta])*
                $Variant:ident = $value:literal,
            )+
        }
    ) => {
        $(#[$raw_attr])*
        #[repr(transparent)]
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $raw_vis struct $Raw($int);

        impl $Raw {
            /// Constructs a new instance from the raw binary value.
            #[must_use]
            pub const fn new(val: $int) -> Self {
                Self(val)
            }

            /// Returns the raw binary value.
            #[must_use]
            pub const fn get(self) -> $int {
                self.0
            }
        }

        impl ::core::fmt::Debug for $Raw {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Debug::fmt(&$Enum::from_val(self.0), f)
            }
        }

        impl ::core::fmt::Display for $Raw {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Display::fmt(&$Enum::from_val(self.0), f)
            }
        }

        $(#[$enum_attr])*
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $enum_vis enum $Enum {
            $(
                $(#[$variant_attr])*
                $Variant,
            )+
            /// Value without a specified semantic in the specification.
            Custom($int),
        }

        impl $Enum {
            /// Returns the raw binary value.
            #[must_use]
            pub const fn val(self) -> $int {
                match self {
                    $(Self::$Variant => $value,)+
                    Self::Custom(val) => val,
                }
            }

            /// Constructs the variant corresponding to the raw binary value.
            ///
            /// Values without a specified semantic are mapped to
            /// [`Self::Custom`].
            #[must_use]
            pub const fn from_val(val: $int) -> Self {
                match val {
                    $($value => Self::$Variant,)+
                    val => Self::Custom(val),
                }
            }
        }

        impl ::core::fmt::Debug for $Enum {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    $(Self::$Variant => f.debug_tuple(stringify!($Variant)).field(&$value).finish(),)+
                    Self::Custom(val) => f.debug_tuple("Custom").field(val).finish(),
                }
            }
        }

        impl ::core::fmt::Display for $Enum {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    $(Self::$Variant => f.write_str(stringify!($Variant)),)+
                    Self::Custom(val) => write!(f, "Custom({val})"),
                }
            }
        }

        impl ::core::convert::From<$int> for $Raw {
            fn from(val: $int) -> Self {
                Self::new(val)
            }
        }

        impl ::core::convert::From<$Raw> for $int {
            fn from(raw: $Raw) -> Self {
                raw.get()
            }
        }

        impl ::core::convert::From<$int> for $Enum {
            fn from(val: $int) -> Self {
                Self::from_val(val)
            }
        }

        impl ::core::convert::From<$Enum> for $int {
            fn from(val: $Enum) -> Self {
                val.val()
            }
        }

        impl ::core::convert::From<$Raw> for $Enum {
            fn from(raw: $Raw) -> Self {
                Self::from_val(raw.get())
            }
        }

        impl ::core::convert::From<$Enum> for $Raw {
            fn from(val: $Enum) -> Self {
                Self::new(val.val())
            }
        }

        impl ::core::cmp::PartialEq<$Enum> for $Raw {
            fn eq(&self, other: &$Enum) -> bool {
                self.0 == other.val()
            }
        }

        impl ::core::cmp::PartialEq<$Raw> for $Enum {
            fn eq(&self, other: &$Raw) -> bool {
                self.val() == other.0
            }
        }

        impl ::core::cmp::PartialEq<$int> for $Raw {
            fn eq(&self, other: &$int) -> bool {
                self.0 == *other
            }
        }

        impl ::core::cmp::PartialEq<$Raw> for $int {
            fn eq(&self, other: &$Raw) -> bool {
                *self == other.0
            }
        }

        impl ::core::cmp::PartialEq<$int> for $Enum {
            fn eq(&self, other: &$int) -> bool {
                self.val() == *other
            }
        }

        impl ::core::cmp::PartialEq<$Enum> for $int {
            fn eq(&self, other: &$Enum) -> bool {
                *self == other.val()
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};

    crate::raw_type! {
        /// Binary representation of a test type.
        pub struct TestRaw(u16);

        /// High-level abstraction of a test type.
        pub enum TestType {
            /// The first defined value.
            Foo = 0,
            /// A defined value with a gap to the previous one.
            Bar = 42,
        }
    }

    /// The newtype must be binary compatible with the underlying integer.
    #[test]
    fn test_layout() {
        assert_eq!(size_of::<TestRaw>(), size_of::<u16>());
        assert_eq!(align_of::<TestRaw>(), align_of::<u16>());
    }

    #[test]
    fn test_const_constructors_and_getters() {
        const RAW: TestRaw = TestRaw::new(42);
        const VAL: u16 = RAW.get();
        const TYP: TestType = TestType::from_val(VAL);
        assert_eq!(VAL, 42);
        assert_eq!(TYP, TestType::Bar);
        assert_eq!(TYP.val(), 42);
    }

    /// Every raw value must be constructible and must round-trip through
    /// the newtype and the enum, including values unknown to the
    /// specification.
    #[test]
    fn test_roundtrip() {
        for val in [0_u16, 42, 1337, u16::MAX] {
            let raw = TestRaw::from(val);
            let typ = TestType::from(raw);
            assert_eq!(u16::from(raw), val);
            assert_eq!(u16::from(typ), val);
            assert_eq!(TestRaw::from(typ), raw);
        }
    }

    #[test]
    fn test_from_val() {
        assert_eq!(TestType::from_val(0), TestType::Foo);
        assert_eq!(TestType::from_val(42), TestType::Bar);
        assert_eq!(TestType::from_val(7), TestType::Custom(7));
        assert_eq!(TestType::Foo.val(), 0);
        assert_eq!(TestType::Bar.val(), 42);
        assert_eq!(TestType::Custom(7).val(), 7);
    }

    /// All three representations must be comparable with each other.
    #[test]
    fn test_partial_eq() {
        assert_eq!(TestRaw::new(42), TestType::Bar);
        assert_eq!(TestType::Bar, TestRaw::new(42));
        assert_eq!(TestRaw::new(42), 42);
        assert_eq!(42, TestRaw::new(42));
        assert_eq!(TestType::Bar, 42);
        assert_eq!(42, TestType::Bar);
        assert_eq!(TestRaw::new(7), TestType::Custom(7));
        assert_ne!(TestRaw::new(0), TestType::Bar);
    }

    /// The types must debug-print the semantic of their value together
    /// with the raw value.
    #[test]
    fn test_debug() {
        assert_eq!(format!("{:?}", TestRaw::new(0)), "Foo(0)");
        assert_eq!(format!("{:?}", TestRaw::new(7)), "Custom(7)");
        assert_eq!(format!("{:?}", TestType::Bar), "Bar(42)");
    }

    /// The types must display-print just the semantic of their value.
    #[test]
    fn test_display() {
        assert_eq!(format!("{}", TestRaw::new(0)), "Foo");
        assert_eq!(format!("{}", TestRaw::new(7)), "Custom(7)");
        assert_eq!(format!("{}", TestType::Bar), "Bar");
    }

    /// Both types must be usable in ordered and hashed collections.
    #[test]
    fn test_ord_and_hash() {
        let set = BTreeSet::from([TestType::Bar, TestType::Foo, TestType::Bar]);
        assert!(set.iter().zip(set.iter().skip(1)).all(|(a, b)| a < b));

        let set = HashSet::from([TestRaw::new(0), TestRaw::new(1)]);
        assert_eq!(set.len(), 2);
    }
}
