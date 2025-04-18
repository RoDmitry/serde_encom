#![allow(clippy::zero_prefixed_literal)]

use crate::error::{Error, ErrorCode, Result};
// use alloc::vec::Vec;
// use core::char;
use core::cmp;
use core::ops::Deref;
use core::str;
use debug_unsafe::slice::SliceGetter;

#[cfg(feature = "std")]
use super::iter::LineColIterator;
#[cfg(feature = "std")]
use crate::io;
#[cfg(feature = "raw_value")]
use crate::raw::BorrowedRawDeserializer;
#[cfg(all(feature = "raw_value", feature = "std"))]
use crate::raw::OwnedRawDeserializer;
#[cfg(feature = "raw_value")]
use serde::de::Visitor;

/// Trait used by the deserializer for iterating over input. This is manually
/// "specialized" for iterating over &[u8]. Once feature(specialization) is
/// stable we can use actual specialization.
///
/// This trait is sealed and cannot be implemented for types outside of
/// `serde_encom`.
pub trait Read<'de>: private::Sealed {
    #[doc(hidden)]
    fn next(&mut self) -> Result<Option<u8>>;
    #[doc(hidden)]
    fn peek(&mut self) -> Result<Option<u8>>;

    /// Only valid after a call to peek(). Discards the peeked byte.
    #[doc(hidden)]
    fn discard(&mut self);

    /// Position of the most recent call to next().
    ///
    /// The most recent call was probably next() and not peek(), but this method
    /// should try to return a sensible result if the most recent call was
    /// actually peek() because we don't always know.
    ///
    /// Only called in case of an error, so performance is not important.
    #[doc(hidden)]
    fn position(&self) -> Position;

    /// Position of the most recent call to peek().
    ///
    /// The most recent call was probably peek() and not next(), but this method
    /// should try to return a sensible result if the most recent call was
    /// actually next() because we don't always know.
    ///
    /// Only called in case of an error, so performance is not important.
    #[doc(hidden)]
    fn peek_position(&self) -> Position;

    /// Offset from the beginning of the input to the next byte that would be
    /// returned by next() or peek().
    #[doc(hidden)]
    fn byte_offset(&self) -> usize;

    fn read_str<'s>(&'s mut self, len: usize) -> Result<&'de str>;

    fn read_slice<'s>(&'s mut self, len: usize) -> Result<&'de [u8]>;

    fn parse_int_any_pos(&mut self) -> Result<u64>;

    // fn parse_int(&mut self) -> Result<ParserNumber>;

    // fn parse_int_any(&mut self) -> Result<ParserNumber>;

    fn str_from_saved(&mut self) -> Result<&'de str>;

    /// Assumes the previous byte was a quotation mark. Parses an EnCom-escaped
    /// string until the next quotation mark using the given scratch space if
    /// necessary. The scratch space is initially empty.
    #[doc(hidden)]
    fn parse_str<'s>(&'s mut self) -> Result<Reference<'de, 's, str>>;

    // Assumes the previous byte was a quotation mark. Parses an EnCom-escaped
    // string until the next quotation mark using the given scratch space if
    // necessary. The scratch space is initially empty.
    //
    // This function returns the raw bytes in the string with escape sequences
    // expanded but without performing unicode validation.
    //#[doc(hidden)]
    //fn parse_str_raw<'s>(&'s mut self) -> Result<Reference<'de, 's, [u8]>>;

    /// Assumes the previous byte was a quotation mark. Parses an EnCom-escaped
    /// string until the next quotation mark but discards the data.
    #[doc(hidden)]
    fn ignore_str(&mut self) -> Result<()>;

    /// Assumes the previous byte was a hex escape sequnce ('\u') in a string.
    /// Parses next hexadecimal sequence.
    #[doc(hidden)]
    fn decode_hex_escape(&mut self) -> Result<u16>;

    /// Switch raw buffering mode on.
    ///
    /// This is used when deserializing `RawValue`.
    #[cfg(feature = "raw_value")]
    #[doc(hidden)]
    fn begin_raw_buffering(&mut self);

    /// Switch raw buffering mode off and provides the raw buffered data to the
    /// given visitor.
    #[cfg(feature = "raw_value")]
    #[doc(hidden)]
    fn end_raw_buffering<V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>;

    /// Whether StreamDeserializer::next needs to check the failed flag. True
    /// for IoRead, false for StrRead and SliceRead which can track failure by
    /// truncating their input slice to avoid the extra check on every next
    /// call.
    #[doc(hidden)]
    const SHOULD_EARLY_RETURN_IF_FAILED: bool;

    /// Mark a persistent failure of StreamDeserializer, either by setting the
    /// flag or by truncating the input data.
    #[doc(hidden)]
    fn set_failed(&mut self, failed: &mut bool);

    fn save_start(&mut self);
    fn save_end(&mut self);
    fn clear_saved(&mut self);
    fn get_saved(&mut self) -> &'de [u8];
    fn saved_is_empty(&self) -> bool;
}

