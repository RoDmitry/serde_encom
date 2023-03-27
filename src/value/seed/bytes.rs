use crate::{Value, ValueVisitor};
use core::result::Result;
use serde::de::DeserializeSeed;

pub(crate) struct BytesSeed;

impl<'de> DeserializeSeed<'de> for BytesSeed {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(ValueVisitor)
    }
}
