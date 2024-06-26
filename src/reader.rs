use crate::{
    token::{Token, TokenType},
    tokenizer::Tokenizer,
    types::{CollectionType, Date, ObjectKey, RealType},
    util::error::{Error, ErrorContext, ErrorContextProvider, ErrorType, ParseResult},
};
use std::str::FromStr;

/// A pair of ([ObjectKey], [RealType]) values representing the next property read
/// from an object.
pub type PropertyInfo<'a> = (ObjectKey<'a>, RealType);

/// [Reader] offers a wrapper around a [Tokenizer] that allows low-level parsing
/// operations on a Clausewitz source file.
///
/// [Reader] is useful for situations where a file can't be properly parsed by either
/// [Deserializer](`crate::de::Deserializer`) or [Value](`crate::value::Value`).
pub struct Reader<'a> {
    tokenizer: Tokenizer<'a>,
    current_depth: usize,
}

impl<'a> Reader<'a> {
    /// Creates a new Reader from the given text.
    pub fn new(text: &'a str) -> Reader {
        Reader {
            tokenizer: Tokenizer::new(text),
            current_depth: 0,
        }
    }

    /// Returns the current position of the underlying tokenizer in the token stream
    pub fn current_position(&self) -> usize {
        self.tokenizer.position
    }

    // Increments the current depth of the reader.
    fn increment_depth(&mut self) {
        self.current_depth = self.current_depth + 1
    }

    /// Decrements the current depth of the reader.
    fn decrement_depth(&mut self) -> ParseResult<Token> {
        if self.current_depth <= 0 {
            return Err(self.parse_error(
                ErrorType::DepthMismatchError,
                "attempted to decrement depth but already at top-level",
            ));
        }

        self.current_depth = self.current_depth - 1;
        Ok(None)
    }

    /// Returns true if this reader is at the root level (not in a collection)
    pub fn is_root_level(&self) -> bool {
        self.current_depth == 0
    }

    /// Obtains the next token from the tokenizer, erroring if the token type doesn't match the expected types.
    pub fn expect_tokens(&mut self, expected_types: &[TokenType]) -> Result<Token, Error> {
        match self.tokenizer.next() {
            Ok(opt) => match opt {
                Some(token) => match expected_types.contains(&token.token_type) {
                    true => Ok(token),
                    false => Err(self.unexpected_token_error(&token, expected_types)),
                },
                None => Err(self.parse_error(
                    ErrorType::UnexpectedTokenError,
                    format!("unexpected EOF, expected {:?}", expected_types),
                )),
            },
            Err(e) => Err(e),
        }
    }

    /// Obtains the next token from the tokenizer, erroring if the token type doesn't match the expected type.
    pub fn expect_token(&mut self, expected_type: TokenType) -> Result<Token, Error> {
        self.expect_tokens(&[expected_type])
    }

    /// Tells the reader to begin reading an object or array.
    pub fn begin_collection(&mut self) -> ParseResult<()> {
        self.expect_token(TokenType::OpenBracket)?;
        self.increment_depth();
        Ok(None)
    }

    /// Tells the reader to stop reading an object or array.
    pub fn end_collection(&mut self) -> ParseResult<()> {
        self.expect_token(TokenType::CloseBracket)?;
        self.decrement_depth()?;
        Ok(None)
    }

    /// Reads the next property name and type, if available.
    pub fn next_property(&mut self) -> ParseResult<PropertyInfo<'a>> {
        let result = self.tokenizer.peek()?;
        if result.is_none() {
            if self.current_depth == 0 {
                // EOF is a valid end for the root object
                return Ok(None);
            }

            return Err(self.parse_error(
                ErrorType::UnexpectedTokenError,
                String::from("unexpected EOF while reading next property"),
            ));
        }

        let token = result.unwrap();
        if token.token_type == TokenType::CloseBracket {
            if self.current_depth == 0 {
                return Err(self.unexpected_token_error(&token, &[TokenType::CloseBracket]));
            }

            // we've reached the end of the object, we're done
            return Ok(None);
        }

