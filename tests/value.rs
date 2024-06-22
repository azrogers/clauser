use clauser::{error::Error, value::Value};

pub fn single_equal(source: &str, expected: Value, desc: &str) -> Result<(), Error> {
    assert_eq!(
        Value::from_str(source)?,
        Value::Object(vec![("val", expected)]),
        "{}",
        desc
    );

    Ok(())
}

#[test]
pub fn parse_primitives() -> Result<(), Error> {
    single_equal("val = 123", Value::Integer(123), "integer parsing")?;
    single_equal("val = 1.23", Value::Decimal(1.23), "decimal parsing")?;
    single_equal("val = \"test\"", Value::String("test"), "string parsing")?;
    single_equal(
        "val = test",
        Value::Identifier("test"),
        "identifier parsing",
    )?;
    single_equal("val = yes", Value::Boolean(true), "boolean parsing")?;

    Ok(())
}

#[test]
pub fn parse_arrays() -> Result<(), Error> {
    single_equal(
        "val = { 0 1 }",
        Value::Array(vec![Value::Integer(0), Value::Integer(1)]),
        "simple array",
    )?;

    single_equal(
        "val = { { 0 } { 1 } }",
        Value::Array(vec![
            Value::Array(vec![Value::Integer(0)]),
            Value::Array(vec![Value::Integer(1)]),
        ]),
        "nested array",
    )?;

    Ok(())
}

#[test]
pub fn parse_objects() -> Result<(), Error> {
    single_equal(
        "val = { a = 1 b = 2 }",
        Value::Object(vec![("a", Value::Integer(1)), ("b", Value::Integer(2))]),
        "simple object",
    )?;
    single_equal(
        "val = { a = 1 b = test }",
        Value::Object(vec![
            ("a", Value::Integer(1)),
            ("b", Value::Identifier("test")),
        ]),
        "multiple object field types",
    )?;
    single_equal(
        "val = { a = { 0 } b = { c = no } }",
        Value::Object(vec![
            ("a", Value::Array(vec![Value::Integer(0)])),
            ("b", Value::Object(vec![("c", Value::Boolean(false))])),
        ]),
        "multiple object field types",
    )?;

    Ok(())
}
