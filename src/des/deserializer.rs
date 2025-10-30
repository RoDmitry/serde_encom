use super::access::{MapAccess, SavedMapAccess, SavedSeqAccess, SeqAccess, VariantAccess};
#[cfg(feature = "std")]
use super::read::IoRead;
use super::read::{Read, Reference, SliceRead, StrRead};
use super::{parser_number::ParserNumber, stream_deserializer::StreamDeserializer};
use crate::error::{Error, ErrorCode, Result};
#[cfg(feature = "float_roundtrip")]
use crate::lexical;
use alloc::string::String;
#[cfg(feature = "float_roundtrip")]
use core::iter;
use core::marker::PhantomData;
use serde::de::{self, Expected, Unexpected};

#[cfg(feature = "arbitrary_precision")]
use crate::number::NumberDeserializer;

macro_rules! overflow {
    ($a:ident * 10 + $b:ident, $c:expr) => {
        match $c {
            c => $a >= c / 10 && ($a > c / 10 || $b > c % 10),
        }
    };
}

macro_rules! deserialize_number {
    ($method:ident) => {
        deserialize_number!($method, deserialize_number);
    };

    ($method:ident, $using:ident) => {
        #[inline]
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: de::Visitor<'de>,
        {
            self.$using(visitor)
        }
    };
}

#[cfg(not(feature = "unbounded_depth"))]
macro_rules! if_checking_recursion_limit {
    ($($body:tt)*) => {
        $($body)*
    };
}

#[cfg(feature = "unbounded_depth")]
macro_rules! if_checking_recursion_limit {
    ($this:ident $($body:tt)*) => {
        if !$this.disable_recursion_limit {
            $this $($body)*
        }
    };
}

macro_rules! check_recursion {
    ($this:ident $($body:tt)*) => {
        if_checking_recursion_limit! {
            $this.remaining_depth -= 1;
            if $this.remaining_depth == 0 {
                return Err($this.peek_error(ErrorCode::RecursionLimitExceeded));
            }
        }

        $this $($body)*

        if_checking_recursion_limit! {
            $this.remaining_depth += 1;
        }
    };
}

#[cfg(not(feature = "float_roundtrip"))]
static POW10: [f64; 309] = [
    1e000, 1e001, 1e002, 1e003, 1e004, 1e005, 1e006, 1e007, 1e008, 1e009, //
    1e010, 1e011, 1e012, 1e013, 1e014, 1e015, 1e016, 1e017, 1e018, 1e019, //
    1e020, 1e021, 1e022, 1e023, 1e024, 1e025, 1e026, 1e027, 1e028, 1e029, //
    1e030, 1e031, 1e032, 1e033, 1e034, 1e035, 1e036, 1e037, 1e038, 1e039, //
    1e040, 1e041, 1e042, 1e043, 1e044, 1e045, 1e046, 1e047, 1e048, 1e049, //
    1e050, 1e051, 1e052, 1e053, 1e054, 1e055, 1e056, 1e057, 1e058, 1e059, //
    1e060, 1e061, 1e062, 1e063, 1e064, 1e065, 1e066, 1e067, 1e068, 1e069, //
    1e070, 1e071, 1e072, 1e073, 1e074, 1e075, 1e076, 1e077, 1e078, 1e079, //
    1e080, 1e081, 1e082, 1e083, 1e084, 1e085, 1e086, 1e087, 1e088, 1e089, //
    1e090, 1e091, 1e092, 1e093, 1e094, 1e095, 1e096, 1e097, 1e098, 1e099, //
    1e100, 1e101, 1e102, 1e103, 1e104, 1e105, 1e106, 1e107, 1e108, 1e109, //
    1e110, 1e111, 1e112, 1e113, 1e114, 1e115, 1e116, 1e117, 1e118, 1e119, //
    1e120, 1e121, 1e122, 1e123, 1e124, 1e125, 1e126, 1e127, 1e128, 1e129, //
    1e130, 1e131, 1e132, 1e133, 1e134, 1e135, 1e136, 1e137, 1e138, 1e139, //
    1e140, 1e141, 1e142, 1e143, 1e144, 1e145, 1e146, 1e147, 1e148, 1e149, //
    1e150, 1e151, 1e152, 1e153, 1e154, 1e155, 1e156, 1e157, 1e158, 1e159, //
    1e160, 1e161, 1e162, 1e163, 1e164, 1e165, 1e166, 1e167, 1e168, 1e169, //
    1e170, 1e171, 1e172, 1e173, 1e174, 1e175, 1e176, 1e177, 1e178, 1e179, //
    1e180, 1e181, 1e182, 1e183, 1e184, 1e185, 1e186, 1e187, 1e188, 1e189, //
    1e190, 1e191, 1e192, 1e193, 1e194, 1e195, 1e196, 1e197, 1e198, 1e199, //
    1e200, 1e201, 1e202, 1e203, 1e204, 1e205, 1e206, 1e207, 1e208, 1e209, //
    1e210, 1e211, 1e212, 1e213, 1e214, 1e215, 1e216, 1e217, 1e218, 1e219, //
    1e220, 1e221, 1e222, 1e223, 1e224, 1e225, 1e226, 1e227, 1e228, 1e229, //
    1e230, 1e231, 1e232, 1e233, 1e234, 1e235, 1e236, 1e237, 1e238, 1e239, //
    1e240, 1e241, 1e242, 1e243, 1e244, 1e245, 1e246, 1e247, 1e248, 1e249, //
    1e250, 1e251, 1e252, 1e253, 1e254, 1e255, 1e256, 1e257, 1e258, 1e259, //
    1e260, 1e261, 1e262, 1e263, 1e264, 1e265, 1e266, 1e267, 1e268, 1e269, //
    1e270, 1e271, 1e272, 1e273, 1e274, 1e275, 1e276, 1e277, 1e278, 1e279, //
    1e280, 1e281, 1e282, 1e283, 1e284, 1e285, 1e286, 1e287, 1e288, 1e289, //
    1e290, 1e291, 1e292, 1e293, 1e294, 1e295, 1e296, 1e297, 1e298, 1e299, //
    1e300, 1e301, 1e302, 1e303, 1e304, 1e305, 1e306, 1e307, 1e308,
];

/// A structure that deserializes EnCom into Rust values.
pub struct Deserializer<R> {
    pub(crate) read: R,
    pub(crate) remaining_depth: u8,
    #[cfg(feature = "float_roundtrip")]
    single_precision: bool,
    #[cfg(feature = "unbounded_depth")]
    disable_recursion_limit: bool,
}

impl<'de, R> Deserializer<R>
where
    R: Read<'de>,
{
    /// Create an EnCom deserializer from one of the possible serde_encom input
    /// sources.
    ///
    /// Typically it is more convenient to use one of these methods instead:
    ///
    ///   - Deserializer::from_str
    ///   - Deserializer::from_slice
    ///   - Deserializer::from_reader
    pub fn new(read: R) -> Self {
        Deserializer {
            read,
            remaining_depth: 128,
            #[cfg(feature = "float_roundtrip")]
            single_precision: false,
            #[cfg(feature = "unbounded_depth")]
            disable_recursion_limit: false,
        }
    }
}

#[cfg(feature = "std")]
impl<R> Deserializer<IoRead<R>>
where
    R: crate::io::Read,
{
    /// Creates an EnCom deserializer from an `io::Read`.
    ///
    /// Reader-based deserializers do not support deserializing borrowed types
    /// like `&str`, since the `std::io::Read` trait has no non-copying methods
    /// -- everything it does involves copying bytes out of the data source.
    pub fn from_reader(reader: R) -> Self {
        Deserializer::new(IoRead::new(reader))
    }
}

