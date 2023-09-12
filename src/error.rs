//! When serializing or deserializing EnCom goes wrong.

use crate::io;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use atoi_simd::AtoiSimdError;
use core::fmt::{self, Debug, Display};
use core::result;
use core::str::FromStr;
use serde::{de, ser};
#[cfg(feature = "std")]
use std::error;
#[cfg(feature = "std")]
use std::io::ErrorKind;

/// This type represents all possible errors that can occur when serializing or
/// deserializing EnCom data.
pub struct Error {
    /// This `Box` allows us to keep the size of `Error` as small as possible. A
    /// larger `Error` type was substantially slower due to all the functions
    /// that pass around `Result<T, Error>`.
    err: Box<ErrorImpl>,
}

/// Alias for a `Result` with the error type `serde_encom::Error`.
pub type Result<T> = result::Result<T, Error>;

impl Error {
    /// One-based line number at which the error was detected.
    ///
    /// Characters in the first line of the input (before the first newline
    /// character) are in line 1.
    pub fn line(&self) -> usize {
        self.err.line
    }

    /// One-based column number at which the error was detected.
    ///
    /// The first character in the input and any characters immediately
    /// following a newline character are in column 1.
    ///
    /// Note that errors may occur in column 0, for example if a read from an IO
    /// stream fails immediately following a previously read newline character.
    pub fn column(&self) -> usize {
        self.err.column
    }

    /// Categorizes the cause of this error.
    ///
    /// - `ErrorCategory::Io` - failure to read or write bytes on an IO stream
    /// - `ErrorCategory::Syntax` - input that is not syntactically valid EnCom
    /// - `ErrorCategory::Data` - input data that is semantically incorrect
    /// - `ErrorCategory::Eof` - unexpected end of the input data
    pub fn classify(&self) -> ErrorCategory {
        match self.err.code {
            ErrorCode::Message(_) => ErrorCategory::Data,
            ErrorCode::Io(_) => ErrorCategory::Io,
            ErrorCode::EofWhileParsingList
            | ErrorCode::EofWhileParsingObject
            | ErrorCode::EofWhileParsingString
            | ErrorCode::EofWhileParsingValue => ErrorCategory::Eof,
            ErrorCode::ExpectedColon
            // | ErrorCode::ExpectedListCommaOrEnd
            | ErrorCode::ExpectedObjectCommaOrEnd
            | ErrorCode::ExpectedSomeIdent
            | ErrorCode::ExpectedSomeValue
            | ErrorCode::ExpectedDoubleQuote
            | ErrorCode::InvalidEscape
            | ErrorCode::InvalidNumber
            | ErrorCode::NumberOutOfRange
            | ErrorCode::InvalidUnicodeCodePoint
            | ErrorCode::ControlCharacterWhileParsingString
            | ErrorCode::KeyMustBeAString
            | ErrorCode::ExpectedNumericKey
            | ErrorCode::FloatKeyMustBeFinite
            // | ErrorCode::LoneLeadingSurrogateInHexEscape
            | ErrorCode::TrailingComma
            | ErrorCode::TrailingCharacters
            // | ErrorCode::UnexpectedEndOfHexEscape
            | ErrorCode::UnexpectedEndOfString
            | ErrorCode::RecursionLimitExceeded => ErrorCategory::Syntax,
        }
    }

    /// Returns true if this error was caused by a failure to read or write
    /// bytes on an IO stream.
    pub fn is_io(&self) -> bool {
        self.classify() == ErrorCategory::Io
    }

    /// Returns true if this error was caused by input that was not
    /// syntactically valid EnCom.
    pub fn is_syntax(&self) -> bool {
        self.classify() == ErrorCategory::Syntax
    }

    /// Returns true if this error was caused by input data that was
    /// semantically incorrect.
    ///
    /// For example, EnCom containing a number is semantically incorrect when the
    /// type being deserialized into holds a String.
    pub fn is_data(&self) -> bool {
        self.classify() == ErrorCategory::Data
    }

    /// Returns true if this error was caused by prematurely reaching the end of
    /// the input data.
    ///
    /// Callers that process streaming input may be interested in retrying the
    /// deserialization once more data is available.
    pub fn is_eof(&self) -> bool {
        self.classify() == ErrorCategory::Eof
    }

