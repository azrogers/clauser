use clauser::{error::Error, types::Date, value::Value};

pub fn single_equal(source: &str, expected: Value<'static>, desc: &str) -> Result<(), Error> {
    assert_eq!(
        Value::from_str(source)?,
        Value::Object(vec![("val".into(), expected)]),
        "{}",
        desc
    );

    Ok(())
}

#[test]
pub fn parse_primitives() -> Result<(), Error> {
    single_equal("val = 123", Value::Integer(123), "integer parsing")?;
    single_equal("val = 1.23", Value::Decimal(1.23), "decimal parsing")?;
    single_equal(
        "val = \"test\"",
        Value::String("test".into()),
        "string parsing",
    )?;
    single_equal(
        "val = test",
        Value::Identifier("test".into()),
        "identifier parsing",
    )?;
    single_equal("val = yes", Value::Boolean(true), "boolean parsing")?;

    Ok(())
}

#[test]
pub fn parse_arrays() -> Result<(), Error> {
    single_equal("val = { }", Value::Array(vec![]), "empty array")?;
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
        Value::Object(vec![
            ("a".into(), Value::Integer(1)),
            ("b".into(), Value::Integer(2)),
        ]),
        "simple object",
    )?;
    single_equal(
        "val = { a = 1 b = test }",
        Value::Object(vec![
            ("a".into(), Value::Integer(1)),
            ("b".into(), Value::Identifier("test".into())),
        ]),
        "multiple object field types",
    )?;
    single_equal(
        "val = { a = { 0 } b = { c = no } }",
        Value::Object(vec![
            ("a".into(), Value::Array(vec![Value::Integer(0)])),
            (
                "b".into(),
                Value::Object(vec![("c".into(), Value::Boolean(false))]),
            ),
        ]),
        "multiple object field types",
    )?;

    Ok(())
}

#[test]
pub fn dates() -> Result<(), Error> {
    single_equal(
        "val = 1940.1.1.15",
        Value::Date(Date::new(1940, 1, 1, 15)),
        "date parsing",
    )?;
    single_equal(
        "val = 4891.1312310.0099.1000090",
        Value::Date(Date::new(4891, 1312310, 99, 1000090)),
        "long date parsing",
    )?;
    single_equal(
        "val = { 2003.1.1 = { 0 } 1902.12.3 = { test = a test2 = b } }",
        Value::Object(vec![
            (
                Date::new(2003, 1, 1, 0).into(),
                Value::Array(vec![Value::Integer(0)]),
            ),
            (
                Date::new(1902, 12, 3, 0).into(),
                Value::Object(vec![
                    ("test".into(), Value::Identifier("a".into())),
                    ("test2".into(), Value::Identifier("b".into())),
                ]),
            ),
        ]),
        "dates as keys",
    )?;

    Ok(())
}
