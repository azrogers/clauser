use std::fmt::Debug;

#[cfg(test)]
use clauser::test::{expect_error, SingleContainer};

use clauser::{
    deserializer::de::from_str,
    parse_error::{ParseError, ParseErrorType},
    test::expect_str,
};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct BasicKeyValue {
    pub bool_val: bool,
    pub int_val: i32,
    pub float_val: f64,
    pub str_val: String,
    pub id_val: String,
}

#[test]
fn basic_key_value() -> Result<(), ParseError> {
    let source = "
	bool_val = yes
	int_val = -193
	float_val = 19.3
	str_val = \"hello world!\"
	id_val = ident";

    let deserialized = from_str::<BasicKeyValue>(&source)?;
    assert_eq!(deserialized.bool_val, true);
    assert_eq!(deserialized.int_val, -193);
    assert_eq!(deserialized.float_val, 19.3);
    assert_eq!(deserialized.str_val, "hello world!");
    assert_eq!(deserialized.id_val, "ident");

    expect_error::<BasicKeyValue>("bool_val = yes", ParseErrorType::MissingField)?;
    expect_error::<BasicKeyValue>("bool_val = 18", ParseErrorType::UnexpectedTokenError)?;

    Ok(())
}

#[derive(Deserialize, Debug)]
struct NestedKeyValue {
    pub obj: BasicKeyValue,
}

#[test]
fn nested_key_value() -> Result<(), ParseError> {
    let source = "
	obj = {
		bool_val = no
		int_val = 2
		float_val = 1.0
		str_val = \"test\"
		id_val = none
	}";

    let deserialized = from_str::<NestedKeyValue>(&source)?;
    assert_eq!(deserialized.obj.bool_val, false);
    assert_eq!(deserialized.obj.int_val, 2);
    assert_eq!(deserialized.obj.float_val, 1.0);
    assert_eq!(deserialized.obj.str_val, "test");
    assert_eq!(deserialized.obj.id_val, "none");

    expect_error::<NestedKeyValue>("obj = 18", ParseErrorType::UnexpectedTokenError)?;
    expect_error::<NestedKeyValue>("obj = {}", ParseErrorType::MissingField)?;
    expect_error::<NestedKeyValue>(
        "obj = { bool_val = 18 }",
        ParseErrorType::UnexpectedTokenError,
    )?;

    Ok(())
}

#[test]
fn primitive_array() -> Result<(), ParseError> {
    assert_eq!(
        from_str::<SingleContainer<Vec<i32>>>("val = { 8 -10 20 30000 49982 0 }")?.val,
        vec![8, -10, 20, 30000, 49982, 0]
    );
    assert_eq!(
        from_str::<SingleContainer<Vec<i32>>>("val = {}")?.val,
        vec![]
    );

    expect_error::<SingleContainer<Vec<i32>>>(
        "val = { 10.0 93 -1 }",
        ParseErrorType::InvalidNumberError,
    )?;
    expect_error::<SingleContainer<Vec<i32>>>(
        "val = { \"test\" }",
        ParseErrorType::UnexpectedTokenError,
    )?;
    expect_error::<SingleContainer<Vec<i32>>>(
        "val = { 18 test }",
        ParseErrorType::UnexpectedTokenError,
    )?;

    Ok(())
}

#[derive(Debug, Deserialize, PartialEq)]
struct StringField {
    str: String,
}

#[test]
pub fn empty_string() -> Result<(), ParseError> {
    SingleContainer::<String>::expect("val = ", String::new())?;
    SingleContainer::<StringField>::expect("val = { str = }", StringField { str: String::new() })?;

    Ok(())
}

#[derive(Debug, Deserialize, PartialEq)]
struct MultiStringField {
    str1: String,
    str2: String,
    str3: String,
    str4: String,
}

#[test]
pub fn significant_newlines() -> Result<(), ParseError> {
    let source = "
		str1 = 
		str2 = test
		str3 =
		str4 = test";

    expect_str::<MultiStringField>(
        source,
        MultiStringField {
            str1: String::new(),
            str2: String::from("test"),
            str3: String::new(),
            str4: String::from("test"),
        },
    )?;

    Ok(())
}
