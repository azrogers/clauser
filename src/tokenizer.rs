use std::marker::PhantomData;

use crate::{
    parse_error::{
        ParseError, ParseErrorContext, ParseErrorContextProvider, ParseErrorType, ParseResult,
    },
    text_helpers::CharHelper,
    token::{ConstructableToken, OwnedToken, Token, TokenType},
};

const COMMENT_CHAR: char = '#';
const NEW_LINE: char = '\n';

pub struct Tokenizer<'a> {
    pub position: usize,
    chars: Vec<char>,
    text: &'a str,
}

impl<'a> Tokenizer<'a> {
    /// Creates a new tokenizer from the input text.
    pub fn new(text: &'a str) -> Tokenizer {
        Tokenizer {
            position: 0,
            chars: text.chars().collect(),
            text,
        }
    }

    /// Parses every token in the input text and returns them in a vector.
    pub fn parse_all(text: &str) -> Result<Vec<OwnedToken>, ParseError> {
        let mut tokenizer = Tokenizer::new(text);
        let tokens: Result<Vec<OwnedToken>, ParseError> = tokenizer.iter_owned().collect();
        Ok(tokens?)
    }

    /// Checks if this tokenizer has hit the end of the character stream.
    pub fn is_done(&self) -> bool {
        self.position >= self.chars.len()
    }

    /// Checks if the given character is the next one in the character stream.
    ///
    /// This checks the next character, not the next token. It doesn't skip whitespace or comments.
    pub fn is_next_char(&self, c: char) -> bool {
        match self.position >= self.chars.len() - 1 {
            true => false,
            false => self.chars[self.position + 1] == c,
        }
    }

    /// Creates a new iterator from this tokenizer.
    pub fn iter_generic<T: ConstructableToken>(&'a mut self) -> TokenIterator<T> {
        TokenIterator::new(self)
    }

    /// Creates a new iterator that returns `Token` objects.
    pub fn iter(&'a mut self) -> TokenIterator<Token> {
        self.iter_generic()
    }

    /// Creates a new iterator that returns `OwnedToken` objects.
    pub fn iter_owned(&'a mut self) -> TokenIterator<OwnedToken> {
        self.iter_generic()
    }

    /// Obtains the next `Token` from the character stream, advancing the internal position.
    pub fn next(&mut self) -> ParseResult<Token> {
        self.next_generic()
    }

    /// Obtains the next `OwnedToken` from the character stream, advancing the internal position.
    pub fn next_owned(&mut self) -> ParseResult<OwnedToken> {
        self.next_generic()
    }

    /// Obtains the next ConstructableToken from the character stream, advancing the internal position.
    fn next_generic<T: ConstructableToken>(&mut self) -> ParseResult<T> {
        if self.is_done() {
            return Ok(None);
        }

        let mut c: char = self.skip_comments()?;
        // skip whitespace
        while !self.is_done() && char::is_whitespace(c) {
            self.position = self.position + 1;
            c = self.skip_comments()?;
        }

        if self.is_done() {
            return Ok(None);
        }

        match c {
            '=' => Ok(Some(self.new_token_incr(TokenType::Equals, 1))),
            ':' => Ok(Some(self.new_token_incr(TokenType::Colon, 1))),
            '{' => Ok(Some(self.new_token_incr(TokenType::OpenBracket, 1))),
            '}' => Ok(Some(self.new_token_incr(TokenType::CloseBracket, 1))),
            '>' => Ok(Some(match self.is_next_char('=') {
                true => self.new_token_incr(TokenType::GreaterThanEq, 2),
                false => self.new_token_incr(TokenType::GreaterThan, 1),
            })),
            '<' => Ok(Some(match self.is_next_char('=') {
                true => self.new_token_incr(TokenType::LessThanEq, 2),
                false => self.new_token_incr(TokenType::LessThan, 1),
            })),
            '?' => match self.is_next_char('=') {
                true => Ok(Some(self.new_token_incr(TokenType::ExistenceCheck, 2))),
                false => Err(self.parse_error(ParseErrorType::TokenizerError, "unexpected char ?")),
            },
            c if (c == '-' || char::is_digit(c, 10)) => {
                // number handling

                let mut num_digits = match c {
                    '-' => 0,
                    _ => 1,
                };

                let start_pos = self.position;
                let mut found_decimal_place: usize = 0;
                let mut has_found_decimal_place: bool = false;

                if !self.is_done() {
                    self.position = self.position + 1;
                }

                while !self.is_done() {
                    let num_c = self.chars[self.position];
                    if num_c == '.' {
                        // 0.05.0, -.5, and .05 are all considered invalid numbers here
                        if has_found_decimal_place || num_digits < 1 {
                            return Err(self
                                .parse_error(ParseErrorType::TokenizerError, "unexpected char ."));
                        }

                        found_decimal_place = self.position;
                        has_found_decimal_place = true;
                    } else if char::is_digit(num_c, 10) {
                        num_digits = num_digits + 1;
                    } else {
                        break;
                    }

                    self.position = self.position + 1;
                }

                // a bare - isn't allowed, and neither is 15. as a number
                if num_digits < 1
                    || (has_found_decimal_place && (self.position - found_decimal_place) < 2)
                {
                    return Err(self.parse_error_pos(
                        ParseErrorType::TokenizerError,
                        self.position - 1,
                        "unexpected end of number",
                    ));
                }

                let token = self.new_token(TokenType::Number, start_pos, self.position - start_pos);
                Ok(Some(token))
            }
            '"' => {
                let start_pos = self.position;

                loop {
                    self.position = self.position + 1;
                    if self.is_done() || self.chars[self.position] == '"' {
                        break;
                    }
                }

                match self.is_done() {
                    true => Err(self.parse_error(
                        ParseErrorType::TokenizerError,
                        "unexpected EOF while reading string",
                    )),
                    false => {
                        let length = self.position - start_pos - 1;
                        self.position = self.position + 1;
                        Ok(Some(self.new_token(
                            TokenType::String,
                            start_pos + 1,
                            length,
                        )))
                    }
                }
            }
            c if c == '_' || c.is_alphanumeric() => {
                let start_pos = self.position;
                loop {
                    self.position = self.position + 1;
                    if self.is_done()
                        || (self.chars[self.position] != '_'
                            && !char::is_alphanumeric(self.chars[self.position]))
                    {
                        break;
                    }
                }

                let length = self.position - start_pos;
                let token = match (length == 3
                    && self.chars[start_pos] == 'y'
                    && self.chars[start_pos + 1] == 'e'
                    && self.chars[start_pos + 2] == 's')
                    || (length == 2
                        && self.chars[start_pos] == 'n'
                        && self.chars[start_pos + 1] == 'o')
                {
                    true => self.new_token(TokenType::Boolean, start_pos, length),
                    false => self.new_token(TokenType::Identifier, start_pos, length),
                };

                Ok(Some(token))
            }
            _ => ParseResult::Err(self.parse_error(
                ParseErrorType::TokenizerError,
                format!("unexpected character {} in input", c),
            )),
        }
    }