pub struct Position {
    pub line: usize,
    pub column: usize,
}

pub enum Reference<'b, 'c, T>
where
    T: ?Sized + 'static,
{
    Borrowed(&'b T),
    Copied(&'c T),
}

impl<'b, 'c, T> Deref for Reference<'b, 'c, T>
where
    T: ?Sized + 'static,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match *self {
            Reference::Borrowed(b) => b,
            Reference::Copied(c) => c,
        }
    }
}

/// EnCom input source that reads from a std::io input stream.
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub struct IoRead<R>
where
    R: io::Read,
{
    iter: LineColIterator<io::Bytes<R>>,
    /// Temporary storage of peeked byte.
    ch: Option<u8>,
    #[cfg(feature = "raw_value")]
    raw_buffer: Option<Vec<u8>>,
}

/// EnCom input source that reads from a slice of bytes.
//
// This is more efficient than other iterators because peek() can be read-only
// and we can compute line/col position only if an error happens.
pub struct SliceRead<'a> {
    slice: &'a [u8],
    /// Index of the *next* byte that will be returned by next() or peek().
    index: usize,
    #[cfg(feature = "raw_value")]
    raw_buffering_start_index: usize,
    save_start: usize,
    save_end: usize,
}

/// EnCom input source that reads from a UTF-8 string.
//
// Able to elide UTF-8 checks by assuming that the input is valid UTF-8.
pub struct StrRead<'a> {
    delegate: SliceRead<'a>,
    #[cfg(feature = "raw_value")]
    data: &'a str,
}

// Prevent users from implementing the Read trait.
mod private {
    pub trait Sealed {}
}

//////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "std")]
impl<R> IoRead<R>
where
    R: io::Read,
{
    /// Create an EnCom input source to read from a std::io input stream.
    pub fn new(reader: R) -> Self {
        IoRead {
            iter: LineColIterator::new(reader.bytes()),
            ch: None,
            #[cfg(feature = "raw_value")]
            raw_buffer: None,
        }
    }
}

#[cfg(feature = "std")]
impl<R> private::Sealed for IoRead<R> where R: io::Read {}

#[cfg(feature = "std")]
impl<R> IoRead<R>
where
    R: io::Read,
{
    fn parse_str_bytes<'s, T, F>(&'s mut self, validate: bool, result: F) -> Result<T>
    where
        T: 's,
        F: FnOnce(&'s Self, &'s [u8]) -> Result<T>,
    {
        unimplemented!()
        /* loop {
            let ch = next_or_eof(self)?;
            if !ESCAPE[ch as usize] {
                scratch.push(ch);
                continue;
            }
            match ch {
                b' ' | b'\n' | b'\t' | b'\r' => {
                    return result(self, scratch);
                }
                /* b'\\' => {
                    parse_escape(self, validate, scratch)?;
                } */
                _ => {
                    if validate {
                        return error(self, ErrorCode::ControlCharacterWhileParsingString);
                    }
                    scratch.push(ch);
                }
            }
        } */
    }
}