impl<'a> Deserializer<SliceRead<'a>> {
    /// Creates an EnCom deserializer from a `&[u8]`.
    pub fn from_slice(bytes: &'a [u8]) -> Self {
        Deserializer::new(SliceRead::new(bytes))
    }
}

impl<'a> Deserializer<StrRead<'a>> {
    /// Creates an EnCom deserializer from a `&str`.
    pub fn from_str(s: &'a str) -> Self {
        Deserializer::new(StrRead::new(s))
    }
}

pub(crate) enum PreParser {
    SavedMap,
    Seq,
    SavedSeq(SavedType),
}

#[derive(PartialEq)]
pub(crate) enum SavedType {
    Str,
    Bytes,
    Number,
    FloatNumber,
    // ExponentNumber,
    None,
}

impl<'de, R: Read<'de>> Deserializer<R> {
    /// The `Deserializer::end` method should be called after a value has been fully deserialized.
    /// This allows the `Deserializer` to validate that the input stream is at the end or that it
    /// only has trailing whitespace.
    pub fn end(&mut self) -> Result<()> {
        match self.parse_whitespace()? {
            Some(_) => Err(self.peek_error(ErrorCode::TrailingCharacters)),
            None => Ok(()),
        }
    }

    /// Turn an EnCom deserializer into an iterator over values of type T.
    pub fn into_iter<T>(self) -> StreamDeserializer<'de, R, T>
    where
        T: de::Deserialize<'de>,
    {
        // This cannot be an implementation of std::iter::IntoIterator because
        // we need the caller to choose what T is.
        let offset = self.read.byte_offset();
        StreamDeserializer {
            de: self,
            offset,
            failed: false,
            output: PhantomData,
            lifetime: PhantomData,
        }
    }

    /// Parse arbitrarily deep EnCom structures without any consideration for
    /// overflowing the stack.
    ///
    /// You will want to provide some other way to protect against stack
    /// overflows, such as by wrapping your Deserializer in the dynamically
    /// growing stack adapter provided by the serde_stacker crate. Additionally
    /// you will need to be careful around other recursive operations on the
    /// parsed result which may overflow the stack after deserialization has
    /// completed, including, but not limited to, Display and Debug and Drop
    /// impls.
    ///
    /// *This method is only available if serde_encom is built with the
    /// `"unbounded_depth"` feature.*
    ///
    /// # Examples
    ///
    /// ```
    /// use serde::Deserialize;
    /// use serde_encom::Value;
    ///
    /// fn main() {
    ///     let mut encom = String::new();
    ///     for _ in 0..10000 {
    ///         encom = format!("[{}]", encom);
    ///     }
    ///
    ///     let mut deserializer = serde_encom::Deserializer::from_str(&encom);
    ///     deserializer.disable_recursion_limit();
    ///     let deserializer = serde_stacker::Deserializer::new(&mut deserializer);
    ///     let value = Value::deserialize(deserializer).unwrap();
    ///
    ///     carefully_drop_nested_arrays(value);
    /// }
    ///
    /// fn carefully_drop_nested_arrays(value: Value) {
    ///     let mut stack = vec![value];
    ///     while let Some(value) = stack.pop() {
    ///         if let Value::Array(array) = value {
    ///             stack.extend(array);
    ///         }
    ///     }
    /// }
    /// ```
    #[cfg(feature = "unbounded_depth")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unbounded_depth")))]
    pub fn disable_recursion_limit(&mut self) {
        self.disable_recursion_limit = true;
    }

    #[inline]
    pub(crate) fn peek(&mut self) -> Result<Option<u8>> {
        self.read.peek()
    }

    #[inline]
    fn peek_or_null(&mut self) -> Result<u8> {
        Ok(self.peek()?.unwrap_or(b'\x00'))
    }

    #[inline]
    pub(crate) fn eat_char(&mut self) {
        self.read.discard();
    }

    #[inline]
    pub(crate) fn next_char(&mut self) -> Result<Option<u8>> {
        self.read.next()
    }

    #[inline]
    fn next_char_or_null(&mut self) -> Result<u8> {
        Ok(self.next_char()?.unwrap_or(b'\x00'))
    }

    /// Error caused by a byte from next_char().
    #[cold]
    pub(crate) fn error(&self, reason: ErrorCode) -> Error {
        let position = self.read.position();
        Error::syntax(reason, position.line, position.column)
    }

    /// Error caused by a byte from peek().
    #[cold]
    pub(crate) fn peek_error(&self, reason: ErrorCode) -> Error {
        let position = self.read.peek_position();
        Error::syntax(reason, position.line, position.column)
    }

    /// Returns the first non-whitespace byte without consuming it, or `None` if
    /// EOF is encountered.
    pub(crate) fn parse_whitespace(&mut self) -> Result<Option<u8>> {
        loop {
            match self.peek()? {
                /* Some(b' ' | b'\n' | b'\t' | b'\r') */
                Some(ch) if ch < 0x21 => {
                    self.eat_char();
                }
                other => {
                    return Ok(other);
                }
            }
        }
    }

    #[inline]
    fn pre_parser_match(&mut self) -> Result<PreParser> {
        loop {
            match self.peek()? {
                Some(b':' | b'{' | b'[') => {
                    return Ok(PreParser::SavedMap);
                }
                Some(b'=') => return Ok(PreParser::SavedSeq(SavedType::Str)),
                Some(b'~') => return Ok(PreParser::SavedSeq(SavedType::Bytes)),
                Some(b'.') => return Ok(PreParser::SavedSeq(SavedType::FloatNumber)),
                /* Some(b'e' | b'E') => { // todo????
                    return Ok(PreParser::SavedSeq(SavedState::ExponentNumber))
                } */
                Some(b'}' | b']' | b' ' | b'\n' | b'\t' | b'\r') | None => {
                    return Ok(PreParser::SavedSeq(SavedType::Number));
                }
                _ => self.eat_char(),
            }
        }
    }

    /// `deserialize_any()` what value is after b'{' or init
    /// no validation of chars, because atoi_simd will do it, if it is numbers
    pub(crate) fn any_after_x7b(&mut self) -> Result<PreParser> {
        if let Some(ch) = self.parse_whitespace()? {
            if ch == b'{' || ch == b'[' || ch == b'-' || ch == b't' || ch == b'f' {
                return Ok(PreParser::Seq);
            }
            self.read.save_start();
        } else {
            return Err(self.error(ErrorCode::EofWhileParsingValue));
        }
        let ret = self.pre_parser_match();
        self.read.save_end();
        ret
    }

    /// checks end of string or bytes
    #[inline]
    pub(crate) fn end_of_str_or_bytes(&mut self) -> Result<()> {
        match self.peek()? {
            Some(b'}' | b']' | b' ' | b'\n' | b'\t' | b'\r') | None => Ok(()),
            _ => Err(self.peek_error(ErrorCode::UnexpectedEndOfString)),
        }
    }

