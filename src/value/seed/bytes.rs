/* use crate::{Value, ValueVisitor};
use serde::de::DeserializeSeed;
use std::result;

pub(crate) struct BytesSeed<T>;

impl<'de, T> DeserializeSeed<'de> for BytesSeed<T> {
    type Value = T;

    fn deserialize<D>(self, deserializer: D) -> result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(ValueVisitor)
    }
} */

/* use crate::{Value, ValueVisitor};
use serde::de::{DeserializeSeed, Visitor};
use std::result;

pub(crate) struct BytesSeed<V> {
    visitor: V,
}

impl<'de, V> DeserializeSeed<'de> for BytesSeed<V>
where
    V: Visitor<'de>,
{
    type Value = V::Value;

    fn deserialize<D>(self, deserializer: D) -> result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(self.visitor)
    }
} */

use crate::{Value, ValueVisitor};
use serde::de::{self, DeserializeSeed, Visitor};
use std::{fmt, result};

/* enum KeyClass {
    String(String),
    Bytes(Vec<u8>),
    #[cfg(feature = "arbitrary_precision")]
    Number,
    #[cfg(feature = "raw_value")]
    RawValue,
} */

pub(crate) struct BytesSeed;

impl<'de> DeserializeSeed<'de> for BytesSeed {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(ValueVisitor)
    }
}

/* impl<'de> Visitor<'de> for BytesSeed {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string key")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match s {
            #[cfg(feature = "arbitrary_precision")]
            crate::number::TOKEN => Ok(KeyClass::Number),
            #[cfg(feature = "raw_value")]
            crate::raw::TOKEN => Ok(KeyClass::RawValue),
            _ => Ok(KeyClass::Map(s.to_owned())),
        }
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match s.as_str() {
            #[cfg(feature = "arbitrary_precision")]
            crate::number::TOKEN => Ok(KeyClass::Number),
            #[cfg(feature = "raw_value")]
            crate::raw::TOKEN => Ok(KeyClass::RawValue),
            _ => Ok(KeyClass::Map(s)),
        }
    }
} */