#[cfg(feature = "std")]
impl<'de, R> Read<'de> for IoRead<R>
where
    R: io::Read,
{
    #[inline]
    fn next(&mut self) -> Result<Option<u8>> {
        match self.ch.take() {
            Some(ch) => {
                #[cfg(feature = "raw_value")]
                {
                    if let Some(buf) = &mut self.raw_buffer {
                        buf.push(ch);
                    }
                }
                Ok(Some(ch))
            }
            None => match self.iter.next() {
                Some(Err(err)) => Err(Error::io(err)),
                Some(Ok(ch)) => {
                    #[cfg(feature = "raw_value")]
                    {
                        if let Some(buf) = &mut self.raw_buffer {
                            buf.push(ch);
                        }
                    }
                    Ok(Some(ch))
                }
                None => Ok(None),
            },
        }
    }

    #[inline]
    fn peek(&mut self) -> Result<Option<u8>> {
        match self.ch {
            Some(ch) => Ok(Some(ch)),
            None => match self.iter.next() {
                Some(Err(err)) => Err(Error::io(err)),
                Some(Ok(ch)) => {
                    self.ch = Some(ch);
                    Ok(self.ch)
                }
                None => Ok(None),
            },
        }
    }

    #[cfg(not(feature = "raw_value"))]
    #[inline]
    fn discard(&mut self) {
        self.ch = None;
    }

    #[cfg(feature = "raw_value")]
    fn discard(&mut self) {
        if let Some(ch) = self.ch.take() {
            if let Some(buf) = &mut self.raw_buffer {
                buf.push(ch);
            }
        }
    }

    fn position(&self) -> Position {
        Position {
            line: self.iter.line(),
            column: self.iter.col(),
        }
    }

    #[inline]
    fn peek_position(&self) -> Position {
        // The LineColIterator updates its position during peek() so it has the
        // right one here.
        self.position()
    }

    #[inline]
    fn byte_offset(&self) -> usize {
        match self.ch {
            Some(_) => self.iter.byte_offset() - 1,
            None => self.iter.byte_offset(),
        }
    }

    #[inline]
    fn read_str<'s>(&'s mut self, _len: usize) -> Result<&'de str> {
        unimplemented!()
    }

    #[inline]
    fn read_slice<'s>(&'s mut self, _len: usize) -> Result<&'de [u8]> {
        unimplemented!()
    }

    #[inline]
    fn parse_int_any_pos(&mut self) -> Result<u64> {
        unimplemented!()
    }

    /* #[inline]
    fn parse_int(&mut self) -> Result<ParserNumber> {
        unimplemented!()
    }

    #[inline]
    fn parse_int_any(&mut self) -> Result<ParserNumber> {
        unimplemented!()
    } */

    #[inline]
    fn str_from_saved(&mut self) -> Result<&'de str> {
        let saved = self.get_saved();
        as_str(self, saved)
    }

    #[inline]
    fn parse_str<'s>(&'s mut self) -> Result<Reference<'de, 's, str>> {
        self.parse_str_bytes(true, as_str).map(Reference::Copied)
    }

    /* #[inline]
    fn parse_str_raw<'s>(&'s mut self) -> Result<Reference<'de, 's, [u8]>> {
        self.parse_str_bytes(false, |_, bytes| Ok(bytes))
            .map(Reference::Copied)
    } */

    fn ignore_str(&mut self) -> Result<()> {
        loop {
            let ch = next_or_eof(self)?;
            if !ESCAPE[ch as usize] {
                continue;
            }
            match ch {
                b' ' | b'\n' | b'\t' | b'\r' => {
                    return Ok(());
                }
                /* b'\\' => {
                    ignore_escape(self)?;
                } */
                _ => {
                    return error(self, ErrorCode::ControlCharacterWhileParsingString);
                }
            }
        }
    }

    fn decode_hex_escape(&mut self) -> Result<u16> {
        let mut n = 0;
        for _ in 0..4 {
            match decode_hex_val(next_or_eof(self)?) {
                None => return error(self, ErrorCode::InvalidEscape),
                Some(val) => {
                    n = (n << 4) + val;
                }
            }
        }
        Ok(n)
    }

    #[cfg(feature = "raw_value")]
    fn begin_raw_buffering(&mut self) {
        self.raw_buffer = Some(Vec::new());
    }

    #[cfg(feature = "raw_value")]
    fn end_raw_buffering<V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let raw = self.raw_buffer.take().unwrap();
        let raw = match String::from_utf8(raw) {
            Ok(raw) => raw,
            Err(_) => return error(self, ErrorCode::InvalidUnicodeCodePoint),
        };
        visitor.visit_map(OwnedRawDeserializer {
            raw_value: Some(raw),
        })
    }

    const SHOULD_EARLY_RETURN_IF_FAILED: bool = true;

    #[inline]
    #[cold]
    fn set_failed(&mut self, failed: &mut bool) {
        *failed = true;
    }

    #[inline]
    fn save_start(&mut self) {
        unimplemented!()
    }
    #[inline]
    fn save_end(&mut self) {
        unimplemented!()
    }
    #[inline]
    fn clear_saved(&mut self) {
        unimplemented!()
    }
    #[inline]
    fn get_saved(&mut self) -> &'de [u8] {
        unimplemented!()
    }
    #[inline]
    fn saved_is_empty(&self) -> bool {
        unimplemented!()
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<'de> SliceRead<'de> {
    /// Create an EnCom input source to read from a slice of bytes.
    pub fn new(slice: &'de [u8]) -> Self {
        SliceRead {
            slice,
            index: 0,
            #[cfg(feature = "raw_value")]
            raw_buffering_start_index: 0,
            save_start: 0,
            save_end: 0,
        }
    }

    fn position_of_index(&self, i: usize) -> Position {
        let mut position = Position { line: 1, column: 0 };
        for ch in &self.slice[..i] {
            match *ch {
                b'\n' => {
                    position.line += 1;
                    position.column = 0;
                }
                _ => {
                    position.column += 1;
                }
            }
        }
        position
    }

    /// The big optimization here over IoRead is that if the string contains no
    /// backslash escape sequences, the returned &str is a slice of the raw EnCom
    /// data so we avoid copying into the scratch space.
    fn parse_str_bytes<'s, T, F>(
        &'s mut self,
        _validate: bool,
        result: F,
    ) -> Result<Reference<'de, 's, T>>
    where
        T: ?Sized + 's,
        F: FnOnce(&'s Self, &'de [u8]) -> Result<&'de T>,
    {
        // Index of the first byte not yet copied into the scratch space.
        let start = self.index;

        loop {
            /* while self.index < self.slice.len() && !ESCAPE[self.slice[self.index] as usize] {
                self.index += 1;
            } */
            if self.index == self.slice.len() {
                return error(self, ErrorCode::EofWhileParsingString);
            }
            match self.slice[self.index] {
                b':' | b'{' => {
                    // if scratch.is_empty() {
                    // Fast path: return a slice of the raw EnCom without any
                    // copying.
                    let borrowed = &self.slice[start..self.index];
                    // self.index += 1;
                    return result(self, borrowed).map(Reference::Borrowed);
                    /* } else {
                        scratch.extend_from_slice(&self.slice[start..self.index]);
                        self.index += 1;
                        return result(self, scratch).map(Reference::Copied);
                    } */
                }
                /* b'\\' => {
                    scratch.extend_from_slice(&self.slice[start..self.index]);
                    self.index += 1;
                    parse_escape(self, validate, scratch)?;
                    start = self.index;
                } */
                _ => {
                    self.index += 1;
                    /* if validate {
                        return error(self, ErrorCode::ControlCharacterWhileParsingString);
                    } */
                }
            }
        }
    }
}

