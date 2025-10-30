use crate::des::deserializer::SavedType;
use crate::des::wrapper::SavedSeqDeserializer;
use crate::des::{deserializer::Deserializer, read::Read};
use crate::error::{Error, ErrorCode, Result};
use serde::de;

pub(crate) struct SavedSeqAccess<'a, R: 'a> {
    des: &'a mut Deserializer<R>,
    saved_type: SavedType,
}

impl<'a, R: 'a> SavedSeqAccess<'a, R> {
    pub(crate) fn new(des: &'a mut Deserializer<R>, saved_type: SavedType) -> Self {
        SavedSeqAccess { des, saved_type }
    }
}

impl<'de, 'a, R: Read<'de> + 'a> de::SeqAccess<'de> for SavedSeqAccess<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.des.parse_whitespace()? {
            Some(b']') => {
                if self.saved_type == SavedType::None {
                    return Ok(None);
                }
            }
            None => return Err(self.des.peek_error(ErrorCode::EofWhileParsingList)),
            _ => {}
        }
        seed.deserialize(SavedSeqDeserializer {
            des: self.des,
            saved_type: &mut self.saved_type,
        })
        .map(Some)
    }
}
