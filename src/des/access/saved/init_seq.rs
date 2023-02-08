use crate::des::deserializer::SavedType;
use crate::des::wrapper::SavedSeqDeserializer;
use crate::des::{deserializer::Deserializer, read::Read};
use crate::error::{Error, ErrorCode, Result};
use serde::de;

pub(crate) struct SavedInitSeqAccess<'a, R: 'a> {
    des: &'a mut Deserializer<R>,
    saved_type: SavedType,
}

impl<'a, R: 'a> SavedInitSeqAccess<'a, R> {
    pub(crate) fn new(des: &'a mut Deserializer<R>, saved_type: SavedType) -> Self {
        SavedInitSeqAccess { des, saved_type }
    }
}

impl<'de, 'a, R: Read<'de> + 'a> de::SeqAccess<'de> for SavedInitSeqAccess<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.des.parse_whitespace()? {
            Some(b'}') => return Err(self.des.peek_error(ErrorCode::TrailingComma)), // todo new error
            None => {
                if self.saved_type == SavedType::None {
                    return Ok(None);
                }
            }
            _ => {}
        }
        seed.deserialize(SavedSeqDeserializer {
            des: self.des,
            saved_type: &mut self.saved_type,
        })
        .map(Some)
    }
}
