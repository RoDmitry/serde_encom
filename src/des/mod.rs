//! Deserialize EnCom data to a Rust data structure.

mod access;
mod deserializer;
mod iter;
pub(crate) mod parser_number;
mod read;
mod stream_deserializer;
mod wrapper;

// use self::wrapper::InitDeserializer;
use crate::error::{Error, Result};
#[cfg(feature = "float_roundtrip")]
use crate::lexical;
use crate::value::Number;
#[cfg(feature = "float_roundtrip")]
use core::iter;
use core::result;
use core::str::FromStr;
use serde::de;

#[cfg(feature = "arbitrary_precision")]
use crate::number::NumberDeserializer;

pub use self::deserializer::Deserializer;
#[cfg(feature = "std")]
pub use self::read::IoRead;
pub use self::read::{Read, SliceRead, StrRead};
pub use self::wrapper::InitDeserializer;

impl FromStr for Number {
    type Err = Error;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        Deserializer::from_str(s)
            .parse_any_signed_number()
            .map(Into::into)
    }
}

fn from_trait<'de, R, T>(read: R) -> Result<T>
where
    R: Read<'de>,
    T: de::Deserialize<'de>,
{
    let mut de = Deserializer::new(read);
    let value = de::Deserialize::deserialize(InitDeserializer { des: &mut de })?;

    // Make sure the whole stream has been consumed.
    de.end()?;
    Ok(value)
}

/// Deserialize an instance of type `T` from an IO stream of EnCom.
///
/// The content of the IO stream is deserialized directly from the stream
/// without being buffered in memory by serde_encom.
///
/// When reading from a source against which short reads are not efficient, such
/// as a [`File`], you will want to apply your own buffering because serde_encom
/// will not buffer the input. See [`std::io::BufReader`].
///
/// It is expected that the input stream ends after the deserialized object.
/// If the stream does not end, such as in the case of a persistent socket connection,
/// this function will not return. It is possible instead to deserialize from a prefix of an input
/// stream without looking for EOF by managing your own [`Deserializer`].
///
/// Note that counter to intuition, this function is usually slower than
/// reading a file completely into memory and then applying [`from_str`]
/// or [`from_slice`] on it. See [issue #160].
///
/// [`File`]: https://doc.rust-lang.org/std/fs/struct.File.html
/// [`std::io::BufReader`]: https://doc.rust-lang.org/std/io/struct.BufReader.html
/// [`from_str`]: ./fn.from_str.html
/// [`from_slice`]: ./fn.from_slice.html
/// [issue #160]: https://github.com/serde-rs/json/issues/160
///
/// # Example
///
/// Reading the contents of a file.
///
/// ```
/// use serde::Deserialize;
///
/// use std::error::Error;
/// use std::fs::File;
/// use std::io::BufReader;
/// use std::path::Path;
///
/// #[derive(Deserialize, Debug)]
/// struct User {
///     fingerprint: String,
///     location: String,
/// }
///
/// fn read_user_from_file<P: AsRef<Path>>(path: P) -> Result<User, Box<dyn Error>> {
///     // Open the file in read-only mode with buffer.
///     let file = File::open(path)?;
///     let reader = BufReader::new(file);
///
///     // Read the EnCom contents of the file as an instance of `User`.
///     let u = serde_encom::from_reader(reader)?;
///
///     // Return the `User`.
///     Ok(u)
/// }
///
/// fn main() {
/// # }
/// # fn fake_main() {
///     let u = read_user_from_file("test.encom").unwrap();
///     println!("{:#?}", u);
/// }
/// ```
///
/// Reading from a persistent socket connection.
///
/// ```
/// use serde::Deserialize;
///
/// use std::error::Error;
/// use std::net::{TcpListener, TcpStream};
///
/// #[derive(Deserialize, Debug)]
/// struct User {
///     fingerprint: String,
///     location: String,
/// }
///
/// fn read_user_from_stream(tcp_stream: TcpStream) -> Result<User, Box<dyn Error>> {
///     let mut de = serde_encom::Deserializer::from_reader(tcp_stream);
///     let u = User::deserialize(&mut de)?;
///
///     Ok(u)
/// }
///
/// fn main() {
/// # }
/// # fn fake_main() {
///     let listener = TcpListener::bind("127.0.0.1:4000").unwrap();
///
///     for stream in listener.incoming() {
///         println!("{:#?}", read_user_from_stream(stream.unwrap()));
///     }
/// }
/// ```
///
/// # Errors
///
/// This conversion can fail if the structure of the input does not match the
/// structure expected by `T`, for example if `T` is a struct type but the input
/// contains something other than an EnCom map. It can also fail if the structure
/// is correct but `T`'s implementation of `Deserialize` decides that something
/// is wrong with the data, for example required struct fields are missing from
/// the EnCom map or some number is too big to fit in the expected primitive
/// type.
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub fn from_reader<R, T>(rdr: R) -> Result<T>
where
    R: crate::io::Read,
    T: de::DeserializeOwned,
{
    from_trait(read::IoRead::new(rdr))
}

/// Deserialize an instance of type `T` from bytes of EnCom text.
///
/// # Example
///
/// ```
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug)]
/// struct User {
///     fingerprint: String,
///     location: String,
/// }
///
/// fn main() {
///     // The type of `j` is `&[u8]`
///     let j = b"
///             fingerprint:18=0xF9BA143B95FF6D82
///             location:14=Menlo Park, CA
///         ";
///
///     let u: User = serde_encom::from_slice(j).unwrap();
///     println!("{:#?}", u);
/// }
/// ```
///
/// # Errors
///
/// This conversion can fail if the structure of the input does not match the
/// structure expected by `T`, for example if `T` is a struct type but the input
/// contains something other than an EnCom map. It can also fail if the structure
/// is correct but `T`'s implementation of `Deserialize` decides that something
/// is wrong with the data, for example required struct fields are missing from
/// the EnCom map or some number is too big to fit in the expected primitive
/// type.
pub fn from_slice<'a, T>(v: &'a [u8]) -> Result<T>
where
    T: de::Deserialize<'a>,
{
    from_trait(read::SliceRead::new(v))
}

/// Deserialize an instance of type `T` from a string of EnCom text.
///
/// # Example
///
/// ```
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug)]
/// struct User {
///     fingerprint: String,
///     location: String,
/// }
///
/// fn main() {
///     // The type of `j` is `&str`
///     let j = "
///             fingerprint:18=0xF9BA143B95FF6D82
///             location:14=Menlo Park, CA
///         ";
///
///     let u: User = serde_encom::from_str(j).unwrap();
///     println!("{:#?}", u);
/// }
/// ```
///
/// # Errors
///
/// This conversion can fail if the structure of the input does not match the
/// structure expected by `T`, for example if `T` is a struct type but the input
/// contains something other than an EnCom map. It can also fail if the structure
/// is correct but `T`'s implementation of `Deserialize` decides that something
/// is wrong with the data, for example required struct fields are missing from
/// the EnCom map or some number is too big to fit in the expected primitive
/// type.
pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: de::Deserialize<'a>,
{
    from_trait(read::StrRead::new(s))
}
