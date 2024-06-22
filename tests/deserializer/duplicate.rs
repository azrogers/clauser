use clauser::{
    parse_error::{ParseError, ParseErrorType},
    test::{expect_error, SingleContainer},
};
use clauser_macros::{duplicate_keys, EnableDuplicateKeys};

#[derive(Debug, PartialEq)]
#[duplicate_keys]
struct DuplicateKeys {
    #[from_duplicate_key]
    item: Vec<String>,
}

#[test]
pub fn duplicate_keys() -> Result<(), ParseError> {
    SingleContainer::<DuplicateKeys>::expect(
        "val = { item = one item = two item = three }",
        DuplicateKeys {
            item: vec!["one", "two", "three"]
                .iter()
                .map(|s| String::from(*s))
                .collect(),
        },
    )?;
    SingleContainer::<DuplicateKeys>::expect(
        "val = { item = one }",
        DuplicateKeys {
            item: vec![String::from("one")],
        },
    )?;

    Ok(())
}

#[derive(Debug, PartialEq)]
#[duplicate_keys]
struct DuplicateAndNormalKeys {
    #[from_duplicate_key]
    item: Vec<String>,
    unique1: i32,
    unique2: String,
}

#[test]
pub fn duplicate_and_normal_keys() -> Result<(), ParseError> {
    SingleContainer::<DuplicateAndNormalKeys>::expect(
        "val = { item = one unique1 = 50 item = two item = three unique2 = cool }",
        DuplicateAndNormalKeys {
            item: vec!["one", "two", "three"]
                .iter()
                .map(|s| String::from(*s))
                .collect(),
            unique1: 50,
            unique2: String::from("cool"),
        },
    )?;

    Ok(())
}

#[test]
pub fn empty_duplicate() -> Result<(), ParseError> {
    SingleContainer::<DuplicateAndNormalKeys>::expect(
        "val = { unique1 = 0 unique2 = test }",
        DuplicateAndNormalKeys {
            item: vec![],
            unique1: 0,
            unique2: String::from("test"),
        },
    )?;

    SingleContainer::<DuplicateKeys>::expect("val = { }", DuplicateKeys { item: vec![] })?;

    Ok(())
}

#[test]
pub fn duplicate_keys_invalid() -> Result<(), ParseError> {
    expect_error::<SingleContainer<i32>>(
        "val = 1 val = 2 val = 3",
        ParseErrorType::DuplicateField,
    )?;

    Ok(())
}