    /// The kind reported by the underlying standard library I/O error, if this
    /// error was caused by a failure to read or write bytes on an I/O stream.
    ///
    /// # Example
    ///
    /// ```
    /// use serde_encom::Value;
    /// use std::io::{self, ErrorKind, Read};
    /// use std::process;
    ///
    /// struct ReaderThatWillTimeOut<'a>(&'a [u8]);
    ///
    /// impl<'a> Read for ReaderThatWillTimeOut<'a> {
    ///     fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    ///         if self.0.is_empty() {
    ///             Err(io::Error::new(ErrorKind::TimedOut, "timed out"))
    ///         } else {
    ///             self.0.read(buf)
    ///         }
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let reader = ReaderThatWillTimeOut(br#" {"k": "#);
    ///
    ///     let _: Value = match serde_encom::from_reader(reader) {
    ///         Ok(value) => value,
    ///         Err(error) => {
    ///             if error.io_error_kind() == Some(ErrorKind::TimedOut) {
    ///                 // Maybe this application needs to retry certain kinds of errors.
    ///
    ///                 # return;
    ///             } else {
    ///                 eprintln!("error: {}", error);
    ///                 process::exit(1);
    ///             }
    ///         }
    ///     };
    /// }
    /// ```
    #[cfg(feature = "std")]
    pub fn io_error_kind(&self) -> Option<ErrorKind> {
        if let ErrorCode::Io(io_error) = &self.err.code {
            Some(io_error.kind())
        } else {
            None
        }
    }
}

/// Categorizes the cause of a `serde_encom::Error`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ErrorCategory {
    /// The error was caused by a failure to read or write bytes on an IO
    /// stream.
    Io,

    /// The error was caused by input that was not syntactically valid EnCom.
    Syntax,

    /// The error was caused by input data that was semantically incorrect.
    ///
    /// For example, EnCom containing a number is semantically incorrect when the
    /// type being deserialized into holds a String.
    Data,

    /// The error was caused by prematurely reaching the end of the input data.
    ///
    /// Callers that process streaming input may be interested in retrying the
    /// deserialization once more data is available.
    Eof,
}

#[cfg(feature = "std")]
#[allow(clippy::fallible_impl_from)]
impl From<Error> for io::Error {
    /// Convert a `serde_encom::Error` into an `io::Error`.
    ///
    /// EnCom syntax and data errors are turned into `InvalidData` IO errors.
    /// EOF errors are turned into `UnexpectedEof` IO errors.
    ///
    /// ```
    /// use std::io;
    ///
    /// enum MyError {
    ///     Io(io::Error),
    ///     EnCom(serde_encom::Error),
    /// }
    ///
    /// impl From<serde_encom::Error> for MyError {
    ///     fn from(err: serde_encom::Error) -> MyError {
    ///         use serde_encom::ErrorCategory;
    ///         match err.classify() {
    ///             ErrorCategory::Io => {
    ///                 MyError::Io(err.into())
    ///             }
    ///             ErrorCategory::Syntax | ErrorCategory::Data | ErrorCategory::Eof => {
    ///                 MyError::EnCom(err)
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    fn from(j: Error) -> Self {
        if let ErrorCode::Io(err) = j.err.code {
            err
        } else {
            match j.classify() {
                ErrorCategory::Io => unreachable!(),
                ErrorCategory::Syntax | ErrorCategory::Data => {
                    io::Error::new(ErrorKind::InvalidData, j)
                }
                ErrorCategory::Eof => io::Error::new(ErrorKind::UnexpectedEof, j),
            }
        }
    }
}

struct ErrorImpl {
    code: ErrorCode,
    line: usize,
    column: usize,
}

pub(crate) enum ErrorCode {
    /// Catchall for syntax error messages
    Message(Box<str>),

    /// Some IO error occurred while serializing or deserializing.
    Io(io::Error),

    /// EOF while parsing a list.
    EofWhileParsingList,

    /// EOF while parsing an object.
    EofWhileParsingObject,

    /// EOF while parsing a string.
    EofWhileParsingString,

    /// EOF while parsing a EnCom value.
    EofWhileParsingValue,

    /// Expected this character to be a `':'`.
    ExpectedColon,

    /// Expected this character to be either a `','` or a `']'`.
    // ExpectedListCommaOrEnd,

    /// Expected this character to be either a `','` or a `'}'`.
    ExpectedObjectCommaOrEnd,

