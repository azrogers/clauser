use std::fmt;

use crate::tokenizer::Tokenizer;

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