impl<'de> private::Sealed for SliceRead<'de> {}

impl<'de> Read<'de> for SliceRead<'de> {
    #[inline]
    fn next(&mut self) -> Result<Option<u8>> {
        // `Ok(self.slice.get(self.index).map(|ch| { self.index += 1; *ch }))`
        // is about 10% slower.
        Ok(if self.index < self.slice.len() {
            let ch = self.slice[self.index];
            self.index += 1;
            Some(ch)
        } else {
            None
        })
    }

    #[inline]
    fn peek(&mut self) -> Result<Option<u8>> {
        // `Ok(self.slice.get(self.index).map(|ch| *ch))` is about 10% slower
        // for some reason.
        Ok(if self.index < self.slice.len() {
            Some(self.slice[self.index])
        } else {
            None
        })
    }

    #[inline]
    fn discard(&mut self) {
        self.index += 1;
    }

    #[inline]
    fn position(&self) -> Position {
        self.position_of_index(self.index)
    }

    #[inline]
    fn peek_position(&self) -> Position {
        // Cap it at slice.len() just in case the most recent call was next()
        // and it returned the last byte.
        self.position_of_index(cmp::min(self.slice.len(), self.index + 1))
    }

    #[inline]
    fn byte_offset(&self) -> usize {
        self.index
    }

    #[inline]
    fn read_str<'s>(&'s mut self, len: usize) -> Result<&'de str> {
        let start = self.index;
        self.index += len;
        as_str(self, &self.slice[start..self.index])
    }

    #[inline]
    fn read_slice<'s>(&'s mut self, len: usize) -> Result<&'de [u8]> {
        let start = self.index;
        self.index += len;
        Ok(&self.slice[start..self.index])
    }

    #[inline]
    fn parse_int_any_pos(&mut self) -> Result<u64> {
        let (res, i) = atoi_simd::parse_any_pos(self.slice.get_safe_unchecked(self.index..))?;
        self.index += i;
        Ok(res)
    }

    /* #[inline]
    fn parse_int(&mut self) -> Result<ParserNumber> {
        let res = if *self.slice.first().ok_or(AtoiSimdError::Empty)? == b'-' {
            self.index += 1;
            ParserNumber::I64(atoi_simd::parse_neg(
                &self.slice.get_safe_unchecked(self.index..),
            )?)
        } else {
            ParserNumber::U64(atoi_simd::parse_pos(&self.slice[self.index..])?)
        };
        self.index = self.slice.len();
        Ok(res)
    }

    #[inline]
    fn parse_int_any(&mut self) -> Result<ParserNumber> {
        let (res, i) = if *self.slice.first().ok_or(AtoiSimdError::Empty)? == b'-' {
            self.index += 1;
            let (v, l) =
                atoi_simd::parse_any_neg(&self.slice.get_safe_unchecked(self.index..))?;
            (ParserNumber::I64(v), l)
        } else {
            let (v, l) = atoi_simd::parse_any_pos(&self.slice[self.index..])?;
            (ParserNumber::U64(v), l)
        };
        self.index += i;
        Ok(res)
    } */

    #[inline]
    fn str_from_saved(&mut self) -> Result<&'de str> {
        let saved = self.get_saved();
        as_str(self, saved)
    }

    #[inline]
    fn parse_str<'s>(&'s mut self) -> Result<Reference<'de, 's, str>> {
        self.parse_str_bytes(true, as_str)
    }

    /* #[inline]
    fn parse_str_raw<'s>(&'s mut self) -> Result<Reference<'de, 's, [u8]>> {
        self.parse_str_bytes(false, |_, bytes| Ok(bytes))
    } */

    fn ignore_str(&mut self) -> Result<()> {
        // loop {
        while self.index < self.slice.len() && !ESCAPE[self.slice[self.index] as usize] {
            self.index += 1;
        }
        if self.index == self.slice.len() {
            return error(self, ErrorCode::EofWhileParsingString);
        }
        match self.slice[self.index] {
            b' ' | b'\n' | b'\t' | b'\r' => {
                self.index += 1;
                Ok(())
            }
            /* b'\\' => {
                self.index += 1;
                ignore_escape(self)?;
            } */
            _ => error(self, ErrorCode::ControlCharacterWhileParsingString),
        }
        // }
    }

    fn decode_hex_escape(&mut self) -> Result<u16> {
        if self.index + 4 > self.slice.len() {
            self.index = self.slice.len();
            return error(self, ErrorCode::EofWhileParsingString);
        }

        let mut n = 0;
        for _ in 0..4 {
            let ch = decode_hex_val(self.slice[self.index]);
            self.index += 1;
            match ch {
                None => return error(self, ErrorCode::InvalidEscape),
                Some(val) => {
                    n = (n << 4) + val;
                }
            }
        }
        Ok(n)
    }

    #[cfg(feature = "raw_value")]
    fn begin_raw_buffering(&mut self) {
        self.raw_buffering_start_index = self.index;
    }

    #[cfg(feature = "raw_value")]
    fn end_raw_buffering<V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let raw = &self.slice[self.raw_buffering_start_index..self.index];
        let raw = match str::from_utf8(raw) {
            Ok(raw) => raw,
            Err(_) => return error(self, ErrorCode::InvalidUnicodeCodePoint),
        };
        visitor.visit_map(BorrowedRawDeserializer {
            raw_value: Some(raw),
        })
    }

    const SHOULD_EARLY_RETURN_IF_FAILED: bool = false;

    #[inline]
    #[cold]
    fn set_failed(&mut self, _failed: &mut bool) {
        self.slice = &self.slice[..self.index];
    }

    #[inline]
    fn save_start(&mut self) {
        self.save_start = self.index;
    }
    #[inline]
    fn save_end(&mut self) {
        self.save_end = self.index;
    }
    #[inline]
    fn clear_saved(&mut self) {
        self.save_end = self.save_start;
    }
    #[inline]
    fn get_saved(&mut self) -> &'de [u8] {
        &self.slice[self.save_start..self.save_end]
    }
    #[inline]
    fn saved_is_empty(&self) -> bool {
        self.save_end == self.save_start
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<'a> StrRead<'a> {
    /// Create an EnCom input source to read from a UTF-8 string.
    pub fn new(s: &'a str) -> Self {
        StrRead {
            delegate: SliceRead::new(s.as_bytes()),
            #[cfg(feature = "raw_value")]
            data: s,
        }
    }
}

impl<'de> private::Sealed for StrRead<'de> {}

impl<'de> Read<'de> for StrRead<'de> {
    #[inline]
    fn next(&mut self) -> Result<Option<u8>> {
        self.delegate.next()
    }

    #[inline]
    fn peek(&mut self) -> Result<Option<u8>> {
        self.delegate.peek()
    }

    #[inline]
    fn discard(&mut self) {
        self.delegate.discard();
    }

    #[inline]
    fn position(&self) -> Position {
        self.delegate.position()
    }

    #[inline]
    fn peek_position(&self) -> Position {
        self.delegate.peek_position()
    }

    #[inline]
    fn byte_offset(&self) -> usize {
        self.delegate.byte_offset()
    }

    #[inline]
    fn read_str<'s>(&'s mut self, len: usize) -> Result<&'de str> {
        self.delegate.read_str(len)
    }

    #[inline]
    fn read_slice<'s>(&'s mut self, len: usize) -> Result<&'de [u8]> {
        self.delegate.read_slice(len)
    }

    #[inline]
    fn parse_int_any_pos(&mut self) -> Result<u64> {
        self.delegate.parse_int_any_pos()
    }

    /* #[inline]
    fn parse_int(&mut self) -> Result<ParserNumber> {
        self.delegate.parse_int()
    }

    #[inline]
    fn parse_int_any(&mut self) -> Result<ParserNumber> {
        self.delegate.parse_int_any()
    } */

    #[inline]
    fn str_from_saved(&mut self) -> Result<&'de str> {
        unsafe { Ok(str::from_utf8_unchecked(self.get_saved())) }
    }

    #[inline]
    fn parse_str<'s>(&'s mut self) -> Result<Reference<'de, 's, str>> {
        self.delegate.parse_str_bytes(true, |_, bytes| {
            // The deserialization input came in as &str with a UTF-8 guarantee,
            // and the \u-escapes are checked along the way, so don't need to
            // check here.
            Ok(unsafe { str::from_utf8_unchecked(bytes) })
        })
    }

    /* fn parse_str_raw<'s>(&'s mut self) -> Result<Reference<'de, 's, [u8]>> {
        self.delegate.parse_str_raw()
    } */

    #[inline]
    fn ignore_str(&mut self) -> Result<()> {
        self.delegate.ignore_str()
    }

    #[inline]
    fn decode_hex_escape(&mut self) -> Result<u16> {
        self.delegate.decode_hex_escape()
    }

    #[cfg(feature = "raw_value")]
    #[inline]
    fn begin_raw_buffering(&mut self) {
        self.delegate.begin_raw_buffering();
    }

    #[cfg(feature = "raw_value")]
    fn end_raw_buffering<V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let raw = &self.data[self.delegate.raw_buffering_start_index..self.delegate.index];
        visitor.visit_map(BorrowedRawDeserializer {
            raw_value: Some(raw),
        })
    }

    const SHOULD_EARLY_RETURN_IF_FAILED: bool = false;

    #[inline]
    #[cold]
    fn set_failed(&mut self, failed: &mut bool) {
        self.delegate.set_failed(failed);
    }

    #[inline]
    fn save_start(&mut self) {
        self.delegate.save_start()
    }
    #[inline]
    fn save_end(&mut self) {
        self.delegate.save_end()
    }
    #[inline]
    fn clear_saved(&mut self) {
        self.delegate.clear_saved()
    }
    #[inline]
    fn get_saved(&mut self) -> &'de [u8] {
        self.delegate.get_saved()
    }
    #[inline]
    fn saved_is_empty(&self) -> bool {
        self.delegate.saved_is_empty()
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<'de, R> private::Sealed for &mut R where R: Read<'de> {}

impl<'de, R> Read<'de> for &mut R
where
    R: Read<'de>,
{
    #[inline]
    fn next(&mut self) -> Result<Option<u8>> {
        R::next(self)
    }

    #[inline]
    fn peek(&mut self) -> Result<Option<u8>> {
        R::peek(self)
    }

    #[inline]
    fn discard(&mut self) {
        R::discard(self);
    }

    #[inline]
    fn position(&self) -> Position {
        R::position(self)
    }

    #[inline]
    fn peek_position(&self) -> Position {
        R::peek_position(self)
    }

    #[inline]
    fn byte_offset(&self) -> usize {
        R::byte_offset(self)
    }

    #[inline]
    fn read_str<'s>(&'s mut self, len: usize) -> Result<&'de str> {
        R::read_str(self, len)
    }

    #[inline]
    fn read_slice<'s>(&'s mut self, len: usize) -> Result<&'de [u8]> {
        R::read_slice(self, len)
    }

    #[inline]
    fn parse_int_any_pos(&mut self) -> Result<u64> {
        R::parse_int_any_pos(self)
    }

    /* #[inline]
    fn parse_int(&mut self) -> Result<ParserNumber> {
        R::parse_int(self)
    }

    #[inline]
    fn parse_int_any(&mut self) -> Result<ParserNumber> {
        R::parse_int_any(self)
    } */

    #[inline]
    fn str_from_saved(&mut self) -> Result<&'de str> {
        R::str_from_saved(self)
    }

    #[inline]
    fn parse_str<'s>(&'s mut self) -> Result<Reference<'de, 's, str>> {
        R::parse_str(self)
    }

    /* #[inline]
    fn parse_str_raw<'s>(&'s mut self) -> Result<Reference<'de, 's, [u8]>> {
        R::parse_str_raw(self)
    } */

    #[inline]
    fn ignore_str(&mut self) -> Result<()> {
        R::ignore_str(self)
    }

    #[inline]
    fn decode_hex_escape(&mut self) -> Result<u16> {
        R::decode_hex_escape(self)
    }

    #[cfg(feature = "raw_value")]
    #[inline]
    fn begin_raw_buffering(&mut self) {
        R::begin_raw_buffering(self);
    }

    #[cfg(feature = "raw_value")]
    #[inline]
    fn end_raw_buffering<V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        R::end_raw_buffering(self, visitor)
    }

    const SHOULD_EARLY_RETURN_IF_FAILED: bool = R::SHOULD_EARLY_RETURN_IF_FAILED;

    #[inline]
    fn set_failed(&mut self, failed: &mut bool) {
        R::set_failed(self, failed);
    }

    #[inline]
    fn save_start(&mut self) {
        R::save_start(self)
    }
    #[inline]
    fn save_end(&mut self) {
        R::save_end(self)
    }
    #[inline]
    fn clear_saved(&mut self) {
        R::clear_saved(self)
    }
    #[inline]
    fn get_saved(&mut self) -> &'de [u8] {
        R::get_saved(self)
    }
    #[inline]
    fn saved_is_empty(&self) -> bool {
        R::saved_is_empty(self)
    }
}

