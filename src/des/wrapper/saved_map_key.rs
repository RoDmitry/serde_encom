use crate::des::read::{Read, Reference};
use crate::error::{Error, Result};
use crate::Deserializer;
use serde::de;
use serde::forward_to_deserialize_any;

/// Only deserialize from this after peeking a '"' byte! Otherwise it may
/// deserialize invalid EnCom successfully.
pub(crate) struct SavedMapKeyDeserializer<'a, R: 'a> {
    pub(crate) des: &'a mut Deserializer<R>,
}

macro_rules! deserialize_integer_key {
    ($method:ident => $visit:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: de::Visitor<'de>,
        {
            self.des.eat_char();
            // self.des.scratch.clear();
            let string = self.des.read.parse_str()?;
            match (string.parse(), string) {
                (Ok(integer), _) => visitor.$visit(integer),
                (Err(_), Reference::Borrowed(s)) => visitor.visit_borrowed_str(s),
                (Err(_), Reference::Copied(s)) => visitor.visit_str(s),
            }
        }
    };
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
        let value = visitor.visit_str(self.des.read.str_from_saved()?);
        self.des.read.clear_saved();
        value
    }

    deserialize_integer_key!(deserialize_i8 => visit_i8);
    deserialize_integer_key!(deserialize_i16 => visit_i16);
    deserialize_integer_key!(deserialize_i32 => visit_i32);
    deserialize_integer_key!(deserialize_i64 => visit_i64);
    deserialize_integer_key!(deserialize_i128 => visit_i128);
    deserialize_integer_key!(deserialize_u8 => visit_u8);
    deserialize_integer_key!(deserialize_u16 => visit_u16);
    deserialize_integer_key!(deserialize_u32 => visit_u32);
    deserialize_integer_key!(deserialize_u64 => visit_u64);
    deserialize_integer_key!(deserialize_u128 => visit_u128);

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
        bool f32 f64 char str string unit unit_struct seq tuple tuple_struct map
        struct identifier ignored_any
    }
}