        self.tokenizer.next()?;
        let key: ObjectKey<'a> = match token.token_type {
            TokenType::Identifier => Ok(self.tokenizer.str_for_token(&token).into()),
            TokenType::Date => Ok(self.tokenizer.date_for_token(&token)?.into()),
            _ => {
                return Err(
                    self.unexpected_token_error(&token, &[TokenType::Identifier, TokenType::Date])
                )
            }
        }?;

        // property_name = ...
        self.expect_token(TokenType::Equals)?;

        let real_type = self.peek_next_type()?.ok_or(self.parse_error(
            ErrorType::UnexpectedTokenError,
            String::from("expected value, got EOF"),
        ))?;

        Ok(Some((key, real_type)))
    }

    /// Reads a string from the token stream, if available.
    pub fn read_string(&mut self) -> Result<&'a str, Error> {
        let token = self.expect_token(TokenType::String)?;
        Ok(self.tokenizer.str_for_token(&token))
    }

    /// Peeks the next string that would be read from the token stream.
    pub fn peek_expected_string(&mut self) -> Result<&'a str, Error> {
        let pos = self.tokenizer.position;
        let str = self.read_string()?;
        self.tokenizer.position = pos;
        Ok(str)
    }

    /// Reads an identifier from the token stream, if available.
    pub fn read_identifier(&mut self) -> Result<&'a str, Error> {
        let token = self.expect_token(TokenType::Identifier)?;
        Ok(self.tokenizer.str_for_token(&token))
    }

    /// Reads a string, identifier, or empty from the input stream, if any.
    pub fn read_stringlike(&mut self) -> Result<&'a str, Error> {
        let in_property = self.tokenizer.last_char_was('=');
        let next_token = self.tokenizer.peek()?;

        if next_token.is_none() {
            // empty string
            return match in_property {
                true => Ok(""),
                false => Err(self.parse_error(
                    ErrorType::UnexpectedTokenError,
                    "expected string or identifier, got EOF",
                )),
            };
        }

        let next_token = next_token.unwrap();

        // new line before the next token, also an empty string
        if in_property && self.new_line_between(self.tokenizer.position, next_token.index) {
            return Ok("");
        }

        match &next_token.token_type {
            TokenType::Identifier => self.read_identifier(),
            TokenType::String => self.read_string(),
            t => {
                // end of collection, it's an empty string
                if *t == TokenType::CloseBracket && self.current_depth > 0 {
                    return Ok("");
                }

                Err(self.parse_error(
                    ErrorType::UnexpectedTokenError,
                    format!(
                        "expected identifier, string, or empty, got {:?}",
                        next_token
                    ),
                ))
            }
        }
    }

    /// Reads a [Date] from the token stream, if available.
    pub fn read_date(&mut self) -> Result<Date, Error> {
        let token = self.expect_token(TokenType::Date)?;
        self.tokenizer.date_for_token(&token)
    }

    /// Reads a boolean from the token stream, if available.
    pub fn read_boolean(&mut self) -> Result<bool, Error> {
        let token = self.expect_token(TokenType::Boolean)?;
        let str = self.tokenizer.str_for_token(&token);
        Ok(str.starts_with('y'))
    }

    /// Reads a number from the token stream, if available.
    pub fn read_number<T: FromStr>(&mut self) -> Result<T, Error> {
        let token = self.expect_token(TokenType::Number)?;
        let str = self.tokenizer.str_for_token(&token);
        self.parse_number(str)
    }

    /// Read a number from the token stream, returning its string value.
    pub fn read_number_as_str(&mut self) -> Result<&'a str, Error> {
        let token = self.expect_token(TokenType::Number)?;
        Ok(self.tokenizer.str_for_token(&token))
    }

    /// Parses a number from a string.
    pub fn parse_number<T: FromStr>(&self, str: &str) -> Result<T, Error> {
        str.parse::<T>().map_err(|_| {
            self.tokenizer.parse_error_pos(
                ErrorType::InvalidNumberError,
                self.tokenizer.position - str.len(),
                format!("failed to parse number from token '{}'", str),
            )
        })
    }

    /// Reads the next array value's type, if available.
    pub fn next_array_value(&mut self) -> ParseResult<RealType> {
        let result = self.tokenizer.peek()?;
        if result.is_none() {
            // end of the array
            return Ok(None);
        }

        let token = result.unwrap();

        if token.token_type == TokenType::CloseBracket {
            return Ok(None);
        }

        let real_type =
            RealType::from_token_type(&token.token_type).ok_or(self.parse_error_token(
                &token,
                ErrorType::UnexpectedTokenError,
                format!("unexpected token type {:?} in array", token.token_type),
            ))?;

        Ok(Some(real_type))
    }

    /// Peeks the type of the next value in the token stream, if any.
    pub fn peek_next_type(&mut self) -> ParseResult<RealType> {
        // peek at the next token to get its type
        let token = self.tokenizer.peek()?;

        match token {
            Some(token) => Ok(Some(RealType::from_token_type(&token.token_type).ok_or(
                self.parse_error_token(
                    &token,
                    ErrorType::UnexpectedTokenError,
                    format!("unexpected token type {:?} in value", token.token_type),
                ),
            )?)),
            None => Ok(None),
        }
    }

    /// Peeks the type of the next value in the token stream like [Reader::peek_next_type],
    /// but this method will error if EOF is encountered.
    pub fn peek_next_type_expect(&mut self) -> Result<RealType, Error> {
        match self.peek_next_type()? {
            Some(token_type) => Ok(token_type),
            None => Err(self.parse_error(
                ErrorType::UnexpectedTokenError,
                "expected next token, found EOF",
            )),
        }
    }

    /// Peeks ahead to see if this collection (array or object) has finished
    pub fn is_collection_ended(&mut self) -> Result<bool, Error> {
        match self.tokenizer.peek()? {
            None => {
                if self.current_depth == 0 {
                    return Ok(true);
                }

                return Err(self.parse_error(
                    ErrorType::UnexpectedTokenError,
                    "expected value or close bracket, found EOF",
                ));
            }
            Some(token) => match token.token_type {
                TokenType::CloseBracket => Ok(true),
                _ => Ok(false),
            },
        }
    }

    /// Attempt to discern between an array or map by peeking ahead.
    ///
    /// Empty collections (`{}`) will return Array.
    pub fn try_discern_array_or_map(&mut self) -> ParseResult<CollectionType> {
        let initial_pos = self.tokenizer.position;

        let token = self.tokenizer.next()?;
        if token.is_none() {
            self.tokenizer.position = initial_pos;
            return Ok(None);
        }

        let token = token.unwrap();

        if token.token_type != TokenType::OpenBracket {
            self.tokenizer.position = initial_pos;
            return Err(self.parse_error_token(
                &token,
                ErrorType::UnexpectedTokenError,
                format!("expected open bracket, found {:?}", token.token_type),
            ));
        }

        let next = self.tokenizer.next()?;
        if next.is_none() {
            self.tokenizer.position = initial_pos;
            return Ok(None);
        }

        let next = next.unwrap();
        let collection_type = match next.token_type {
            TokenType::CloseBracket => Some(CollectionType::Array),
            TokenType::Identifier | TokenType::Date => {
                // if it's an object, there will be an equals
                let next = self.tokenizer.next()?;
                if let Some(next) = next {
                    match next.token_type {
                        TokenType::Equals => Some(CollectionType::Object),
                        _ => Some(CollectionType::Array),
                    }
                } else {
                    None
                }
            }
            _ => Some(CollectionType::Array),
        };

        self.tokenizer.position = initial_pos;

        Ok(collection_type)
    }

    /// Checks if this property might not have a value.
    pub fn is_next_value_empty(&mut self) -> Result<bool, Error> {
        let next_token = self.tokenizer.peek()?;

        Ok(match next_token {
            Some(next_token) => match next_token.token_type {
                // the next token is an identifier, meaning there's no value, just the next property
                TokenType::Identifier => true,
                // the next token is the end of the collection, so there's no value
                TokenType::CloseBracket => true,
                _ => false,
            },
            // EOF means empty, right?
            None => true,
        })
    }

    /// Creates a new [Error] using the current position of the tokenizer.
    pub fn parse_error(&self, error_type: ErrorType, message: impl ToString) -> Error {
        self.tokenizer.parse_error(error_type, message)
    }

    /// Creates a new [Error] using the position of the given token.
    pub fn parse_error_token(
        &self,
        token: &Token,
        error_type: ErrorType,
        message: impl ToString,
    ) -> Error {
        self.tokenizer.parse_error_token(token, error_type, message)
    }

    /// Checks if there's a new line between the exclusive range `[start, end]``
    fn new_line_between(&self, start: usize, end: usize) -> bool {
        self.tokenizer.find_end_of_line(start) < end
    }

    /// Creates a new [Error] for an unexpected token error.
    pub fn unexpected_token_error(&self, token: &Token, expected_type: &[TokenType]) -> Error {
        self.parse_error_token(
            token,
            ErrorType::UnexpectedTokenError,
            format!(
                "unexpected token type {:?}, expected one of {:?}",
                token.token_type, expected_type
            ),
        )
    }
}

impl<'a> ErrorContextProvider for Reader<'a> {
    fn get_line_context(&self, position: usize, max_lines: usize) -> Option<ErrorContext> {
        self.tokenizer.get_line_context(position, max_lines)
    }
}