//////////////////////////////////////////////////////////////////////////////

/// Marker for whether StreamDeserializer can implement FusedIterator.
pub trait Fused: private::Sealed {}
impl<'a> Fused for SliceRead<'a> {}
impl<'a> Fused for StrRead<'a> {}

// Lookup table of bytes that must be escaped. A value of true at index i means
// that byte i requires an escape sequence in the input.
static ESCAPE: [bool; 256] = {
    const CT: bool = true; // control character \x00..=\x1F
    const QU: bool = true; // quote \x22
    const BS: bool = true; // backslash \x5C
    const __: bool = false; // allow unescaped
    [
        //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 0
        CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 1
        __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
        __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ]
};

#[inline]
fn next_or_eof<'de, R>(read: &mut R) -> Result<u8>
where
    R: ?Sized + Read<'de>,
{
    match read.next()? {
        Some(b) => Ok(b),
        None => error(read, ErrorCode::EofWhileParsingString),
    }
}

/* #[inline]
fn peek_or_eof<'de, R>(read: &mut R) -> Result<u8>
where
    R: ?Sized + Read<'de>,
{
    match read.peek()? {
        Some(b) => Ok(b),
        None => error(read, ErrorCode::EofWhileParsingString),
    }
} */

fn error<'de, R, T>(read: &R, reason: ErrorCode) -> Result<T>
where
    R: ?Sized + Read<'de>,
{
    let position = read.position();
    Err(Error::syntax(reason, position.line, position.column))
}

