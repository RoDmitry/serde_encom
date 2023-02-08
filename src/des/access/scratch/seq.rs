use crate::des::deserializer::ScratchState;
use crate::des::wrapper::SavedSeqDeserializer;
use crate::des::{deserializer::Deserializer, read::Read};
use crate::error::{Error, ErrorCode, Result};
use serde::de;

pub(crate) struct ScratchSeqAccess<'a, R: 'a> {
    des: &'a mut Deserializer<R>,
    state: ScratchState,
}

impl<'a, R: 'a> ScratchSeqAccess<'a, R> {
    pub(crate) fn new(des: &'a mut Deserializer<R>, state: ScratchState) -> Self {
        ScratchSeqAccess { des, state }
    }
}

impl<'de, 'a, R: Read<'de> + 'a> de::SeqAccess<'de> for ScratchSeqAccess<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.des.parse_whitespace()? {
            Some(b'}') => {
                if self.state == ScratchState::None {
                    return Ok(None);
                }
            }
            None => return Err(self.des.peek_error(ErrorCode::EofWhileParsingList)),
            _ => {}
        }
        seed.deserialize(SavedSeqDeserializer {
            des: self.des,
            state: &mut self.state,
        })
        .map(Some)
    }
}
