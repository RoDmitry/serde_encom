use crate::des::{deserializer::Deserializer, read::Read};
use crate::error::{Error, ErrorCode, Result};
use serde::de;

pub(crate) struct InitSeqAccess<'a, R: 'a> {
    pub(super) des: &'a mut Deserializer<R>,
}

impl<'a, R: 'a> InitSeqAccess<'a, R> {
    pub(crate) fn new(des: &'a mut Deserializer<R>) -> Self {
        InitSeqAccess { des }
    }
}

impl<'de, 'a, R: Read<'de> + 'a> de::SeqAccess<'de> for InitSeqAccess<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.des.parse_whitespace()? {
            /*  Some(b' ') if !self.first => {
                self.de.eat_char();
                self.de.parse_whitespace()?
            }
            Some(b) => {
                if self.first {
                    self.first = false;
                    Some(b)
                } else {
                    return Err(self.de.peek_error(ErrorCode::ExpectedListCommaOrEnd));
                }
            } */
            Some(b']') => Err(self.des.peek_error(ErrorCode::TrailingComma)), // todo new error
            Some(_) => seed.deserialize(&mut *self.des).map(Some),
            None => Ok(None),
        }

        /* match peek {
            Some(b']') => Err(self.de.peek_error(ErrorCode::TrailingComma)),
            Some(_) => Ok(Some(seed.deserialize(&mut *self.de)?)),
            None => Err(self.de.peek_error(ErrorCode::EofWhileParsingValue)),
        } */
    }
}