    #[inline]
    pub(crate) fn deserialize_str_by_len<V>(&mut self, visitor: V, len: usize) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.eat_char();
        let res = visitor.visit_borrowed_str(self.read.read_str(len)?);
        self.end_of_str_or_bytes()?;
        res
    }

    #[inline]
    pub(crate) fn deserialize_bytes_by_len<V>(&mut self, visitor: V, len: usize) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.eat_char();
        let res = visitor.visit_borrowed_bytes(self.read.read_slice(len)?);
        self.end_of_str_or_bytes()?;
        res
    }

    /// what value is in main deserializer `deserialize_any()`
    pub(crate) fn any_map_value<V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let parsed_int = self.read.parse_int_any_pos()?;
        let ret = match self.peek()? {
            Some(b'=') => self.deserialize_str_by_len(visitor, parsed_int as usize),
            Some(b'~') => self.deserialize_bytes_by_len(visitor, parsed_int as usize),
            Some(b'.') => visitor.visit_f64(self.parse_decimal(true, parsed_int, 0)?),
            /* Some(b'e' | b'E') => {
                ParserNumber::F64(self.parse_exponent(true, integer, 0)?).visit(visitor)
            } */
            Some(b'}' | b']' | b' ' | b'\n' | b'\t' | b'\r') | None => {
                visitor.visit_u64(parsed_int)
            }
            Some(_) => Err(self.error(ErrorCode::ExpectedSomeIdent)), // todo: new error?
        };
        self.read.clear_saved();
        ret
    }

    #[cold]
    pub(crate) fn peek_invalid_type(&mut self, exp: &dyn Expected) -> Error {
        let err = match self.peek_or_null().unwrap_or(b'\x00') {
            b'n' => {
                self.eat_char();
                /* if let Err(err) = self.parse_ident(b"ull") {
                    return err;
                } */
                de::Error::invalid_type(Unexpected::Unit, exp)
            }
            b't' => {
                self.eat_char();
                /* if let Err(err) = self.parse_ident(b"rue") {
                    return err;
                } */
                de::Error::invalid_type(Unexpected::Bool(true), exp)
            }
            b'f' => {
                self.eat_char();
                /* if let Err(err) = self.parse_ident(b"alse") {
                    return err;
                } */
                de::Error::invalid_type(Unexpected::Bool(false), exp)
            }
            b'-' => {
                self.eat_char();
                match self.parse_any_number(false) {
                    Ok(n) => n.invalid_type(exp),
                    Err(err) => return err,
                }
            }
            b'0'..=b'9' => match self.parse_any_number(true) {
                Ok(n) => n.invalid_type(exp),
                Err(err) => return err,
            },
            /* b'"' => {
                self.eat_char();
                self.scratch.clear();
                match self.read.parse_str(&mut self.scratch) {
                    Ok(s) => de::Error::invalid_type(Unexpected::Str(&s), exp),
                    Err(err) => return err,
                }
            } */
            b'[' => de::Error::invalid_type(Unexpected::Seq, exp),
            b'{' => de::Error::invalid_type(Unexpected::Map, exp),
            _ => self.peek_error(ErrorCode::ExpectedSomeValue),
        };

        self.fix_position(err)
    }

    pub(crate) fn deserialize_number<'any, V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'any>,
    {
        let peek = match self.parse_whitespace()? {
            Some(b) => b,
            None => {
                return Err(self.peek_error(ErrorCode::EofWhileParsingValue));
            }
        };

        let value = match peek {
            b'-' => {
                self.eat_char();
                self.parse_integer(false)?.visit(visitor)
            }
            b'0'..=b'9' => self.parse_integer(true)?.visit(visitor),
            _ => Err(self.peek_invalid_type(&visitor)),
        };

        match value {
            Ok(value) => Ok(value),
            Err(err) => Err(self.fix_position(err)),
        }
    }

    #[cfg(feature = "float_roundtrip")]
    pub(crate) fn do_deserialize_f32<'any, V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'any>,
    {
        self.single_precision = true;
        let val = self.deserialize_number(visitor);
        self.single_precision = false;
        val
    }

    // TODO: rewrite
    pub(crate) fn do_deserialize_i128<'any, V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'any>,
    {
        let mut buf = String::new();

        match self.parse_whitespace()? {
            Some(b'-') => {
                self.eat_char();
                buf.push('-');
            }
            Some(_) => {}
            None => {
                return Err(self.peek_error(ErrorCode::EofWhileParsingValue));
            }
        };

        self.scan_integer128(&mut buf)?;

        let value = match buf.parse() {
            Ok(int) => visitor.visit_i128(int),
            Err(_) => {
                return Err(self.error(ErrorCode::NumberOutOfRange));
            }
        };

        match value {
            Ok(value) => Ok(value),
            Err(err) => Err(self.fix_position(err)),
        }
    }

    // TODO: rewrite
    pub(crate) fn do_deserialize_u128<'any, V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'any>,
    {
        match self.parse_whitespace()? {
            Some(b'-') => {
                return Err(self.peek_error(ErrorCode::NumberOutOfRange));
            }
            Some(_) => {}
            None => {
                return Err(self.peek_error(ErrorCode::EofWhileParsingValue));
            }
        }

        let mut buf = String::new();
        self.scan_integer128(&mut buf)?;

        let value = match buf.parse() {
            Ok(int) => visitor.visit_u128(int),
            Err(_) => {
                return Err(self.error(ErrorCode::NumberOutOfRange));
            }
        };

        match value {
            Ok(value) => Ok(value),
            Err(err) => Err(self.fix_position(err)),
        }
    }

    // TODO: remove?
    fn scan_integer128(&mut self, buf: &mut String) -> Result<()> {
        match self.next_char_or_null()? {
            b'0' => {
                buf.push('0');
                // There can be only one leading '0'.
                match self.peek_or_null()? {
                    b'0'..=b'9' => Err(self.peek_error(ErrorCode::InvalidNumber)),
                    _ => Ok(()),
                }
            }
            c @ b'1'..=b'9' => {
                buf.push(c as char);
                while let c @ b'0'..=b'9' = self.peek_or_null()? {
                    self.eat_char();
                    buf.push(c as char);
                }
                Ok(())
            }
            _ => Err(self.error(ErrorCode::InvalidNumber)),
        }
    }

    #[cold]
    pub(crate) fn fix_position(&self, err: Error) -> Error {
        err.fix_position(move |code| self.error(code))
    }

    /* fn parse_ident(&mut self, ident: &[u8]) -> Result<()> {
        for expected in ident {
            match self.next_char()? {
                None => {
                    return Err(self.error(ErrorCode::EofWhileParsingValue));
                }
                Some(next) => {
                    if next != *expected {
                        return Err(self.error(ErrorCode::ExpectedSomeIdent));
                    }
                }
            }
        }

        Ok(())
    } */

    /* fn parse_integer_old(&mut self, positive: bool) -> Result<ParserNumber> {
        let next = match self.next_char()? {
            Some(b) => b,
            None => {
                return Err(self.error(ErrorCode::EofWhileParsingValue));
            }
        };

        match next {
            b'0' => {
                // There can be only one leading '0'.
                match self.peek_or_null()? {
                    b'0'..=b'9' => Err(self.peek_error(ErrorCode::InvalidNumber)),
                    _ => self.parse_number(positive, 0),
                }
            }
            c @ b'1'..=b'9' => {
                // SIMD version
                /* self.scratch.clear();
                self.scratch.push(c);

                while let Some(ch) = self.peek()? {
                    if ch < b'0' || ch > b'9' {
                        break;
                    }
                    self.scratch.push(ch);
                    self.eat_char();
                }
                return self.parse_number(positive, parse(self.scratch.get_slice()).unwrap()); */

                // Original version
                let mut significand = (c & 0xF) as u64;

                loop {
                    match self.peek_or_null()? {
                        c @ b'0'..=b'9' => {
                            let digit = (c & 0xF) as u64;

                            // We need to be careful with overflow. If we can,
                            // try to keep the number as a `u64` until we grow
                            // too large. At that point, switch to parsing the
                            // value as a `f64`.
                            if overflow!(significand * 10 + digit, u64::MAX) {
                                return Ok(ParserNumber::F64(
                                    self.parse_long_integer(positive, significand)?,
                                ));
                            }

                            significand = significand * 10 + digit;
                            self.eat_char();
                        }
                        _ => {
                            return self.parse_number(positive, significand);
                        }
                    }
                }
            }
            _ => Err(self.error(ErrorCode::InvalidNumber)),
        }
    } */

    fn parse_integer(&mut self, positive: bool) -> Result<ParserNumber> {
        // if positive {
        let significand = self.read.parse_int_any_pos()?;
        self.parse_number(positive, significand)
        /* } else {
            self.parse_integer_old(false)
        } */
    }

    #[inline]
    fn parse_number(&mut self, positive: bool, significand: u64) -> Result<ParserNumber> {
        Ok(match self.peek_or_null()? {
            b'.' => ParserNumber::F64(self.parse_decimal(positive, significand, 0)?),
            b'e' | b'E' => ParserNumber::F64(self.parse_exponent(positive, significand, 0)?),
            _ => {
                if positive {
                    ParserNumber::U64(significand)
                } else {
                    let neg = (significand as i64).wrapping_neg();

                    // Convert into a float if we underflow, or on `-0`.
                    if neg >= 0 {
                        ParserNumber::F64(-(significand as f64))
                    } else {
                        ParserNumber::I64(neg)
                    }
                }
            }
        })
    }

    pub(crate) fn parse_decimal(
        &mut self,
        positive: bool,
        mut significand: u64,
        exponent_before_decimal_point: i32,
    ) -> Result<f64> {
        self.eat_char();

        let mut exponent_after_decimal_point = 0;
        while let c @ b'0'..=b'9' = self.peek_or_null()? {
            let digit = (c & 0xF) as u64;

            if overflow!(significand * 10 + digit, u64::MAX) {
                let exponent = exponent_before_decimal_point + exponent_after_decimal_point;
                return self.parse_decimal_overflow(positive, significand, exponent);
            }

            self.eat_char();
            significand = significand * 10 + digit;
            exponent_after_decimal_point -= 1;
        }

        // Error if there is not at least one digit after the decimal point.
        if exponent_after_decimal_point == 0 {
            match self.peek()? {
                Some(_) => return Err(self.peek_error(ErrorCode::InvalidNumber)),
                None => return Err(self.peek_error(ErrorCode::EofWhileParsingValue)),
            }
        }

        let exponent = exponent_before_decimal_point + exponent_after_decimal_point;
        match self.peek_or_null()? {
            b'e' | b'E' => self.parse_exponent(positive, significand, exponent),
            _ => self.f64_from_parts(positive, significand, exponent),
        }
    }

    fn parse_exponent(
        &mut self,
        positive: bool,
        significand: u64,
        starting_exp: i32,
    ) -> Result<f64> {
        self.eat_char();

        let positive_exp = match self.peek_or_null()? {
            b'+' => {
                self.eat_char();
                true
            }
            b'-' => {
                self.eat_char();
                false
            }
            _ => true,
        };

        let next = match self.next_char()? {
            Some(b) => b,
            None => {
                return Err(self.error(ErrorCode::EofWhileParsingValue));
            }
        };

        // Make sure a digit follows the exponent place.
        let mut exp = match next {
            c @ b'0'..=b'9' => (c & 0xF) as i32,
            _ => {
                return Err(self.error(ErrorCode::InvalidNumber));
            }
        };

        while let c @ b'0'..=b'9' = self.peek_or_null()? {
            self.eat_char();
            let digit = (c & 0xF) as i32;

            if overflow!(exp * 10 + digit, i32::MAX) {
                let zero_significand = significand == 0;
                return self.parse_exponent_overflow(positive, zero_significand, positive_exp);
            }

            exp = exp * 10 + digit;
        }

        let final_exp = if positive_exp {
            starting_exp.saturating_add(exp)
        } else {
            starting_exp.saturating_sub(exp)
        };

        self.f64_from_parts(positive, significand, final_exp)
    }

    #[cfg(feature = "float_roundtrip")]
    fn f64_from_parts(&mut self, positive: bool, significand: u64, exponent: i32) -> Result<f64> {
        let f = if self.single_precision {
            lexical::parse_concise_float::<f32>(significand, exponent) as f64
        } else {
            lexical::parse_concise_float::<f64>(significand, exponent)
        };

        if f.is_infinite() {
            Err(self.error(ErrorCode::NumberOutOfRange))
        } else {
            Ok(if positive { f } else { -f })
        }
    }

    #[cfg(not(feature = "float_roundtrip"))]
    fn f64_from_parts(
        &mut self,
        positive: bool,
        significand: u64,
        mut exponent: i32,
    ) -> Result<f64> {
        let mut f = significand as f64;
        loop {
            match POW10.get(exponent.wrapping_abs() as usize) {
                Some(&pow) => {
                    if exponent >= 0 {
                        f *= pow;
                        if f.is_infinite() {
                            return Err(self.error(ErrorCode::NumberOutOfRange));
                        }
                    } else {
                        f /= pow;
                    }
                    break;
                }
                None => {
                    if f == 0.0 {
                        break;
                    }
                    if exponent >= 0 {
                        return Err(self.error(ErrorCode::NumberOutOfRange));
                    }
                    f /= 1e308;
                    exponent += 308;
                }
            }
        }
        Ok(if positive { f } else { -f })
    }

    /* #[cfg(feature = "float_roundtrip")]
    #[cold]
    #[inline(never)]
    fn parse_long_integer(&mut self, positive: bool, partial_significand: u64) -> Result<f64> {
        // To deserialize floats we'll first push the integer and fraction
        // parts, both as byte strings, into the scratch buffer and then feed
        // both slices to lexical's parser. For example if the input is
        // `12.34e5` we'll push b"1234" into scratch and then pass b"12" and
        // b"34" to lexical. `integer_end` will be used to track where to split
        // the scratch buffer.
        //
        // Note that lexical expects the integer part to contain *no* leading
        // zeroes and the fraction part to contain *no* trailing zeroes. The
        // first requirement is already handled by the integer parsing logic.
        // The second requirement will be enforced just before passing the
        // slices to lexical in f64_long_from_parts.
        self.scratch.clear();
        self.scratch
            .extend_from_slice(itoa::Buffer::new().format(partial_significand).as_bytes());

        loop {
            match tri!(self.peek_or_null()) {
                c @ b'0'..=b'9' => {
                    self.scratch.push(c);
                    self.eat_char();
                }
                b'.' => {
                    self.eat_char();
                    return self.parse_long_decimal(positive, self.scratch.len());
                }
                b'e' | b'E' => {
                    return self.parse_long_exponent(positive, self.scratch.len());
                }
                _ => {
                    return self.f64_long_from_parts(positive, self.scratch.len(), 0);
                }
            }
        }
    }

    #[cfg(not(feature = "float_roundtrip"))]
    #[cold]
    #[inline(never)]
    fn parse_long_integer(&mut self, positive: bool, significand: u64) -> Result<f64> {
        let mut exponent = 0;
        loop {
            match self.peek_or_null()? {
                b'0'..=b'9' => {
                    self.eat_char();
                    // This could overflow... if your integer is gigabytes long.
                    // Ignore that possibility.
                    exponent += 1;
                }
                b'.' => {
                    return self.parse_decimal(positive, significand, exponent);
                }
                b'e' | b'E' => {
                    return self.parse_exponent(positive, significand, exponent);
                }
                _ => {
                    return self.f64_from_parts(positive, significand, exponent);
                }
            }
        }
    } */

    #[cfg(feature = "float_roundtrip")]
    #[cold]
    fn parse_long_decimal(&mut self, positive: bool, integer_end: usize) -> Result<f64> {
        let mut at_least_one_digit = integer_end < self.scratch.len();
        while let c @ b'0'..=b'9' = tri!(self.peek_or_null()) {
            self.scratch.push(c);
            self.eat_char();
            at_least_one_digit = true;
        }

        if !at_least_one_digit {
            match tri!(self.peek()) {
                Some(_) => return Err(self.peek_error(ErrorCode::InvalidNumber)),
                None => return Err(self.peek_error(ErrorCode::EofWhileParsingValue)),
            }
        }

        match tri!(self.peek_or_null()) {
            b'e' | b'E' => self.parse_long_exponent(positive, integer_end),
            _ => self.f64_long_from_parts(positive, integer_end, 0),
        }
    }

    #[cfg(feature = "float_roundtrip")]
    fn parse_long_exponent(&mut self, positive: bool, integer_end: usize) -> Result<f64> {
        self.eat_char();

        let positive_exp = match tri!(self.peek_or_null()) {
            b'+' => {
                self.eat_char();
                true
            }
            b'-' => {
                self.eat_char();
                false
            }
            _ => true,
        };

        let next = match tri!(self.next_char()) {
            Some(b) => b,
            None => {
                return Err(self.error(ErrorCode::EofWhileParsingValue));
            }
        };

        // Make sure a digit follows the exponent place.
        let mut exp = match next {
            c @ b'0'..=b'9' => (c & 0xF) as i32,
            _ => {
                return Err(self.error(ErrorCode::InvalidNumber));
            }
        };

        while let c @ b'0'..=b'9' = tri!(self.peek_or_null()) {
            self.eat_char();
            let digit = (c & 0xF) as i32;

            if overflow!(exp * 10 + digit, i32::MAX) {
                let zero_significand = self.scratch.iter().all(|&digit| digit == b'0');
                return self.parse_exponent_overflow(positive, zero_significand, positive_exp);
            }

            exp = exp * 10 + digit;
        }

        let final_exp = if positive_exp { exp } else { -exp };

        self.f64_long_from_parts(positive, integer_end, final_exp)
    }

    // This cold code should not be inlined into the middle of the hot
    // decimal-parsing loop above.
    #[cfg(feature = "float_roundtrip")]
    #[cold]
    #[inline(never)]
    fn parse_decimal_overflow(
        &mut self,
        positive: bool,
        significand: u64,
        exponent: i32,
    ) -> Result<f64> {
        let mut buffer = itoa::Buffer::new();
        let significand = buffer.format(significand);
        let fraction_digits = -exponent as usize;
        self.scratch.clear();
        if let Some(zeros) = fraction_digits.checked_sub(significand.len() + 1) {
            self.scratch.extend(iter::repeat(b'0').take(zeros + 1));
        }
        self.scratch.extend_from_slice(significand.as_bytes());
        let integer_end = self.scratch.len() - fraction_digits;
        self.parse_long_decimal(positive, integer_end)
    }

    #[cfg(not(feature = "float_roundtrip"))]
    #[cold]
    #[inline(never)]
    fn parse_decimal_overflow(
        &mut self,
        positive: bool,
        significand: u64,
        exponent: i32,
    ) -> Result<f64> {
        // The next multiply/add would overflow, so just ignore all further
        // digits.
        while let b'0'..=b'9' = self.peek_or_null()? {
            self.eat_char();
        }

        match self.peek_or_null()? {
            b'e' | b'E' => self.parse_exponent(positive, significand, exponent),
            _ => self.f64_from_parts(positive, significand, exponent),
        }
    }

    // This cold code should not be inlined into the middle of the hot
    // exponent-parsing loop above.
    #[cold]
    #[inline(never)]
    fn parse_exponent_overflow(
        &mut self,
        positive: bool,
        zero_significand: bool,
        positive_exp: bool,
    ) -> Result<f64> {
        // Error instead of +/- infinity.
        if !zero_significand && positive_exp {
            return Err(self.error(ErrorCode::NumberOutOfRange));
        }

        while let b'0'..=b'9' = self.peek_or_null()? {
            self.eat_char();
        }
        Ok(if positive { 0.0 } else { -0.0 })
    }

    #[cfg(feature = "float_roundtrip")]
    fn f64_long_from_parts(
        &mut self,
        positive: bool,
        integer_end: usize,
        exponent: i32,
    ) -> Result<f64> {
        let integer = &self.scratch[..integer_end];
        let fraction = &self.scratch[integer_end..];

        let f = if self.single_precision {
            lexical::parse_truncated_float::<f32>(integer, fraction, exponent) as f64
        } else {
            lexical::parse_truncated_float::<f64>(integer, fraction, exponent)
        };

        if f.is_infinite() {
            Err(self.error(ErrorCode::NumberOutOfRange))
        } else {
            Ok(if positive { f } else { -f })
        }
    }

    // TODO: rewrite, but it's unreachable
    pub(crate) fn parse_any_signed_number(&mut self) -> Result<ParserNumber> {
        unreachable!();
        /* let peek = match self.peek()? {
            Some(b) => b,
            None => {
                return Err(self.peek_error(ErrorCode::EofWhileParsingValue));
            }
        };

        let value = match peek {
            b'-' => {
                self.eat_char();
                self.parse_any_number(false)
            }
            b'0'..=b'9' => self.parse_any_number(true),
            _ => Err(self.peek_error(ErrorCode::InvalidNumber)),
        };

        let value = match self.peek()? {
            Some(_) => Err(self.peek_error(ErrorCode::InvalidNumber)),
            None => value,
        };

        match value {
            Ok(value) => Ok(value),
            // The de::Error impl creates errors with unknown line and column.
            // Fill in the position here by looking at the current index in the
            // input. There is no way to tell whether this should call `error`
            // or `peek_error` so pick the one that seems correct more often.
            // Worst case, the position is off by one character.
            Err(err) => Err(self.fix_position(err)),
        } */
    }

    #[cfg(not(feature = "arbitrary_precision"))]
    #[inline]
    pub(crate) fn parse_any_number(&mut self, positive: bool) -> Result<ParserNumber> {
        self.parse_integer(positive)
    }

    #[cfg(feature = "arbitrary_precision")]
    fn parse_any_number(&mut self, positive: bool) -> Result<ParserNumber> {
        let mut buf = String::with_capacity(16);
        if !positive {
            buf.push('-');
        }
        self.scan_integer(&mut buf)?;
        if positive {
            if let Ok(unsigned) = buf.parse() {
                return Ok(ParserNumber::U64(unsigned));
            }
        } else {
            if let Ok(signed) = buf.parse() {
                return Ok(ParserNumber::I64(signed));
            }
        }
        Ok(ParserNumber::String(buf))
    }

    #[cfg(feature = "arbitrary_precision")]
    fn scan_or_eof(&mut self, buf: &mut String) -> Result<u8> {
        match tri!(self.next_char()) {
            Some(b) => {
                buf.push(b as char);
                Ok(b)
            }
            None => Err(self.error(ErrorCode::EofWhileParsingValue)),
        }
    }

    #[cfg(feature = "arbitrary_precision")]
    fn scan_integer(&mut self, buf: &mut String) -> Result<()> {
        match tri!(self.scan_or_eof(buf)) {
            b'0' => {
                // There can be only one leading '0'.
                match tri!(self.peek_or_null()) {
                    b'0'..=b'9' => Err(self.peek_error(ErrorCode::InvalidNumber)),
                    _ => self.scan_number(buf),
                }
            }
            b'1'..=b'9' => loop {
                match tri!(self.peek_or_null()) {
                    c @ b'0'..=b'9' => {
                        self.eat_char();
                        buf.push(c as char);
                    }
                    _ => {
                        return self.scan_number(buf);
                    }
                }
            },
            _ => Err(self.error(ErrorCode::InvalidNumber)),
        }
    }

    #[cfg(feature = "arbitrary_precision")]
    fn scan_number(&mut self, buf: &mut String) -> Result<()> {
        match tri!(self.peek_or_null()) {
            b'.' => self.scan_decimal(buf),
            e @ (b'e' | b'E') => self.scan_exponent(e as char, buf),
            _ => Ok(()),
        }
    }

    #[cfg(feature = "arbitrary_precision")]
    fn scan_decimal(&mut self, buf: &mut String) -> Result<()> {
        self.eat_char();
        buf.push('.');

        let mut at_least_one_digit = false;
        while let c @ b'0'..=b'9' = tri!(self.peek_or_null()) {
            self.eat_char();
            buf.push(c as char);
            at_least_one_digit = true;
        }

        if !at_least_one_digit {
            match tri!(self.peek()) {
                Some(_) => return Err(self.peek_error(ErrorCode::InvalidNumber)),
                None => return Err(self.peek_error(ErrorCode::EofWhileParsingValue)),
            }
        }

        match tri!(self.peek_or_null()) {
            e @ (b'e' | b'E') => self.scan_exponent(e as char, buf),
            _ => Ok(()),
        }
    }

    #[cfg(feature = "arbitrary_precision")]
    fn scan_exponent(&mut self, e: char, buf: &mut String) -> Result<()> {
        self.eat_char();
        buf.push(e);

        match tri!(self.peek_or_null()) {
            b'+' => {
                self.eat_char();
                buf.push('+');
            }
            b'-' => {
                self.eat_char();
                buf.push('-');
            }
            _ => {}
        }

        // Make sure a digit follows the exponent place.
        match tri!(self.scan_or_eof(buf)) {
            b'0'..=b'9' => {}
            _ => {
                return Err(self.error(ErrorCode::InvalidNumber));
            }
        }

        while let c @ b'0'..=b'9' = tri!(self.peek_or_null()) {
            self.eat_char();
            buf.push(c as char);
        }

        Ok(())
    }

    pub(crate) fn parse_object_colon(&mut self) -> Result<()> {
        match self.parse_whitespace()? {
            Some(b':') => {
                self.eat_char();
                Ok(())
            }
            Some(b'{' | b'[') => Ok(()), // fixed ":{" here
            Some(_) => Err(self.peek_error(ErrorCode::ExpectedColon)),
            None => Err(self.peek_error(ErrorCode::EofWhileParsingObject)),
        }
    }

    fn end_seq(&mut self) -> Result<()> {
        match self.parse_whitespace()? {
            Some(b']') => {
                self.eat_char();
                Ok(())
            }
            /* Some(b' ') => {
                self.eat_char();
                match self.parse_whitespace() {
                    Ok(Some(b'}')) => Err(self.peek_error(ErrorCode::TrailingComma)),
                    _ => Err(self.peek_error(ErrorCode::TrailingCharacters)),
                }
            } */
            Some(_) => Err(self.peek_error(ErrorCode::TrailingCharacters)),
            None => Err(self.peek_error(ErrorCode::EofWhileParsingList)),
        }
    }

    #[inline]
    pub(crate) fn end_seq_init(&mut self) -> Result<()> {
        match self.parse_whitespace()? {
            Some(_) => Err(self.peek_error(ErrorCode::TrailingCharacters)),
            None => Ok(()),
        }
    }

    fn end_map(&mut self) -> Result<()> {
        match self.parse_whitespace()? {
            Some(b'}') => {
                self.eat_char();
                Ok(())
            }
            // Some(b' ') => Err(self.peek_error(ErrorCode::TrailingComma)),
            Some(_) => Err(self.peek_error(ErrorCode::TrailingCharacters)),
            None => Err(self.peek_error(ErrorCode::EofWhileParsingObject)),
        }
    }

    #[inline]
    pub(crate) fn end_map_init(&mut self) -> Result<()> {
        match self.parse_whitespace()? {
            Some(_) => Err(self.peek_error(ErrorCode::TrailingCharacters)),
            None => Ok(()),
        }
    }

    fn ignore_value(&mut self) -> Result<()> {
        self.read.clear_saved();
        let mut enclosing = None;

        loop {
            let peek = match self.parse_whitespace()? {
                Some(b) => b,
                None => {
                    return Err(self.peek_error(ErrorCode::EofWhileParsingValue));
                }
            };

            let frame = match peek {
                b'n' => {
                    self.eat_char();
                    // self.parse_ident(b"ull")?;
                    None
                }
                b't' => {
                    self.eat_char();
                    // self.parse_ident(b"rue")?;
                    None
                }
                b'f' => {
                    self.eat_char();
                    // self.parse_ident(b"alse")?;
                    None
                }
                b'-' => {
                    self.eat_char();
                    self.ignore_integer()?;
                    None
                }
                b'0'..=b'9' => {
                    self.ignore_integer()?;
                    None
                }
                /* b'"' => {
                    self.eat_char();
                    self.read.ignore_str()?;
                    None
                } */
                frame @ b'{' | frame @ b'[' => {
                    /* if let Some(v) = enclosing.take() {
                        self.scratch.push(v);
                    } */
                    self.eat_char();
                    Some(frame)
                }
                _ => return Err(self.peek_error(ErrorCode::ExpectedSomeValue)),
            };

            let (mut accept_comma, frame) = match frame {
                Some(frame) => (false, frame),
                None => match enclosing.take() {
                    Some(frame) => (true, frame),
                    // None => match self.scratch.pop() {
                    // Some(frame) => (true, frame),
                    None => return Ok(()),
                    // },
                },
            };

            loop {
                match self.parse_whitespace()? {
                    Some(b' ') if accept_comma => {
                        self.eat_char();
                        break;
                    }
                    Some(b']') if frame == b'[' => {}
                    Some(b'}') if frame == b'{' => {}
                    Some(_) => {
                        if accept_comma {
                            return Err(self.peek_error(match frame {
                                b'[' => ErrorCode::ExpectedListCommaOrEnd,
                                b'{' => ErrorCode::ExpectedObjectCommaOrEnd,
                                _ => unreachable!(),
                            }));
                        } else {
                            break;
                        }
                    }
                    None => {
                        return Err(self.peek_error(match frame {
                            b'[' => ErrorCode::EofWhileParsingList,
                            b'{' => ErrorCode::EofWhileParsingObject,
                            _ => unreachable!(),
                        }));
                    }
                }

                self.eat_char();
                /* frame = match self.scratch.pop() {
                    Some(frame) => frame,
                    None => return Ok(()),
                }; */
                accept_comma = true;
            }

            if frame == b'{' || frame == b'[' {
                match self.parse_whitespace()? {
                    Some(b'"') => self.eat_char(),
                    Some(_) => return Err(self.peek_error(ErrorCode::KeyMustBeAString)),
                    None => return Err(self.peek_error(ErrorCode::EofWhileParsingObject)),
                }
                self.read.ignore_str()?;
                match self.parse_whitespace()? {
                    Some(b':') => self.eat_char(),
                    Some(_) => return Err(self.peek_error(ErrorCode::ExpectedColon)),
                    None => return Err(self.peek_error(ErrorCode::EofWhileParsingObject)),
                }
            }

            enclosing = Some(frame);
        }
    }

    fn ignore_integer(&mut self) -> Result<()> {
        match self.next_char_or_null()? {
            b'0' => {
                // There can be only one leading '0'.
                if let b'0'..=b'9' = self.peek_or_null()? {
                    return Err(self.peek_error(ErrorCode::InvalidNumber));
                }
            }
            b'1'..=b'9' => {
                while let b'0'..=b'9' = self.peek_or_null()? {
                    self.eat_char();
                }
            }
            _ => {
                return Err(self.error(ErrorCode::InvalidNumber));
            }
        }

        match self.peek_or_null()? {
            b'.' => self.ignore_decimal(),
            b'e' | b'E' => self.ignore_exponent(),
            _ => Ok(()),
        }
    }

    fn ignore_decimal(&mut self) -> Result<()> {
        self.eat_char();

        let mut at_least_one_digit = false;
        while let b'0'..=b'9' = self.peek_or_null()? {
            self.eat_char();
            at_least_one_digit = true;
        }

        if !at_least_one_digit {
            return Err(self.peek_error(ErrorCode::InvalidNumber));
        }

        match self.peek_or_null()? {
            b'e' | b'E' => self.ignore_exponent(),
            _ => Ok(()),
        }
    }

    fn ignore_exponent(&mut self) -> Result<()> {
        self.eat_char();

        match self.peek_or_null()? {
            b'+' | b'-' => self.eat_char(),
            _ => {}
        }

        // Make sure a digit follows the exponent place.
        match self.next_char_or_null()? {
            b'0'..=b'9' => {}
            _ => {
                return Err(self.error(ErrorCode::InvalidNumber));
            }
        }

        while let b'0'..=b'9' = self.peek_or_null()? {
            self.eat_char();
        }

        Ok(())
    }

    #[cfg(feature = "raw_value")]
    fn deserialize_raw_value<V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.parse_whitespace()?;
        self.read.begin_raw_buffering();
        self.ignore_value()?;
        self.read.end_raw_buffering(visitor)
    }
}

