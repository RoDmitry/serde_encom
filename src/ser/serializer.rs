use super::compound::{Compound, State};
use super::formatter::CompactFormatter;
use super::formatter::Formatter;
use super::formatter::PrettyFormatter;
use crate::error::{Error, Result};
use crate::io;
use alloc::string::ToString;
use core::fmt::Display;
use core::num::FpCategory;
use serde::ser::{self, Serialize};

/// A structure for serializing Rust values into EnCom.
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub(crate) struct Serializer<W, F = CompactFormatter> {
    pub(crate) writer: W,
    pub(crate) formatter: F,
}

impl<W> Serializer<W>
where
    W: io::Write,
{
    /// Creates a new EnCom serializer.
    #[inline]
    pub fn new(writer: W) -> Self {
        Serializer::with_formatter(writer, CompactFormatter)
    }
}

impl<'a, W> Serializer<W, PrettyFormatter<'a>>
where
    W: io::Write,
{
    /// Creates a new EnCom pretty print serializer.
    #[inline]
    pub fn pretty(writer: W) -> Self {
        Serializer::with_formatter(writer, PrettyFormatter::new())
    }
}

impl<W, F> Serializer<W, F>
where
    W: io::Write,
    F: Formatter,
{
    /// Creates a new EnCom visitor whose output will be written to the writer
    /// specified.
    #[inline]
    pub fn with_formatter(writer: W, formatter: F) -> Self {
        Serializer { writer, formatter }
    }

    /* /// Unwrap the `Writer` from the `Serializer`.
    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
    } */
}

pub(crate) trait SerializerExtras: Sized {
    fn serialize_keystr(self, value: &str) -> Result<()>;
}

impl<'a, W, F> SerializerExtras for &'a mut Serializer<W, F>
where
    W: io::Write,
    F: Formatter,
{
    #[inline]
    fn serialize_keystr(self, value: &str) -> Result<()> {
        self.formatter
            .write_key(&mut self.writer, value)
            .map_err(Error::io)
    }
}

impl<'a, W, F> ser::Serializer for &'a mut Serializer<W, F>
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
        self.formatter
            .write_bool(&mut self.writer, value)
            .map_err(Error::io)
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<()> {
        self.formatter
            .write_i8(&mut self.writer, value)
            .map_err(Error::io)
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<()> {
        self.formatter
            .write_i16(&mut self.writer, value)
            .map_err(Error::io)
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<()> {
        self.formatter
            .write_i32(&mut self.writer, value)
            .map_err(Error::io)
    }

    #[inline]
    fn serialize_i64(self, value: i64) -> Result<()> {
        self.formatter
            .write_i64(&mut self.writer, value)
            .map_err(Error::io)
    }

    #[inline]
    fn serialize_i128(self, value: i128) -> Result<()> {
        self.formatter
            .write_i128(&mut self.writer, value)
            .map_err(Error::io)
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<()> {
        self.formatter
            .write_u8(&mut self.writer, value)
            .map_err(Error::io)
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<()> {
        self.formatter
            .write_u16(&mut self.writer, value)
            .map_err(Error::io)
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<()> {
        self.formatter
            .write_u32(&mut self.writer, value)
            .map_err(Error::io)
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<()> {
        self.formatter
            .write_u64(&mut self.writer, value)
            .map_err(Error::io)
    }

    #[inline]
    fn serialize_u128(self, value: u128) -> Result<()> {
        self.formatter
            .write_u128(&mut self.writer, value)
            .map_err(Error::io)
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<()> {
        match value.classify() {
            FpCategory::Nan | FpCategory::Infinite => self
                .formatter
                .write_null(&mut self.writer)
                .map_err(Error::io),
            _ => self
                .formatter
                .write_f32(&mut self.writer, value)
                .map_err(Error::io),
        }
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<()> {
        match value.classify() {
            FpCategory::Nan | FpCategory::Infinite => self
                .formatter
                .write_null(&mut self.writer)
                .map_err(Error::io),
            _ => self
                .formatter
                .write_f64(&mut self.writer, value)
                .map_err(Error::io),
        }
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<()> {
        // A char encoded as UTF-8 takes 4 bytes at most.
        let mut buf = [0; 4];
        self.serialize_str(value.encode_utf8(&mut buf))
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<()> {
        self.formatter
            .write_u64(&mut self.writer, value.len() as u64)
            .map_err(Error::io)?;
        self.formatter
            .begin_string(&mut self.writer)
            .map_err(Error::io)?;
        self.formatter
            .write_bytes(&mut self.writer, value.as_bytes())
            .map_err(Error::io)
    }

    #[inline]
    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        /* use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(value.len()))?;
        for byte in value {
            seq.serialize_element(byte)?;
        }
        seq.end() */
        self.formatter
            .write_u64(&mut self.writer, value.len() as u64)
            .map_err(Error::io)?;
        self.formatter
            .begin_bytes(&mut self.writer)
            .map_err(Error::io)?;
        self.formatter
            .write_bytes(&mut self.writer, value)
            .map_err(Error::io)
    }

    #[inline]
    fn serialize_unit(self) -> Result<()> {
        self.formatter
            .write_null(&mut self.writer)
            .map_err(Error::io)
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
        self.formatter
            .begin_object(&mut self.writer)
            .map_err(Error::io)?;
        self.formatter
            .begin_data_key(&mut self.writer, true)
            .map_err(Error::io)?;
        self.serialize_keystr(variant)?;
        self.formatter
            .end_data_key(&mut self.writer)
            .map_err(Error::io)?;
        /* tri!(self
        .formatter
        .begin_object_value(&mut self.writer, true)
        .map_err(Error::io)); */
        value.serialize(&mut *self)?;
        self.formatter
            .end_data(&mut self.writer)
            .map_err(Error::io)?;
        self.formatter
            .end_object(&mut self.writer)
            .map_err(Error::io)
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

    /// here called for the Vec to serialize
    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        if len == Some(0) {
            Ok(Compound::Map {
                ser: self,
                state: State::Empty,
            })
        } else {
            self.formatter
                .begin_array(&mut self.writer)
                .map_err(Error::io)?;
            Ok(Compound::Map {
                ser: self,
                state: State::First,
            })
        }
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.formatter
            .begin_object(&mut self.writer)
            .map_err(Error::io)?;
        self.formatter
            .begin_data_key(&mut self.writer, true)
            .map_err(Error::io)?;
        self.serialize_str(variant)?;
        self.formatter
            .end_data_key(&mut self.writer)
            .map_err(Error::io)?;
        /* tri!(self
        .formatter
        .begin_object_value(&mut self.writer, false)
        .map_err(Error::io)); */
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        if len == Some(0) {
            /* tri!(self
            .formatter
            .end_object(&mut self.writer)
            .map_err(Error::io)); */
            Ok(Compound::Map {
                ser: self,
                state: State::Empty,
            })
        } else {
            self.formatter
                .begin_object(&mut self.writer)
                .map_err(Error::io)?;
            Ok(Compound::Map {
                ser: self,
                state: State::First,
            })
        }
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
        self.formatter
            .begin_object(&mut self.writer)
            .map_err(Error::io)?;
        self.formatter
            .begin_data_key(&mut self.writer, true)
            .map_err(Error::io)?;
        self.serialize_str(variant)?;
        self.formatter
            .end_data_key(&mut self.writer)
            .map_err(Error::io)?;
        /* tri!(self
        .formatter
        .begin_object_value(&mut self.writer, true)
        .map_err(Error::io)); */
        self.serialize_map(Some(len))
    }

    #[inline]
    fn collect_str<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Display,
    {
        self.serialize_str(&value.to_string())
    }
}
