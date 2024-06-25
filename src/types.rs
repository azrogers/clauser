use std::borrow::Cow;

#[cfg(feature = "serde")]
use serde::Deserialize;

use zerocopy::transmute;
use zerocopy_derive::{AsBytes, FromBytes, FromZeroes};

use crate::token::TokenType;

/// Types that are actually differentiable purely from tokens.
#[derive(Debug, Eq, PartialEq)]
pub enum RealType {
    /// An object or an array.
    ///
    /// While *most* objects or arrays can be differentiated, a string of
    /// `{}` could be either one - so we have to admit we don't know.
    ObjectOrArray,
    /// An integer or decimal number.
    Number,
    /// A boolean "yes" or "no" value.
    Boolean,
    /// A string.
    String,
    /// An identifier.
    Identifier,
    /// A date.
    Date,
}

/// The possible kinds of collections in a Clausewitz file.
#[derive(Debug, Eq, PartialEq)]
pub enum CollectionType {
    /// A key-value map.
    Object,
    /// A sequence of values.
    Array,
}

impl RealType {
    /// Creates a [RealType] from a [TokenType], if possible.
    ///
    /// Not all [TokenType] values have equivalent [RealType] values.
    pub fn from_token_type(t: &TokenType) -> Option<RealType> {
        match *t {
            TokenType::Boolean => Some(RealType::Boolean),
            TokenType::Number => Some(RealType::Number),
            TokenType::Identifier => Some(RealType::Identifier),
            TokenType::String => Some(RealType::String),
            TokenType::OpenBracket => Some(RealType::ObjectOrArray),
            TokenType::Date => Some(RealType::Date),
            _ => None,
        }
    }
}

/// A value specifying years, months, days, and possibly hours.
#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, FromZeroes, FromBytes, AsBytes, Hash,
)]
#[repr(C)]
#[cfg_attr(feature = "serde", derive(Deserialize), serde(from = "u128"))]
pub struct Date {
    /// The number of years in this date.
    pub years: u32,
    /// The number of months in this date.
    pub months: u32,
    /// The number of days in this date.
    pub days: u32,
    /// The number of hours in this date.
    /// If the source value had no hours component, this will be 0.
    pub hours: u32,
}

impl Date {
    /// Creates a new [Date] from the given years, months, days, and hours values.
    pub fn new(years: u32, months: u32, days: u32, hours: u32) -> Date {
        Date {
            years,
            months,
            days,
            hours,
        }
    }
}

impl From<[u32; 4]> for Date {
    fn from(value: [u32; 4]) -> Self {
        Date {
            years: value[0],
            months: value[1],
            days: value[2],
            hours: value[3],
        }
    }
}

impl From<Date> for [u32; 4] {
    fn from(value: Date) -> Self {
        [value.years, value.months, value.days, value.hours]
    }
}

impl From<u128> for Date {
    fn from(value: u128) -> Self {
        let parts: [u32; 4] = transmute!(value);

        parts.into()
    }
}

impl From<Date> for u128 {
    fn from(value: Date) -> Self {
        let indices: [u32; 4] = [value.years, value.months, value.days, value.hours];
        transmute!(indices)
    }
}

impl From<Date> for (u32, u32, u32, u32) {
    fn from(value: Date) -> Self {
        (value.years, value.months, value.days, value.hours)
    }
}

/// Represents the key of an object in a [Value](`crate::value::Value`).
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum ObjectKey<'src> {
    /// An Identifier key
    Identifier(Cow<'src, str>),
    /// A Date key
    Date(Date),
}

impl<'src> From<Date> for ObjectKey<'src> {
    fn from(value: Date) -> Self {
        ObjectKey::Date(value)
    }
}

impl<'src> From<&'src str> for ObjectKey<'src> {
    fn from(value: &'src str) -> Self {
        ObjectKey::Identifier(value.into())
    }
}

impl<'src> From<String> for ObjectKey<'src> {
    fn from(value: String) -> Self {
        ObjectKey::Identifier(value.into())
    }
}
