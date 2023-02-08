use super::super::InitMapAccess;
use crate::des::wrapper::SavedMapKeyDeserializer;
use crate::des::{deserializer::Deserializer, read::Read};
use crate::error::{Error, Result};
use serde::de;

pub(crate) struct SavedInitMapAccess<'a, R: 'a> {
    des: InitMapAccess<'a, R>,
}

impl<'a, R: 'a> SavedInitMapAccess<'a, R> {
    pub(crate) fn new(des: &'a mut Deserializer<R>) -> Self {
        SavedInitMapAccess {
            des: InitMapAccess { des },
        }
    }
}

impl<'de, 'a, R: Read<'de> + 'a> de::MapAccess<'de> for SavedInitMapAccess<'a, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.des.des.read.saved_is_empty() {
            self.des.next_key_seed(seed)
        } else {
            seed.deserialize(SavedMapKeyDeserializer { des: self.des.des })
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
