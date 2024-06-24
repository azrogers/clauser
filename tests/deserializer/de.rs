use std::{collections::HashMap, fmt::Debug};

use super::util::{expect_error, expect_str, SingleContainer};

use clauser::{
    de::from_str,
    error::{Error, ErrorType},
    types::Date,
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
fn basic_key_value() -> Result<(), Error> {
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

    expect_error::<BasicKeyValue>("bool_val = yes", ErrorType::MissingField)?;
    expect_error::<BasicKeyValue>("bool_val = 18", ErrorType::UnexpectedTokenError)?;

    Ok(())
}

#[derive(Deserialize, Debug)]
struct NestedKeyValue {
    pub obj: BasicKeyValue,
}

#[test]
fn nested_key_value() -> Result<(), Error> {
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

    expect_error::<NestedKeyValue>("obj = 18", ErrorType::UnexpectedTokenError)?;
    expect_error::<NestedKeyValue>("obj = {}", ErrorType::MissingField)?;
    expect_error::<NestedKeyValue>("obj = { bool_val = 18 }", ErrorType::UnexpectedTokenError)?;

    Ok(())
}

#[test]
fn primitive_array() -> Result<(), Error> {
    assert_eq!(
        from_str::<SingleContainer<Vec<i32>>>("val = { 8 -10 20 30000 49982 0 }")?.val,
        vec![8, -10, 20, 30000, 49982, 0]
    );
    assert_eq!(
        from_str::<SingleContainer<Vec<i32>>>("val = {}")?.val,
        Vec::<i32>::new()
    );

    expect_error::<SingleContainer<Vec<i32>>>(
        "val = { 10.0 93 -1 }",
        ErrorType::InvalidNumberError,
    )?;
    expect_error::<SingleContainer<Vec<i32>>>(
        "val = { \"test\" }",
        ErrorType::UnexpectedTokenError,
    )?;
    expect_error::<SingleContainer<Vec<i32>>>(
        "val = { 18 test }",
        ErrorType::UnexpectedTokenError,
    )?;

    Ok(())
}

#[derive(Debug, Deserialize, PartialEq)]
struct StringField {
    str: String,
}

#[test]
pub fn empty_string() -> Result<(), Error> {
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
pub fn significant_newlines() -> Result<(), Error> {
    let source = "
		str1 = 
		str2 = test
		str3 =
		str4 = test";

    // todo: is this actually the behavior we want?
    // todo: should identifier values even be *able* to be empty or should that only be options?
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

#[derive(Deserialize, Debug, PartialEq)]
struct DateAsKeys {
    keys: HashMap<Date, String>,
}

#[test]
pub fn dates() -> Result<(), Error> {
    SingleContainer::<Date>::expect("val = 1940.1.1.18", Date::new(1940, 1, 1, 18))?;
    SingleContainer::<Date>::expect("val = 1933.11.4", Date::new(1933, 11, 4, 0))?;
    SingleContainer::<Date>::expect("val = 1033.08.2.30", Date::new(1033, 8, 2, 30))?;
    let mut map = HashMap::new();
    map.insert(Date::new(1932, 1, 3, 0), String::from("test"));
    map.insert(Date::new(1, 1, 1, 1), String::from("ok"));
    SingleContainer::<DateAsKeys>::expect(
        "val = { keys = { 1932.01.3 = test 01.01.01.01 = ok } }",
        DateAsKeys { keys: map },
    )?;

    Ok(())
}
