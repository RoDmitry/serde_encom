use crate::{Value, ValueVisitor};
use serde::de::DeserializeSeed;
use std::result::Result;

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
