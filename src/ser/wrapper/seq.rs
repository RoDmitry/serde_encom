use super::DataSerializer;
use crate::error::{Error, Result};
use crate::io;
use crate::ser::compound::Compound;
use crate::ser::formatter::Formatter;
use crate::ser::serializer::{Serializer, SerializerExtras};
use alloc::string::ToString;
use core::fmt::Display;
use serde::ser::{self, Serialize};

pub(crate) struct SeqSerializer<'a, W: 'a, F: 'a> {
    pub(crate) ser: &'a mut Serializer<W, F>,
}

impl<'a, W, F> ser::Serializer for SeqSerializer<'a, W, F>
where
    W: io::Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Compound<'a, W, F>;
    type SerializeTuple = Compound<'a, W, F>;
    type SerializeTupleStruct = Compound<'a, W, F>;
    type SerializeTupleVariant = Compound<'a, W, F>;
    type SerializeMap = Compound<'a, W, F>;
    type SerializeStruct = Compound<'a, W, F>;
    type SerializeStructVariant = Compound<'a, W, F>;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<()> {
        self.ser.serialize_bool(value)
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<()> {
        self.ser.serialize_i8(value)
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<()> {
        self.ser.serialize_i16(value)
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<()> {
        self.ser.serialize_i32(value)
    }

    #[inline]
    fn serialize_i64(self, value: i64) -> Result<()> {
        self.ser.serialize_i64(value)
    }

    #[inline]
    fn serialize_i128(self, value: i128) -> Result<()> {
        self.ser.serialize_i128(value)
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<()> {
        self.ser.serialize_u8(value)
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<()> {
        self.ser.serialize_u16(value)
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<()> {
        self.ser.serialize_u32(value)
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<()> {
        self.ser.serialize_u64(value)
    }

    #[inline]
    fn serialize_u128(self, value: u128) -> Result<()> {
        self.ser.serialize_u128(value)
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<()> {
        self.ser.serialize_f32(value)
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<()> {
        self.ser.serialize_f64(value)
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<()> {
        self.ser.serialize_char(value)
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<()> {
        self.ser.serialize_str(value)
    }

    #[inline]
    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        self.ser.serialize_bytes(value)
    }

    #[inline]
    fn serialize_unit(self) -> Result<()> {
        self.ser.serialize_unit()
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    /// Serialize newtypes without an object wrapper.
    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        /* tri!(self
        .ser
        .formatter
        .begin_object(&mut self.ser.writer)
        .map_err(Error::io)); */
        self.ser
            .formatter
            .begin_data_key(&mut self.ser.writer, true)
            .map_err(Error::io)?;
        self.ser.serialize_keystr(variant)?;
        self.ser
            .formatter
            .end_data_key(&mut self.ser.writer)
            .map_err(Error::io)?;
        /* self.ser
        .formatter
        .begin_object_value(&mut self.ser.writer)
        .map_err(Error::io)?; */
        value.serialize(DataSerializer { ser: self.ser })?;
        self.ser
            .formatter
            .end_data(&mut self.ser.writer)
            .map_err(Error::io)
        /* self.ser
        .formatter
        .end_object(&mut self.ser.writer)
        .map_err(Error::io) */
    }

    #[inline]
    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.ser.serialize_seq(len)
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.ser.serialize_tuple(len)
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.ser.serialize_tuple_struct(_name, len)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.ser
            .serialize_tuple_variant(name, variant_index, variant, len)
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        self.ser.serialize_map(len)
    }

    #[inline]
    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        match name {
            #[cfg(feature = "arbitrary_precision")]
            crate::number::TOKEN => Ok(Compound::Number { ser: self }),
            #[cfg(feature = "raw_value")]
            crate::raw::TOKEN => Ok(Compound::RawValue { ser: self }),
            _ => self.serialize_map(Some(len)),
        }
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.ser
            .serialize_struct_variant(_name, _variant_index, variant, len)
    }

    #[inline]
    fn collect_str<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Display,
    {
        self.serialize_str(&value.to_string())
    }
}
