use crate::des::deserializer::ScratchState;
use crate::des::wrapper::ScratchDeserializer;
use crate::des::{deserializer::Deserializer, read::Read};
use crate::error::{Error, ErrorCode, Result};
use serde::de;

pub(crate) struct ScratchInitSeqAccess<'a, R: 'a> {
    des: &'a mut Deserializer<R>,
    state: ScratchState,
}

impl<'a, R: 'a> ScratchInitSeqAccess<'a, R> {
    pub(crate) fn new(des: &'a mut Deserializer<R>, state: ScratchState) -> Self {
        ScratchInitSeqAccess { des, state }
    }
}

impl<'de, 'a, R: Read<'de> + 'a> de::SeqAccess<'de> for ScratchInitSeqAccess<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.des.parse_whitespace()? {
            Some(b'}') => Err(self.des.peek_error(ErrorCode::TrailingComma)), // todo new error
            Some(_) => seed
                .deserialize(ScratchDeserializer {
                    des: self.des,
                    state: &mut self.state,
                })
                .map(Some),
            None => {
                if self.state == ScratchState::None {
                    Ok(None)
                } else {
                    seed.deserialize(ScratchDeserializer {
                        des: self.des,
                        state: &mut self.state,
                    })
                    .map(Some)
                }
            }
        }
    }
}
