use crate::des::access::{
    InitMapAccess, InitSeqAccess, SavedInitMapAccess, SavedInitSeqAccess, VariantAccess,
};
use crate::des::deserializer::{Deserializer, PreParser};
use crate::des::read::Read;
use crate::error::{Error, ErrorCode, Result};
#[cfg(feature = "float_roundtrip")]
use crate::lexical;
use serde::de;

pub struct InitDeserializer<'a, R> {
    pub(crate) des: &'a mut Deserializer<R>,
}

impl<'de, 'a, R: Read<'de>> de::Deserializer<'de> for InitDeserializer<'a, R> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        /* let peek = match self.des.parse_whitespace()? {
            Some(b) => b,
            None => {
                return Err(self.des.peek_error(ErrorCode::EofWhileParsingValue));
            }
        }; */

        let value = match self.des.any_after_x7b()? {
            PreParser::SavedMap => {
                let value = visitor.visit_map(SavedInitMapAccess::new(self.des));
                (value, self.des.end_map_init())
            }
            PreParser::SavedSeq(s) => {
                let value = visitor.visit_seq(SavedInitSeqAccess::new(self.des, s));
                (value, self.des.end_seq_init())
            }
            PreParser::Seq => {
                let value = visitor.visit_seq(InitSeqAccess::new(self.des));
                (value, self.des.end_seq_init())
            }
        };

        /* let value = match peek {
            b'n' => {
                self.des.eat_char();
                visitor.visit_unit()
            }
            b'-' => {
                self.des.eat_char();
                self.des.parse_any_number(false)?.visit(visitor)
            }
            b'0'..=b'9' => self.des.parse_any_number(true)?.visit(visitor),
            b'{' => {
                let value = visitor.visit_map(InitMapAccess::new(self.des));
                match (value, self.des.end_map_flat()) {
                    (Ok(ret), Ok(())) => Ok(ret),
                    (Err(err), _) | (_, Err(err)) => Err(err),
                }
            }
            _ => Err(self.des.peek_error(ErrorCode::ExpectedSomeValue)),
        }; */

        match value {
            (Ok(ret), Ok(())) => Ok(ret),
            // The de::Error impl creates errors with unknown line and column.
            // Fill in the position here by looking at the current index in the
            // input. There is no way to tell whether this should call `error`
            // or `peek_error` so pick the one that seems correct more often.
            // Worst case, the position is off by one character.
            (Err(err), _) | (_, Err(err)) => Err(self.des.fix_position(err)),
        }
    }

    to_inner_des_method!(deserialize_bool);
    to_inner_des_method!(deserialize_i8);
    to_inner_des_method!(deserialize_i16);
    to_inner_des_method!(deserialize_i32);
    to_inner_des_method!(deserialize_i64);
    to_inner_des_method!(deserialize_u8);
    to_inner_des_method!(deserialize_u16);
    to_inner_des_method!(deserialize_u32);
    to_inner_des_method!(deserialize_u64);
    to_inner_des_method!(deserialize_f32);
    to_inner_des_method!(deserialize_f64);
    to_inner_des_method!(deserialize_i128);
    to_inner_des_method!(deserialize_u128);
    to_inner_des_method!(deserialize_char);
    to_inner_des_method!(deserialize_str);
    to_inner_des_method!(deserialize_string);
    to_inner_des_method!(deserialize_bytes);
    to_inner_des_method!(deserialize_byte_buf);
    to_inner_des_method!(deserialize_option);
    to_inner_des_method!(deserialize_unit);
    to_inner_des_method!(deserialize_identifier);
    to_inner_des_method!(deserialize_ignored_any);

    #[inline]
    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.des.deserialize_unit_struct(name, visitor)
    }

    /// Parses a newtype struct as the underlying value.
    #[inline]
    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.des.deserialize_newtype_struct(name, visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let peek = match self.des.parse_whitespace()? {
            Some(b) => b,
            None => {
                return Err(self.des.peek_error(ErrorCode::EofWhileParsingValue));
            }
        };

        let value = visitor.visit_seq(InitSeqAccess::new(self.des));
        match (value, self.des.end_seq_init()) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(err), _) | (_, Err(err)) => Err(self.des.fix_position(err)),
        }
    }

    #[inline]
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let peek = match self.des.parse_whitespace()? {
            Some(b) => b,
            None => {
                return Err(self.des.peek_error(ErrorCode::EofWhileParsingValue));
            }
        };

        let value = visitor.visit_map(InitMapAccess::new(self.des));
        match (value, self.des.end_map_init()) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(err), _) | (_, Err(err)) => Err(self.des.fix_position(err)),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let peek = match self.des.parse_whitespace()? {
            Some(b) => b,
            None => {
                return Err(self.des.peek_error(ErrorCode::EofWhileParsingValue));
            }
        };

        let value = visitor.visit_map(InitMapAccess::new(self.des));
        match (value, self.des.end_map_init()) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(err), _) | (_, Err(err)) => Err(self.des.fix_position(err)),
        }
    }

    /// Parses an enum as an object like `{"$KEY":$VALUE}`, where $VALUE is either a straight
    /// value, a `[..]`, or a `{..}`.
    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.des.parse_whitespace()? {
            Some(_) => visitor.visit_enum(VariantAccess::new(self.des)),
            None => Err(self.des.peek_error(ErrorCode::EofWhileParsingValue)),
        }
    }
}