    /// Expected to parse either a `true`, `false`, or a `null`.
    ExpectedSomeIdent,

    /// Expected this character to start a EnCom value.
    ExpectedSomeValue,

    /// Expected this character to be a `"`.
    ExpectedDoubleQuote,

    /// Invalid hex escape code.
    InvalidEscape,

    /// Invalid number.
    InvalidNumber,

    /// Number is bigger than the maximum value of its type.
    NumberOutOfRange,

    /// Invalid unicode code point.
    InvalidUnicodeCodePoint,

    /// Control character found while parsing a string.
    ControlCharacterWhileParsingString,

    /// Object key is not a string.
    KeyMustBeAString,

    /// Contents of key were supposed to be a number.
    ExpectedNumericKey,

    /// Object key is a non-finite float value.
    FloatKeyMustBeFinite,

    /// Lone leading surrogate in hex escape.
    // LoneLeadingSurrogateInHexEscape,

    /// EnCom has a comma after the last value in an array or map.
    TrailingComma,

    /// EnCom has non-whitespace trailing characters after the value.
    TrailingCharacters,

    /// Unexpected end of hex escape.
    // UnexpectedEndOfHexEscape,

    /// Unexpected end of hex escape.
    UnexpectedEndOfString,

    /// Encountered nesting of EnCom maps and arrays more than 128 layers deep.
    RecursionLimitExceeded,
}

impl Error {
    #[cold]
    pub(crate) fn syntax(code: ErrorCode, line: usize, column: usize) -> Self {
        Error {
            err: Box::new(ErrorImpl { code, line, column }),
        }
    }

    // Not public API. Should be pub(crate).
    //
    // Update `eager_json` crate when this function changes.
    #[doc(hidden)]
    #[cold]
    pub fn io(error: io::Error) -> Self {
        Error {
            err: Box::new(ErrorImpl {
                code: ErrorCode::Io(error),
                line: 0,
                column: 0,
            }),
        }
    }

    #[cold]
    pub(crate) fn fix_position<F>(self, f: F) -> Self
    where
        F: FnOnce(ErrorCode) -> Error,
    {
        if self.err.line == 0 {
            f(self.err.code)
        } else {
            self
        }
    }
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorCode::Message(msg) => f.write_str(msg),
            ErrorCode::Io(err) => Display::fmt(err, f),
            ErrorCode::EofWhileParsingList => f.write_str("EOF while parsing a list"),
            ErrorCode::EofWhileParsingObject => f.write_str("EOF while parsing an object"),
            ErrorCode::EofWhileParsingString => f.write_str("EOF while parsing a string"),
            ErrorCode::EofWhileParsingValue => f.write_str("EOF while parsing a value"),
            ErrorCode::ExpectedColon => f.write_str("expected `:`"),
            // ErrorCode::ExpectedListCommaOrEnd => f.write_str("expected `,` or `]`"),
            ErrorCode::ExpectedObjectCommaOrEnd => f.write_str("expected `,` or `}`"),
            ErrorCode::ExpectedSomeIdent => f.write_str("expected ident"),
            ErrorCode::ExpectedSomeValue => f.write_str("expected value"),
            ErrorCode::ExpectedDoubleQuote => f.write_str("expected `\"`"),
            ErrorCode::InvalidEscape => f.write_str("invalid escape"),
            ErrorCode::InvalidNumber => f.write_str("invalid number"),
            ErrorCode::NumberOutOfRange => f.write_str("number out of range"),
            ErrorCode::InvalidUnicodeCodePoint => f.write_str("invalid unicode code point"),
            ErrorCode::ControlCharacterWhileParsingString => {
                f.write_str("control character (\\u0000-\\u001F) found while parsing a string")
            }
            ErrorCode::KeyMustBeAString => f.write_str("key must be a string"),
            ErrorCode::ExpectedNumericKey => {
                f.write_str("invalid value: expected key to be a number in quotes")
            }
            ErrorCode::FloatKeyMustBeFinite => {
                f.write_str("float key must be finite (got NaN or +/-inf)")
            }
            /* ErrorCode::LoneLeadingSurrogateInHexEscape => {
                f.write_str("lone leading surrogate in hex escape")
            } */
            ErrorCode::TrailingComma => f.write_str("trailing comma"),
            ErrorCode::TrailingCharacters => f.write_str("trailing characters"),
            // ErrorCode::UnexpectedEndOfHexEscape => f.write_str("unexpected end of hex escape"),
            ErrorCode::UnexpectedEndOfString => f.write_str("unexpected end of string"),
            ErrorCode::RecursionLimitExceeded => f.write_str("recursion limit exceeded"),
        }
    }
}