#[inline]
fn as_str<'de, 's, R: Read<'de>>(read: &R, slice: &'s [u8]) -> Result<&'s str> {
    str::from_utf8(slice).or_else(|_| error(read, ErrorCode::InvalidUnicodeCodePoint))
}

/// Parses an EnCom escape sequence and appends it into the scratch space. Assumes
/// the previous byte read was a backslash.
/* fn parse_escape<'de, R: Read<'de>>(
    read: &mut R,
    validate: bool,
    scratch: &mut Vec<u8>,
) -> Result<()> {
    let ch = next_or_eof(read)?;

    match ch {
        b'"' => scratch.push(b'"'),
        b'\\' => scratch.push(b'\\'),
        b'/' => scratch.push(b'/'),
        b'b' => scratch.push(b'\x08'),
        b'f' => scratch.push(b'\x0c'),
        b'n' => scratch.push(b'\n'),
        b'r' => scratch.push(b'\r'),
        b't' => scratch.push(b'\t'),
        b'u' => {
            fn encode_surrogate(scratch: &mut Vec<u8>, n: u16) {
                scratch.extend_from_slice(&[
                    (n >> 12 & 0b0000_1111) as u8 | 0b1110_0000,
                    (n >> 6 & 0b0011_1111) as u8 | 0b1000_0000,
                    (n & 0b0011_1111) as u8 | 0b1000_0000,
                ]);
            }

            let c = match read.decode_hex_escape()? {
                n @ 0xDC00..=0xDFFF => {
                    return if validate {
                        error(read, ErrorCode::LoneLeadingSurrogateInHexEscape)
                    } else {
                        encode_surrogate(scratch, n);
                        Ok(())
                    };
                }

                // Non-BMP characters are encoded as a sequence of two hex
                // escapes, representing UTF-16 surrogates. If deserializing a
                // utf-8 string the surrogates are required to be paired,
                // whereas deserializing a byte string accepts lone surrogates.
                n1 @ 0xD800..=0xDBFF => {
                    if peek_or_eof(read)? == b'\\' {
                        read.discard();
                    } else {
                        return if validate {
                            read.discard();
                            error(read, ErrorCode::UnexpectedEndOfHexEscape)
                        } else {
                            encode_surrogate(scratch, n1);
                            Ok(())
                        };
                    }

                    if peek_or_eof(read)? == b'u' {
                        read.discard();
                    } else {
                        return if validate {
                            read.discard();
                            error(read, ErrorCode::UnexpectedEndOfHexEscape)
                        } else {
                            encode_surrogate(scratch, n1);
                            // The \ prior to this byte started an escape sequence,
                            // so we need to parse that now. This recursive call
                            // does not blow the stack on malicious input because
                            // the escape is not \u, so it will be handled by one
                            // of the easy nonrecursive cases.
                            parse_escape(read, validate, scratch)
                        };
                    }

                    let n2 = read.decode_hex_escape()?;

                    if n2 < 0xDC00 || n2 > 0xDFFF {
                        return error(read, ErrorCode::LoneLeadingSurrogateInHexEscape);
                    }

                    let n = (((n1 - 0xD800) as u32) << 10 | (n2 - 0xDC00) as u32) + 0x1_0000;

                    match char::from_u32(n) {
                        Some(c) => c,
                        None => {
                            return error(read, ErrorCode::InvalidUnicodeCodePoint);
                        }
                    }
                }

                // Every u16 outside of the surrogate ranges above is guaranteed
                // to be a legal char.
                n => char::from_u32(n as u32).unwrap(),
            };

            scratch.extend_from_slice(c.encode_utf8(&mut [0_u8; 4]).as_bytes());
        }
        _ => {
            return error(read, ErrorCode::InvalidEscape);
        }
    }

    Ok(())
} */

