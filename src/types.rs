use std::borrow::Cow;

use serde::Deserialize;
use zerocopy::transmute;
use zerocopy_derive::{AsBytes, FromBytes, FromZeroes};

use crate::token::TokenType;

/// Types that are actually differentiable purely from tokens.
#[derive(Debug, Eq, PartialEq)]
pub enum RealType {
    ObjectOrArray,
    Number,
    Boolean,
    String,
    Identifier,
    Date,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CollectionType {
    /// A key-value map.
    Object,
    /// A sequence of values.
    Array,
}

impl RealType {
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

#[derive(
    Debug, PartialEq, PartialOrd, Clone, Copy, FromZeroes, FromBytes, AsBytes, Deserialize,
)]
#[repr(C)]
#[serde(from = "u128")]
pub struct Date {
    pub years: u32,
    pub months: u32,
    pub days: u32,
    pub hours: u32,
}

impl Date {
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

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum ObjectKey<'src> {
    Identifier(Cow<'src, str>),
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