impl From<AtoiSimdError<'_>> for ErrorCode {
    fn from(e: AtoiSimdError) -> Self {
        match e {
            AtoiSimdError::Empty => ErrorCode::EofWhileParsingValue,
            AtoiSimdError::Size(_, _) => ErrorCode::NumberOutOfRange, //todo: new error
            AtoiSimdError::Overflow(_, _) => ErrorCode::NumberOutOfRange,
            AtoiSimdError::Invalid64(_, _, _) | AtoiSimdError::Invalid128(_, _, _) => {
                ErrorCode::InvalidNumber
            }
        }
    }
}

impl From<AtoiSimdError<'_>> for Error {
    fn from(e: AtoiSimdError<'_>) -> Self {
        Self {
            err: Box::new(ErrorImpl {
                code: e.into(),
                line: 0,
                column: 0,
            }),
        }
    }
}

impl serde::de::StdError for Error {
    #[cfg(feature = "std")]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.err.code {
            ErrorCode::Io(err) => err.source(),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&*self.err, f)
    }
}

impl Display for ErrorImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.line == 0 {
            Display::fmt(&self.code, f)
        } else {
            write!(
                f,
                "{} at line {} column {}",
                self.code, self.line, self.column
            )
        }
    }
}

// Remove two layers of verbosity from the debug representation. Humans often
// end up seeing this representation because it is what unwrap() shows.
impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error({:?}, line: {}, column: {})",
            self.err.code.to_string(),
            self.err.line,
            self.err.column
        )
    }
}

impl de::Error for Error {
    #[cold]
    fn custom<T: Display>(msg: T) -> Error {
        make_error(msg.to_string())
    }

    #[cold]
    fn invalid_type(unexp: de::Unexpected, exp: &dyn de::Expected) -> Self {
        if let de::Unexpected::Unit = unexp {
            Error::custom(format_args!("invalid type: null, expected {}", exp))
        } else {
            Error::custom(format_args!("invalid type: {}, expected {}", unexp, exp))
        }
    }
}

impl ser::Error for Error {
    #[cold]
    fn custom<T: Display>(msg: T) -> Error {
        make_error(msg.to_string())
    }
}

// Parse our own error message that looks like "{} at line {} column {}" to work
// around erased-serde round-tripping the error through de::Error::custom.
fn make_error(mut msg: String) -> Error {
    let (line, column) = parse_line_col(&mut msg).unwrap_or((0, 0));
    Error {
        err: Box::new(ErrorImpl {
            code: ErrorCode::Message(msg.into_boxed_str()),
            line,
            column,
        }),
    }
}

fn parse_line_col(msg: &mut String) -> Option<(usize, usize)> {
    let start_of_suffix = match msg.rfind(" at line ") {
        Some(index) => index,
        None => return None,
    };

    // Find start and end of line number.
    let start_of_line = start_of_suffix + " at line ".len();
    let mut end_of_line = start_of_line;
    while starts_with_digit(&msg[end_of_line..]) {
        end_of_line += 1;
    }

    if !msg[end_of_line..].starts_with(" column ") {
        return None;
    }

    // Find start and end of column number.
    let start_of_column = end_of_line + " column ".len();
    let mut end_of_column = start_of_column;
    while starts_with_digit(&msg[end_of_column..]) {
        end_of_column += 1;
    }

    if end_of_column < msg.len() {
        return None;
    }

    // Parse numbers.
    let line = match usize::from_str(&msg[start_of_line..end_of_line]) {
        Ok(line) => line,
        Err(_) => return None,
    };
    let column = match usize::from_str(&msg[start_of_column..end_of_column]) {
        Ok(column) => column,
        Err(_) => return None,
    };

    msg.truncate(start_of_suffix);
    Some((line, column))
}

fn starts_with_digit(slice: &str) -> bool {
    match slice.as_bytes().first() {
        None => false,
        Some(&byte) => byte >= b'0' && byte <= b'9',
    }
}
