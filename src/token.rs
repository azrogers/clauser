use std::fmt;

use crate::tokenizer::Tokenizer;

/// A token that can be produced by a `Tokenizer`.
pub trait ConstructableToken {
    /// Creates a token type from the given information.
    fn from_tokenizer(t: &Tokenizer, token_type: TokenType, index: usize, length: usize) -> Self;
}

/// A single token from a Clausewitz source file.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum TokenType {
    /// A non-string bit of text.
    Identifier,
    /// An integer or decimal number.
    Number,
    /// A string wrapped in double quotes.
    String,
    /// The `=` symbol.
    Equals,
    /// The `{` symbol.
    OpenBracket,
    /// The `}` symbol.
    CloseBracket,
    /// The `>` symbol.
    GreaterThan,
    /// The `<` symbol.
    LessThan,
    /// The `>=` symbol.
    GreaterThanEq,
    /// The `<=` symbol.
    LessThanEq,
    /// The `?=` symbol.
    ExistenceCheck,
    /// A yes or no value.
    Boolean,
    /// A token in the form `\d+.\d+.\d+(.\d+)?`.`
    Date,
}

/// A single [Token] obtained from a [Tokenizer].
#[derive(Debug)]
pub struct Token {
    /// The index of this token in source text.
    pub index: usize,
    /// The length of this token in chars.
    pub length: usize,
    /// The type of this token.
    pub token_type: TokenType,
}

impl Token {
    /// Creates a new token with the given type and bounds.
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

/// A [Token] that owns its value.
#[derive(Debug)]
pub struct OwnedToken {
    /// The index of this token in the source text.
    pub index: usize,
    /// The type of this token.
    pub token_type: TokenType,
    /// The value of this token.
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
