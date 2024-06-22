use std::collections::HashMap;

use crate::token::TokenType;

/// Types that are actually differentiable purely from tokens.
#[derive(Debug, Eq, PartialEq)]
pub enum RealType {
    ObjectOrArray,
    Number,
    Boolean,
    String,
    Identifier,
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
            _ => None,
        }
    }
}
