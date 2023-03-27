use crate::{Value, ValueVisitor};
use core::result::Result;
use serde::de::DeserializeSeed;

pub(crate) struct U64Seed;

impl<'de> DeserializeSeed<'de> for U64Seed {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_u64(ValueVisitor)
    }
}
