use crate::{Value, ValueVisitor};
use serde::de::DeserializeSeed;
use std::result::Result;

pub(crate) struct StrSeed;

impl<'de> DeserializeSeed<'de> for StrSeed {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(ValueVisitor)
    }
}