/// Parses an EnCom escape sequence and discards the value. Assumes the previous
/// byte read was a backslash.
/* fn ignore_escape<'de, R>(read: &mut R) -> Result<()>
where
    R: ?Sized + Read<'de>,
{
    let ch = next_or_eof(read)?;

    match ch {
        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {}
        b'u' => {
            // At this point we don't care if the codepoint is valid. We just
            // want to consume it. We don't actually know what is valid or not
            // at this point, because that depends on if this string will
            // ultimately be parsed into a string or a byte buffer in the "real"
            // parse.

            read.decode_hex_escape()?;
        }
        _ => {
            return error(read, ErrorCode::InvalidEscape);
        }
    }

    Ok(())
} */

static HEX: [u8; 256] = {
    const __: u8 = 255; // not a hex digit
    [
        //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 0
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
        00, 01, 02, 03, 04, 05, 06, 07, 08, 09, __, __, __, __, __, __, // 3
        __, 10, 11, 12, 13, 14, 15, __, __, __, __, __, __, __, __, __, // 4
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 5
        __, 10, 11, 12, 13, 14, 15, __, __, __, __, __, __, __, __, __, // 6
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ]
};

fn decode_hex_val(val: u8) -> Option<u16> {
    let n = HEX[val as usize] as u16;
    if n == 255 {
        None
    } else {
        Some(n)
    }
}
