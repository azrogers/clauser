use std::fmt;

use serde::{de::Visitor, Deserialize};
use zerocopy::transmute;
use zerocopy_derive::{AsBytes, FromBytes, FromZeroes};

use crate::{de::Deserializer, error::Error, tokenizer::Tokenizer};

/// A token that can be produced by a `Tokenizer`.
pub trait ConstructableToken {
    fn from_tokenizer(t: &Tokenizer, token_type: TokenType, index: usize, length: usize) -> Self;
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum TokenType {
    /**
     * Tokens should never be invalid!
     */
    Invalid,
    /**
     * A non-string bit of text.
     */
    Identifier,
    /**
     * An integer or decimal number.
     */
    Number,
    /**
     * A string wrapped in double quotes.
     */
    String,
    /**
     * The '=' symbol.
     */
    Equals,
    /**
     * The '{' symbol.
     */
    OpenBracket,
    /**
     * The '}' symbol.
     */
    CloseBracket,
    /**
     * The '>' symbol.
     */
    GreaterThan,
    /**
     * The '<' symbol.
     */
    LessThan,
    /**
     * The '>=' symbol.
     */
    GreaterThanEq,
    /**
     * The '<=' symbol.
     */
    LessThanEq,
    /**
     * The '?=' symbol.
     */
    ExistenceCheck,
    /**
     * A yes or no value.
     */
    Boolean,
    /**
     * A token in the form `YYYY.MM.DD(.HH)` with the last component, hours, being optional.`
     */
    Date,
}

#[derive(Debug)]
pub struct Token {
    pub index: usize,
    pub length: usize,
    pub token_type: TokenType,
}

impl Token {
    pub fn new(token_type: TokenType, index: usize, length: usize) -> Token {
        Token {
            index,
            length,
            token_type,
        }
    }
}

impl ConstructableToken for Token {
    fn from_tokenizer(_t: &Tokenizer, token_type: TokenType, index: usize, length: usize) -> Self {
        Token::new(token_type, index, length)
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "token type {:?} at pos {}, length {}",
            self.token_type, self.index, self.length
        )
    }
}

#[derive(Debug)]
pub struct OwnedToken {
    pub index: usize,
    pub token_type: TokenType,
    pub value: String,
}

impl ConstructableToken for OwnedToken {
    fn from_tokenizer(t: &Tokenizer, token_type: TokenType, index: usize, length: usize) -> Self {
        OwnedToken {
            index,
            token_type,
            value: t.str_for_range((index, index + length)).to_owned(),
        }
    }
}

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

#[derive(Debug, PartialEq, Clone, Copy, FromZeroes, FromBytes, AsBytes, Deserialize)]
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
