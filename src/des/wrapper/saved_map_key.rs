use crate::des::read::Read;
use crate::error::{Error, ErrorCode, Result};
use crate::Deserializer;
use serde::de;
use serde::forward_to_deserialize_any;

/// Only deserialize from this after peeking a '"' byte! Otherwise it may
/// deserialize invalid EnCom successfully.
pub(crate) struct SavedMapKeyDeserializer<'a, R: 'a> {
    pub(crate) des: &'a mut Deserializer<R>,
}

macro_rules! deserialize_numeric_key {
    ($method:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: de::Visitor<'de>,
        {
            self.deserialize_number(visitor)
        }
    };

    ($method:ident, $delegate:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: de::Visitor<'de>,
        {
            self.des.eat_char();

            match self.des.peek()? {
                Some(b'0'..=b'9' | b'-') => {}
                _ => return Err(self.des.error(ErrorCode::ExpectedNumericKey)),
            }

            let value = self.des.$delegate(visitor)?;

            match self.des.peek()? {
                Some(b'"') => self.des.eat_char(),
                _ => return Err(self.des.peek_error(ErrorCode::ExpectedDoubleQuote)),
            }

            Ok(value)
        }
    };
}

impl<'de, 'a, R> SavedMapKeyDeserializer<'a, R>
where
    R: Read<'de>,
{
    deserialize_numeric_key!(deserialize_number, deserialize_number);
}

impl<'de, 'a, R> de::Deserializer<'de> for SavedMapKeyDeserializer<'a, R>
where
    R: Read<'de>,
{
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // self.des.eat_char();
        // self.des.scratch.reset();
        // visitor.visit_borrowed_bytes(self.des.scratch.get_slice())
        let value = visitor.visit_borrowed_str(self.des.read.str_from_saved()?);
        self.des.read.clear_saved();
        value
    }

    deserialize_numeric_key!(deserialize_i8);
    deserialize_numeric_key!(deserialize_i16);
    deserialize_numeric_key!(deserialize_i32);
    deserialize_numeric_key!(deserialize_i64);
    deserialize_numeric_key!(deserialize_i128, deserialize_i128);
    deserialize_numeric_key!(deserialize_u8);
    deserialize_numeric_key!(deserialize_u16);
    deserialize_numeric_key!(deserialize_u32);
    deserialize_numeric_key!(deserialize_u64);
    deserialize_numeric_key!(deserialize_u128, deserialize_u128);
    #[cfg(not(feature = "float_roundtrip"))]
    deserialize_numeric_key!(deserialize_f32);
    #[cfg(feature = "float_roundtrip")]
    deserialize_numeric_key!(deserialize_f32, deserialize_f32);
    deserialize_numeric_key!(deserialize_f64);

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.des.eat_char();

        let peek = match self.des.next_char()? {
            Some(b) => b,
            None => {
                return Err(self.des.peek_error(ErrorCode::EofWhileParsingValue));
            }
        };

        let value = match peek {
            b't' => visitor.visit_bool(true),
            b'f' => visitor.visit_bool(false),
            _ => {
                let s = self.des.read.parse_str()?;
                Err(de::Error::invalid_type(de::Unexpected::Str(&s), &visitor))
            }
        };

        match value {
            Ok(value) => Ok(value),
            Err(err) => Err(self.des.fix_position(err)),
        }
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // Map keys cannot be null.
        visitor.visit_some(self)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        #[cfg(feature = "raw_value")]
        {
            if name == crate::raw::TOKEN {
                return self.des.deserialize_raw_value(visitor);
            }
        }

        let _ = name;
        visitor.visit_newtype_struct(self)
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

    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.des.deserialize_bytes(visitor)
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.des.deserialize_bytes(visitor)
    }

    forward_to_deserialize_any! {
        char str string unit unit_struct seq tuple tuple_struct map
        struct identifier ignored_any
    }
}
