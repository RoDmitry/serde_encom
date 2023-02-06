use crate::des::{deserializer::Deserializer, read::Read};
use crate::error::{Error, Result};
use serde::de;

pub(crate) struct VariantAccess<'a, R: 'a> {
    des: &'a mut Deserializer<R>,
}

impl<'a, R: 'a> VariantAccess<'a, R> {
    pub(crate) fn new(des: &'a mut Deserializer<R>) -> Self {
        VariantAccess { des }
    }
}

impl<'de, 'a, R: Read<'de> + 'a> de::EnumAccess<'de> for VariantAccess<'a, R> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.des)?;
        self.des.parse_object_colon()?;
        Ok((val, self))
    }
}

impl<'de, 'a, R: Read<'de> + 'a> de::VariantAccess<'de> for VariantAccess<'a, R> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        de::Deserialize::deserialize(self.des)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.des)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.des, visitor)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_struct(self.des, "", fields, visitor)
    }
}
