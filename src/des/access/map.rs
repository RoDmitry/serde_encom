use crate::des::wrapper::MapKeyDeserializer;
use crate::des::{deserializer::Deserializer, read::Read};
use crate::error::{Error, ErrorCode, Result};
use serde::de;

pub(crate) struct MapAccess<'a, R: 'a> {
    pub(super) des: &'a mut Deserializer<R>,
}

impl<'a, R: 'a> MapAccess<'a, R> {
    pub(crate) fn new(des: &'a mut Deserializer<R>) -> Self {
        MapAccess { des }
    }
}

impl<'de, 'a, R: Read<'de> + 'a> de::MapAccess<'de> for MapAccess<'a, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.des.parse_whitespace()? {
            Some(b'}') => Ok(None),
            /* Some(b' ') if !self.first => {
                self.de.eat_char();
                self.de.parse_whitespace()?
            }
            Some(b) => {
                if self.first {
                    self.first = false;
                    Some(b)
                } else {
                    return Err(self.de.peek_error(ErrorCode::ExpectedObjectCommaOrEnd));
                }
            } */
            Some(_) => seed
                .deserialize(MapKeyDeserializer { des: self.des })
                .map(Some),
            None => Err(self.des.peek_error(ErrorCode::EofWhileParsingObject)),
        }
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.des.parse_object_colon()?;

        seed.deserialize(&mut *self.des)
    }
}
