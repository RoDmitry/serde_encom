use crate::{Value, ValueVisitor};
use serde::de::DeserializeSeed;
use std::result::Result;

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
