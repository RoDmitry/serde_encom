use super::super::MapAccess;
use crate::des::wrapper::ScratchMapKeyDeserializer;
use crate::des::{deserializer::Deserializer, read::Read};
use crate::error::{Error, Result};
use serde::de;

pub(crate) struct ScratchMapAccess<'a, R: 'a> {
    des: MapAccess<'a, R>,
}

impl<'a, R: 'a> ScratchMapAccess<'a, R> {
    pub(crate) fn new(des: &'a mut Deserializer<R>) -> Self {
        ScratchMapAccess {
            des: MapAccess { des },
        }
    }
}

impl<'de, 'a, R: Read<'de> + 'a> de::MapAccess<'de> for ScratchMapAccess<'a, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.des.des.scratch.is_empty() {
            self.des.next_key_seed(seed)
        } else {
            seed.deserialize(ScratchMapKeyDeserializer { des: self.des.des })
                .map(Some)
        }
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.des.next_value_seed(seed)
    }
}
