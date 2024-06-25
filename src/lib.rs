//! # clauser
//!
//! clauser is a library for working with configuration, script, and data files from the Clausewitz engine
//! used by Paradox Interactive for their grand strategy games.
//!
//! It contains a number of components:
//! - [Deserializer](`de::Deserializer`) is a [serde] deserializer that can deserialize
//!   Clausewitz files into Rust data structures.
//! - [Value](`value::Value`) allows deserializing a Clausewitz file into a tree of values,
//!   for situations where the schema of the data isn't known beforehand.
//! - [Tokenizer](`tokenizer::Tokenizer`) turns a Clausewitz file into a series of tokens.
//! - [Reader](`reader::Reader`) is a wrapper around [Tokenizer](`tokenizer::Tokenizer`) that enables
//!   low-level parsing operations on a Clausewitz source file.

#![feature(let_chains)]
#![feature(doc_cfg)]

/// [serde] deserializer for Clausewitz files.
#[cfg(feature = "serde")]
#[doc(cfg(feature = "serde"))]
pub mod de;

/// Low-level parser for Clausewitz files.
pub mod reader;
/// Tokens obtained from a source file.
pub mod token;
/// A Tokenizer for parsing Clausewitz files.
pub mod tokenizer;
/// Various types required to represent Clausewitz data.
pub mod types;
/// Deserialization for Clausewitz files without a known schema.
pub mod value;

mod schema;
mod util;

#[cfg(feature = "macros")]
extern crate clauser_macros;
#[cfg(feature = "macros")]
#[doc(cfg(feature = "macros"))]
pub use clauser_macros::duplicate_keys;

/// Library error type.
pub mod error {
    pub use super::util::error::{Error, ErrorType};
}

#[doc(hidden)]
pub mod static_assertions {
    pub use static_assertions::*;
}
