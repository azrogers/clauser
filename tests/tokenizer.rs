use clauser::{
    parse_error::{ParseError, ParseErrorType},
    token::{OwnedToken, TokenType},
    tokenizer::Tokenizer,
};

#[derive(Debug)]
pub struct ExpectedToken(TokenType, &'static str);

impl PartialEq<OwnedToken> for ExpectedToken {
    fn eq(&self, other: &OwnedToken) -> bool {
        other.token_type == self.0 && other.value == self.1
    }
}

impl PartialEq<ExpectedToken> for OwnedToken {
    fn eq(&self, other: &ExpectedToken) -> bool {
        PartialEq::<OwnedToken>::eq(other, self)
    }
}

fn assert_vec_equal(tokens: &Vec<OwnedToken>, expected: &Vec<ExpectedToken>) {
    assert_eq!(
        tokens.len(),
        expected.len(),
        "number of tokens {} doesn't equal expected {}",
        tokens.len(),
        expected.len()
    );

    let mut i = 0;
    while i < expected.len() {
        assert_eq!(
            tokens[i], expected[i],
            "token {:?} at index {} doesn't match expected {:?}",
            tokens[i], i, expected[i]
        );

        i = i + 1;
    }
}

fn expect_error(text: &str) -> Result<(), ParseError> {
    let tokens = Tokenizer::parse_all(text);
    assert!(
        tokens.is_err(),
        "expected result to be err, got {:?}",
        tokens.unwrap()
    );
    match tokens {
        Ok(_) => Ok(()),
        Err(e) => match e.error_type == ParseErrorType::TokenizerError {
            true => Ok(()),
            false => Err(e),
        },
    }
}

fn expect_token_and_value(
    text: &str,
    expected_type: TokenType,
    expected_value: &str,
) -> Result<(), ParseError> {
    let tokens = Tokenizer::parse_all(text)?;
    assert_eq!(
        tokens.len(),
        1,
        "only one token is expected to be parsed, but {} were found",
        tokens.len()
    );
    let token = tokens.first().unwrap();
    assert_eq!(token.token_type, expected_type);
    assert_eq!(token.value, expected_value);
    Ok(())
}

#[test]
fn basic_number() -> Result<(), ParseError> {
    expect_token_and_value("100", TokenType::Number, "100")?;
    expect_token_and_value("-100", TokenType::Number, "-100")?;
    expect_token_and_value("3019.29", TokenType::Number, "3019.29")?;
    expect_token_and_value("-3019.29", TokenType::Number, "-3019.29")?;
    expect_token_and_value("\t\t\t100.0\t\t\n", TokenType::Number, "100.0")?;
    expect_token_and_value(
        "# cool comment\n\t\t\t100.0\t\t\n",
        TokenType::Number,
        "100.0",
    )?;
    expect_error("-")?;
    expect_error(".01")?;
    expect_error("0.1.2")?;
    expect_error("-1.")?;
    expect_error("-.")?;
    expect_error("-.0")?;

    Ok(())
}

#[test]
fn basic_identifier_boolean() -> Result<(), ParseError> {
    expect_token_and_value("yes", TokenType::Boolean, "yes")?;
    expect_token_and_value("no", TokenType::Boolean, "no")?;
    expect_token_and_value("test", TokenType::Identifier, "test")?;
    expect_token_and_value("_a_longer_test", TokenType::Identifier, "_a_longer_test")?;

    Ok(())
}

#[test]
fn basic_string() -> Result<(), ParseError> {
    expect_token_and_value("\"str\"", TokenType::String, "str")?;
    expect_token_and_value(
        "\"this is\na multi line string\"",
        TokenType::String,
        "this is\na multi line string",
    )?;
    assert_vec_equal(
        &Tokenizer::parse_all("\"str1\"\"str2\"#comment\n\"str3\"")?,
        &vec![
            ExpectedToken(TokenType::String, "str1"),
            ExpectedToken(TokenType::String, "str2"),
            ExpectedToken(TokenType::String, "str3"),
        ],
    );

    expect_error("\"unclosed")?;
    expect_error("unopened\"")?;
    expect_error("'single quotes'")?;

    Ok(())
}

#[test]
fn basic_symbols() -> Result<(), ParseError> {
    expect_token_and_value("=", TokenType::Equals, "=")?;
    expect_token_and_value(":", TokenType::Colon, ":")?;
    expect_token_and_value("{", TokenType::OpenBracket, "{")?;
    expect_token_and_value("}", TokenType::CloseBracket, "}")?;
    expect_token_and_value(">", TokenType::GreaterThan, ">")?;
    expect_token_and_value(">=", TokenType::GreaterThanEq, ">=")?;
    expect_token_and_value("<", TokenType::LessThan, "<")?;
    expect_token_and_value("<=", TokenType::LessThanEq, "<=")?;
    expect_token_and_value("?=", TokenType::ExistenceCheck, "?=")?;

    Ok(())
}

#[test]
fn iterator() -> Result<(), ParseError> {
    let tokens = Tokenizer::parse_all("{ property = \"test\" } # comment\n82.3 > 1 >= 0")?;
    let expected = vec![
        ExpectedToken(TokenType::OpenBracket, "{"),
        ExpectedToken(TokenType::Identifier, "property"),
        ExpectedToken(TokenType::Equals, "="),
        ExpectedToken(TokenType::String, "test"),
        ExpectedToken(TokenType::CloseBracket, "}"),
        ExpectedToken(TokenType::Number, "82.3"),
        ExpectedToken(TokenType::GreaterThan, ">"),
        ExpectedToken(TokenType::Number, "1"),
        ExpectedToken(TokenType::GreaterThanEq, ">="),
        ExpectedToken(TokenType::Number, "0"),
    ];
    assert_vec_equal(&tokens, &expected);
    Ok(())
}
