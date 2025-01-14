//! Implements NBT serialisation and deserialization for three different integer encodings.

pub use crate::de::{from_be_bytes, from_bytes, from_le_bytes, from_net_bytes, Deserializer};
pub use crate::ser::{
    to_be_bytes, to_be_bytes_in, to_bytes, to_bytes_in, to_le_bytes, to_le_bytes_in, to_net_bytes,
    to_net_bytes_in, Serializer,
};
pub use crate::value::Value;
pub use byteorder::{BigEndian, LittleEndian};

use std::borrow::Cow;
use std::fmt::{Debug, Display};

pub use error::NbtError;

#[cfg(test)]
mod test;

mod de;
mod error;
mod ser;
mod value;

mod private {
    use byteorder::{BigEndian, LittleEndian};

    use crate::{EndiannessImpl, NetworkLittleEndian, Variant};

    /// Prevents [`VariantImpl`](super::VariantImpl) from being implemented for
    /// types outside of this crate.
    pub trait Sealed {}

    impl Sealed for BigEndian {}
    impl EndiannessImpl for BigEndian {
        const AS_ENUM: Variant = Variant::BigEndian;
    }

    impl Sealed for LittleEndian {}
    impl EndiannessImpl for LittleEndian {
        const AS_ENUM: Variant = Variant::LittleEndian;
    }

    impl Sealed for NetworkLittleEndian {}
    impl EndiannessImpl for NetworkLittleEndian {
        const AS_ENUM: Variant = Variant::NetworkEndian;
    }
}

/// Implemented by all NBT variants.
pub trait EndiannessImpl: private::Sealed {
    /// Used to convert a variant to an enum.
    /// This is used to match generic types in order to prevent
    /// having to duplicate all deserialisation code three times.
    const AS_ENUM: Variant;
}

/// NBT format variant.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Variant {
    /// Used by Bedrock for data saved to disk.
    /// Every data type is written in little endian format.
    LittleEndian,
    /// Used by Java.
    /// Every data types is written in big endian format.
    BigEndian,
    /// Used by Bedrock for NBT transferred over the network.
    /// This format is the same as [`LittleEndian`], except that type lengths
    /// (such as for strings or lists), are varints instead of shorts.
    /// The integer and long types are also varints.
    NetworkEndian,
}

/// Used by Bedrock for NBT transferred over the network.
/// This format is the same as [`LittleEndian`], except that type lengths
/// (such as for strings or lists), are varints instead of shorts.
/// The integer and long types are also varints.
pub enum NetworkLittleEndian {}

/// NBT field type
// Compiler complains about unused enum variants even though they're constructed using a transmute.
#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum FieldType {
    /// Indicates the end of a compound tag.
    End = 0,
    /// A signed byte.
    Byte = 1,
    /// A signed short.
    Short = 2,
    /// A signed int.
    Int = 3,
    /// A signed long.
    Long = 4,
    /// A float.
    Float = 5,
    /// A double.
    Double = 6,
    /// An array of byte tags.
    ByteArray = 7,
    /// A UTF-8 string.
    String = 8,
    /// List of tags.
    /// Every item in the list must be of the same type.
    List = 9,
    /// A key-value map.
    Compound = 10,
    /// An array of int tags.
    IntArray = 11,
    /// An array of long tags.
    LongArray = 12,
}

impl TryFrom<u8> for FieldType {
    type Error = NbtError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        const LAST_DISC: u8 = FieldType::LongArray as u8;
        if v > LAST_DISC {
            return Err(NbtError::TypeOutOfRange { actual: v });
        }

        // SAFETY: Because `Self` is marked as `repr(u8)`, its layout is guaranteed to start
        // with a `u8` discriminant as its first field. Additionally, the raw discriminant is verified
        // to be in the enum's range.
        Ok(unsafe { std::mem::transmute::<u8, FieldType>(v) })
    }
}

impl serde::de::Error for NbtError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        NbtError::Other(Cow::Owned(msg.to_string()))
    }
}

impl serde::ser::Error for NbtError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        NbtError::Other(Cow::Owned(msg.to_string()))
    }
}
