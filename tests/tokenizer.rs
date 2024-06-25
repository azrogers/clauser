use clauser::{
    error::{Error, ErrorType},
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

fn expect_error(text: &str) -> Result<(), Error> {
    let tokens = Tokenizer::parse_all(text);
    assert!(
        tokens.is_err(),
        "expected result to be err, got {:?}",
        tokens.unwrap()
    );
    match tokens {
        Ok(_) => Ok(()),
        Err(e) => match e.error_type == ErrorType::TokenizerError {
            true => Ok(()),
            false => Err(e),
        },
    }
}

fn expect_token_and_value(
    text: &str,
    expected_type: TokenType,
    expected_value: &str,
) -> Result<(), Error> {
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

// verifies that the text matches the token and that it's the right type
fn expect_token(text: &str, expected_type: TokenType) -> Result<(), Error> {
    expect_token_and_value(text, expected_type, text)
}

#[test]
fn number() -> Result<(), Error> {
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
    expect_error("0.1...2")?;
    expect_error("-1.")?;
    expect_error("-.")?;
    expect_error("-.0")?;

    Ok(())
}

#[test]
fn boolean() -> Result<(), Error> {
    expect_token_and_value("yes", TokenType::Boolean, "yes")?;
    expect_token_and_value("no", TokenType::Boolean, "no")?;

    Ok(())
}

#[test]
fn identifier() -> Result<(), Error> {
    expect_token_and_value("test", TokenType::Identifier, "test")?;
    expect_token_and_value("_a_longer_test", TokenType::Identifier, "_a_longer_test")?;
    expect_token_and_value(
        "test:with:colons",
        TokenType::Identifier,
        "test:with:colons",
    )?;

    Ok(())
}

#[test]
fn string() -> Result<(), Error> {
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
fn symbols() -> Result<(), Error> {
    expect_token_and_value("=", TokenType::Equals, "=")?;
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
fn date() -> Result<(), Error> {
    expect_token("1940.1.1", TokenType::Date)?;
    expect_token("1980.08.11.1", TokenType::Date)?;
    expect_token("2031.08.00.2", TokenType::Date)?;
    expect_token("289312.3.37817283780", TokenType::Date)?;

    expect_error("val = 1930.1.")?;
    expect_error("val = 1930.1.3.")?;
    expect_error("val = 1959..1")?;

    Ok(())
}

#[test]
fn iterator() -> Result<(), Error> {
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

#[test]
fn error_context() {}

#[test]
fn error_cases() {}
