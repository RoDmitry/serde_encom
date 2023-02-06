use crate::des::deserializer::{Deserializer, ScratchState};
use crate::des::read::{Read, StrOrBytes};
use crate::error::{Error, ErrorCode, Result};
#[cfg(feature = "float_roundtrip")]
use crate::lexical;
use crate::to_inner_des_method;
use atoi_simd::parse;
use serde::de;

pub(crate) struct ScratchDeserializer<'a, 's, R> {
    pub(crate) des: &'a mut Deserializer<R>,
    pub(crate) state: &'s mut ScratchState,
}

impl<'de, 'a, R: Read<'de>> de::Deserializer<'de> for ScratchDeserializer<'a, '_, R> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.des.scratch.is_empty() {
            return Err(self.des.peek_error(ErrorCode::EofWhileParsingList)); // todo: change err?
        }

        let ret = match self.state {
            ScratchState::Str | ScratchState::Bytes => {
                let len = parse(self.des.scratch.get_slice())?;
                let value = {
                    self.des.eat_char();
                    let res = match self.des.read.str_or_bytes(len)? {
                        StrOrBytes::Str(v) => visitor.visit_borrowed_str(v),
                        StrOrBytes::Bytes(v) => visitor.visit_borrowed_bytes(v),
                    };

                    // checks for ' ' or '}' after slice
                    match self.des.peek()? {
                        Some(b' ') | Some(b'}') | Some(b'\n') | Some(b'\t') | Some(b'\r')
                        | None => res,
                        _ => Err(self.des.peek_error(ErrorCode::UnexpectedEndOfString)),
                    }
                };
                self.des.scratch.clear();
                value
            }
            ScratchState::Number => {
                let value = visitor.visit_u64(parse(self.des.scratch.get_slice())?);
                self.des.scratch.clear();
                value
            }
            _ => todo!(), // todo: different types (ex i64)
        };
        *self.state = ScratchState::None;
        ret
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
    to_inner_des_method!(deserialize_seq);

    #[inline]
    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.des.deserialize_unit_struct(name, visitor)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.des.deserialize_newtype_struct(name, visitor)
    }

    #[inline]
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.des.deserialize_tuple(len, visitor)
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.des.deserialize_tuple_struct(name, len, visitor)
    }

    #[inline]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.des.deserialize_map(visitor)
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.des.deserialize_struct(name, fields, visitor)
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.des.deserialize_enum(name, variants, visitor)
    }
}