    /// Obtains the next token from the character stream without changing the internal position.
    pub fn peek(&mut self) -> ParseResult<Token> {
        let pos = self.position;
        let result = self.next();
        self.position = pos;
        result
    }

    /// Obtains the next two tokens from the character stream without changing the internal position.
    pub fn peek_next_two(&mut self) -> Result<(Option<Token>, Option<Token>), ParseError> {
        let pos = self.position;
        let result = (self.next()?, self.next()?);
        self.position = pos;
        Ok(result)
    }

    /// Returns a borrowed string slice of the token's contents.
    pub fn str_for_token(&self, t: &Token) -> &'a str {
        let end = t.index + t.length;
        return &self.text[t.index..end];
    }

    /// Returns a borrowed string slice of the contents of `range`.
    ///
    /// `range` is a tuple of (start_index, end_index).
    pub fn str_for_range(&self, range: (usize, usize)) -> &'a str {
        let (start, end) = range;
        return &self.text[start..end];
    }

    /// Returns the index of the end of the line that position is on
    /// (either a new line character or the last character of `text`)
    pub fn find_end_of_line(&self, position: usize) -> usize {
        let helper = CharHelper(&self.chars);
        helper.find_line_end(position)
    }

    /// Creates a new parse error using the given position.
    pub fn parse_error_pos(
        &'a self,
        error_type: ParseErrorType,
        position: usize,
        message: impl ToString,
    ) -> ParseError {
        // clamp position to length to avoid panicking
        let position = usize::min(position, self.chars.len() - 1);
        ParseError::new(Some(self), error_type, position, message)
    }

    /// Creates a new parse error using the current position in the tokenizer.
    pub fn parse_error(&'a self, error_type: ParseErrorType, message: impl ToString) -> ParseError {
        self.parse_error_pos(error_type, self.position, message)
    }

    /// Creates a new parse error using the position of the given token
    pub fn parse_error_token(
        &'a self,
        t: &Token,
        error_type: ParseErrorType,
        message: impl ToString,
    ) -> ParseError {
        self.parse_error_pos(error_type, t.index, message)
    }

    fn skip_comments(&mut self) -> Result<char, ParseError> {
        if self.is_done() {
            return Ok(*self.chars.last().unwrap_or(&'\0'));
        }
        let mut c = self.chars[self.position];
        if c == COMMENT_CHAR {
            while self.position < self.chars.len() - 1 && c != NEW_LINE {
                self.position = self.position + 1;
                c = self.chars[self.position];
            }
        }
        Ok(c)
    }

    fn new_token<T: ConstructableToken>(
        &self,
        token_type: TokenType,
        index: usize,
        length: usize,
    ) -> T {
        T::from_tokenizer(self, token_type, index, length)
    }

    fn new_token_incr<T: ConstructableToken>(&mut self, token_type: TokenType, length: usize) -> T {
        let token = self.new_token(token_type, self.position, length);
        self.position = self.position + length;
        token
    }
}

impl<'a> ParseErrorContextProvider for Tokenizer<'a> {
    fn get_line_context(&self, position: usize, max_lines: usize) -> Option<ParseErrorContext> {
        Some(ParseErrorContext::from_chars(
            self.text,
            &self.chars,
            position,
            max_lines,
        ))
    }
}

/// Represents a tokenizer as an iterator of `Token` objects.
pub struct TokenIterator<'a, T: ConstructableToken> {
    tokenizer: &'a mut Tokenizer<'a>,
    finished: bool,
    _phantom: PhantomData<T>,
}

impl<'a, T: ConstructableToken> TokenIterator<'a, T> {
    pub fn new(tokenizer: &'a mut Tokenizer<'a>) -> TokenIterator<'a, T> {
        TokenIterator {
            tokenizer,
            finished: false,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: ConstructableToken> Iterator for TokenIterator<'a, T> {
    type Item = Result<T, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        match self.tokenizer.next_generic() {
            Ok(val) => match val {
                Some(t) => Some(Ok(t)),
                None => None,
            },
            Err(e) => {
                // stop reading after an error - the parse state will be corrupted
                self.finished = true;
                Some(Err(e))
            }
        }
    }
}
