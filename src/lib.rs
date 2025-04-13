#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
// Ignored clippy lints
#![allow(
    clippy::explicit_auto_deref,
    clippy::manual_range_contains,
    clippy::match_like_matches_macro,
    clippy::module_inception,
    clippy::needless_doctest_main,
    clippy::should_implement_trait
)]

extern crate alloc;

mod des;
mod error;
mod features_check;
mod io;
mod macros;
mod ser;
mod value;

pub use crate::des::*;
pub use crate::error::*;
pub use crate::ser::*;
pub use crate::value::*;