impl<'de, R: Read<'de>> de::Deserializer<'de> for &mut Deserializer<R> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let peek = match self.parse_whitespace()? {
            Some(b) => b,
            None => {
                return Err(self.peek_error(ErrorCode::EofWhileParsingValue));
            }
        };

        let value = match peek {
            b'n' => {
                self.eat_char();
                // self.parse_ident(b"ull")?;
                visitor.visit_unit()
            }
            b't' => {
                self.eat_char();
                // self.parse_ident(b"rue")?;
                visitor.visit_bool(true)
            }
            b'f' => {
                self.eat_char();
                // self.parse_ident(b"alse")?;
                visitor.visit_bool(false)
            }
            b'-' => {
                self.eat_char();
                self.parse_any_number(false)?.visit(visitor)
            }
            // b'0'..=b'9' => self.parse_any_number(true)?.visit(visitor),
            /* b'"' => {
                self.eat_char();
                self.scratch.clear();
                match self.read.parse_str(&mut self.scratch)? {
                    Reference::Borrowed(s) => visitor.visit_borrowed_str(s),
                    Reference::Copied(s) => visitor.visit_str(s),
                }
            } */
            /* b'[' => {
                check_recursion! {
                    self.eat_char();
                    let ret = visitor.visit_seq(SeqAccess::new(self));
                }

                match (ret, self.end_seq()) {
                    (Ok(ret), Ok(())) => Ok(ret),
                    (Err(err), _) | (_, Err(err)) => Err(err),
                }
            } */
            b'{' | b'[' => {
                check_recursion! {
                    self.eat_char();
                    let value = match self.any_after_x7b()? {
                        PreParser::SavedMap => {
                            let value = visitor.visit_map(SavedMapAccess::new(self));
                            (value, self.end_map())
                        }
                        PreParser::SavedSeq(v) => {
                            let value = visitor.visit_seq(SavedSeqAccess::new(self, v));
                            (value, self.end_seq())
                        }
                        PreParser::Seq => {
                            let value = visitor.visit_seq(SeqAccess::new(self));
                            (value, self.end_seq())
                        }
                    };
                }
                match value {
                    (Ok(ret), Ok(())) => Ok(ret),
                    (Err(err), _) | (_, Err(err)) => Err(err),
                }
            }
            _ => self.any_map_value(visitor),
        };

        match value {
            Ok(value) => Ok(value),
            // The de::Error impl creates errors with unknown line and column.
            // Fill in the position here by looking at the current index in the
            // input. There is no way to tell whether this should call `error`
            // or `peek_error` so pick the one that seems correct more often.
            // Worst case, the position is off by one character.
            Err(err) => Err(self.fix_position(err)),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let peek = match self.parse_whitespace()? {
            Some(b) => b,
            None => {
                return Err(self.peek_error(ErrorCode::EofWhileParsingValue));
            }
        };

        let value = match peek {
            b't' => {
                self.eat_char();
                // self.parse_ident(b"rue")?;
                visitor.visit_bool(true)
            }
            b'f' => {
                self.eat_char();
                // self.parse_ident(b"alse")?;
                visitor.visit_bool(false)
            }
            _ => Err(self.peek_invalid_type(&visitor)),
        };

        match value {
            Ok(value) => Ok(value),
            Err(err) => Err(self.fix_position(err)),
        }
    }

    deserialize_number!(deserialize_i8);
    deserialize_number!(deserialize_i16);
    deserialize_number!(deserialize_i32);
    deserialize_number!(deserialize_i64);
    deserialize_number!(deserialize_u8);
    deserialize_number!(deserialize_u16);
    deserialize_number!(deserialize_u32);
    deserialize_number!(deserialize_u64);
    #[cfg(not(feature = "float_roundtrip"))]
    deserialize_number!(deserialize_f32);
    deserialize_number!(deserialize_f64);

    #[cfg(feature = "float_roundtrip")]
    deserialize_number!(deserialize_f32, do_deserialize_f32);
    deserialize_number!(deserialize_i128, do_deserialize_i128);
    deserialize_number!(deserialize_u128, do_deserialize_u128);

    #[inline]
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        /* let peek = match self.parse_whitespace()? {
            Some(b) => b,
            None => {
                return Err(self.peek_error(ErrorCode::EofWhileParsingValue));
            }
        }; */

        // takes str len before :
        let value = if let ParserNumber::U64(len) = self.parse_integer(true)? {
            self.deserialize_str_by_len(visitor, len as usize)
        } else {
            Err(self.peek_invalid_type(&visitor))
        };

        /* let value = match peek {
            b'"' => {
                self.eat_char();
                self.scratch.clear();
                match self.read.parse_str(&mut self.scratch)? {
                    Reference::Borrowed(s) => visitor.visit_borrowed_str(s),
                    Reference::Copied(s) => visitor.visit_str(s),
                }
            }
            _ => Err(self.peek_invalid_type(&visitor)),
        }; */

        match value {
            Ok(value) => Ok(value),
            Err(err) => Err(self.fix_position(err)),
        }
    }

    #[inline]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    /// Parses an EnCom string as bytes. Note that this function does not check
    /// whether the bytes represent a valid UTF-8 string.
    ///
    /// The relevant part of the EnCom specification is Section 8.2 of [RFC
    /// 7159]:
    ///
    /// > When all the strings represented in an EnCom text are composed entirely
    /// > of Unicode characters (however escaped), then that EnCom text is
    /// > interoperable in the sense that all software implementations that
    /// > parse it will agree on the contents of names and of string values in
    /// > objects and arrays.
    /// >
    /// > However, the ABNF in this specification allows member names and string
    /// > values to contain bit sequences that cannot encode Unicode characters;
    /// > for example, "\uDEAD" (a single unpaired UTF-16 surrogate). Instances
    /// > of this have been observed, for example, when a library truncates a
    /// > UTF-16 string without checking whether the truncation split a
    /// > surrogate pair.  The behavior of software that receives EnCom texts
    /// > containing such values is unpredictable; for example, implementations
    /// > might return different values for the length of a string value or even
    /// > suffer fatal runtime exceptions.
    ///
    /// [RFC 7159]: https://tools.ietf.org/html/rfc7159
    ///
    /// The behavior of serde_encom is specified to fail on non-UTF-8 strings
    /// when deserializing into Rust UTF-8 string types such as String, and
    /// succeed with non-UTF-8 bytes when deserializing using this method.
    ///
    /// Escape sequences are processed as usual, and for `\uXXXX` escapes it is
    /// still checked if the hex number represents a valid Unicode code point.
    ///
    /// # Examples
    ///
    /// You can use this to parse EnCom strings containing invalid UTF-8 bytes,
    /// or unpaired surrogates.
    ///
    /// ```
    ///
    /// fn look_at_bytes() -> Result<(), serde_encom::Error> {
    ///     let encom_data = b"3=\xe5\x00\xe5";
    ///     let bytes: &[u8] = serde_encom::from_slice(encom_data)?;
    ///
    ///     assert_eq!(b'\xe5', bytes[0]);
    ///     assert_eq!(b'\0', bytes[1]);
    ///     assert_eq!(b'\xe5', bytes[2]);
    ///
    ///
    ///     Ok(())
    /// }
    /// #
    /// # look_at_bytes().unwrap();
    /// ```
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        /* let peek = match self.parse_whitespace()? {
            Some(b) => b,
            None => {
                return Err(self.peek_error(ErrorCode::EofWhileParsingValue));
            }
        };

        let value = match peek {
            /* b'"' => {
                self.eat_char();
                self.scratch.clear();
                match self.read.parse_str_raw(&mut self.scratch)? {
                    Reference::Borrowed(b) => visitor.visit_borrowed_bytes(b),
                    Reference::Copied(b) => visitor.visit_bytes(b),
                }
            } */
            b'{' => self.deserialize_seq(visitor),
            _ => Err(self.peek_invalid_type(&visitor)),
        }; */

        let value = if let ParserNumber::U64(len) = self.parse_integer(true)? {
            self.deserialize_bytes_by_len(visitor, len as usize)
        } else {
            Err(self.peek_invalid_type(&visitor))
        };

        match value {
            Ok(value) => Ok(value),
            Err(err) => Err(self.fix_position(err)),
        }
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    /// Parses a `null` as a None, and any other values as a `Some(...)`.
    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.parse_whitespace()? {
            Some(b'n') => {
                self.eat_char();
                // self.parse_ident(b"ull")?;
                visitor.visit_none()
            }
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let peek = match self.parse_whitespace()? {
            Some(b) => b,
            None => {
                return Err(self.peek_error(ErrorCode::EofWhileParsingValue));
            }
        };

        let value = match peek {
            b'n' => {
                self.eat_char();
                // self.parse_ident(b"ull")?;
                visitor.visit_unit()
            }
            _ => Err(self.peek_invalid_type(&visitor)),
        };

        match value {
            Ok(value) => Ok(value),
            Err(err) => Err(self.fix_position(err)),
        }
    }

    #[inline]
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    /// Parses a newtype struct as the underlying value.
    #[inline]
    fn deserialize_newtype_struct<V>(self, name: &str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        #[cfg(feature = "raw_value")]
        {
            if name == crate::raw::TOKEN {
                return self.deserialize_raw_value(visitor);
            }
        }

        let _ = name;
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let peek = match self.parse_whitespace()? {
            Some(b) => b,
            None => {
                return Err(self.peek_error(ErrorCode::EofWhileParsingValue));
            }
        };

        let value = match peek {
            b'{' | b'[' => {
                check_recursion! {
                    self.eat_char();
                    let ret = visitor.visit_seq(SeqAccess::new(self));
                }

                match (ret, self.end_seq()) {
                    (Ok(ret), Ok(())) => Ok(ret),
                    (Err(err), _) | (_, Err(err)) => Err(err),
                }
            }
            _ => Err(self.peek_invalid_type(&visitor)),
        };

        match value {
            Ok(value) => Ok(value),
            Err(err) => Err(self.fix_position(err)),
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
        let peek = match self.parse_whitespace()? {
            Some(b) => b,
            None => {
                return Err(self.peek_error(ErrorCode::EofWhileParsingValue));
            }
        };

        let value = match peek {
            b'{' => {
                check_recursion! {
                    self.eat_char();
                    let ret = visitor.visit_map(MapAccess::new(self));
                }

                match (ret, self.end_map()) {
                    (Ok(ret), Ok(())) => Ok(ret),
                    (Err(err), _) | (_, Err(err)) => Err(err),
                }
            }
            _ => Err(self.peek_invalid_type(&visitor)),
        };

        match value {
            Ok(value) => Ok(value),
            Err(err) => Err(self.fix_position(err)),
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
        let peek = match self.parse_whitespace()? {
            Some(b) => b,
            None => {
                return Err(self.peek_error(ErrorCode::EofWhileParsingValue));
            }
        };

        let value = match peek {
            /* b'[' => {
                check_recursion! {
                    self.eat_char();
                    let ret = visitor.visit_seq(SeqAccess::new(self));
                }

                match (ret, self.end_seq()) {
                    (Ok(ret), Ok(())) => Ok(ret),
                    (Err(err), _) | (_, Err(err)) => Err(err),
                }
            } */
            b'{' => {
                check_recursion! {
                    self.eat_char();
                    let ret = visitor.visit_map(MapAccess::new(self));
                }

                match (ret, self.end_map()) {
                    (Ok(ret), Ok(())) => Ok(ret),
                    (Err(err), _) | (_, Err(err)) => Err(err),
                }
            }
            _ => Err(self.peek_invalid_type(&visitor)),
        };

        match value {
            Ok(value) => Ok(value),
            Err(err) => Err(self.fix_position(err)),
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
        match self.parse_whitespace()? {
            Some(b'{') => {
                check_recursion! {
                    self.eat_char();
                    let value = visitor.visit_enum(VariantAccess::new(self))?;
                }

                match self.parse_whitespace()? {
                    Some(b'}') => {
                        self.eat_char();
                        Ok(value)
                    }
                    Some(_) => Err(self.error(ErrorCode::ExpectedSomeValue)),
                    None => Err(self.error(ErrorCode::EofWhileParsingObject)),
                }
            }
            Some(_) => visitor.visit_enum(VariantAccess::new(self)), // it skips extra {} of enum
            // Some(b'"') => visitor.visit_enum(UnitVariantAccess::new(self)),
            // Some(_) => Err(self.peek_error(ErrorCode::ExpectedSomeValue)),
            None => Err(self.peek_error(ErrorCode::EofWhileParsingValue)),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // self.deserialize_str(visitor) // originaly
        if self.parse_whitespace()?.is_none() {
            return Err(self.peek_error(ErrorCode::EofWhileParsingValue));
        };

        let value = {
            // self.read.clear_saved();
            match self.read.parse_str()? {
                Reference::Borrowed(s) => visitor.visit_borrowed_str(s),
                Reference::Copied(s) => visitor.visit_str(s),
            }
        };

        match value {
            Ok(value) => Ok(value),
            Err(err) => Err(self.fix_position(err)),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.ignore_value()?;
        visitor.visit_unit()
    }
}
