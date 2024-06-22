use std::fmt::Debug;

use super::util::{expect_error, SingleContainer};

use clauser::{
    de::from_str,
    error::{Error, ErrorType},
};
use clauser_macros::EnableDuplicateKeys;
use serde::Deserialize;

#[derive(Deserialize, PartialEq, Debug)]
enum BasicEnum {
    Value1,
    Value2,
    Value3,
}

#[test]
fn basic_enum() -> Result<(), Error> {
    assert_eq!(
        from_str::<SingleContainer<BasicEnum>>("val = Value1")?.val,
        BasicEnum::Value1
    );
    assert_eq!(
        from_str::<SingleContainer<BasicEnum>>("val = Value2")?.val,
        BasicEnum::Value2
    );
    assert_eq!(
        from_str::<SingleContainer<BasicEnum>>("val = Value3")?.val,
        BasicEnum::Value3
    );

    expect_error::<SingleContainer<BasicEnum>>("val = Value0", ErrorType::UnknownVariant)?;
    expect_error::<SingleContainer<BasicEnum>>("val = 100", ErrorType::UnexpectedTokenError)?;
    expect_error::<SingleContainer<BasicEnum>>("val = ", ErrorType::UnexpectedTokenError)?;

    Ok(())
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
enum BasicUntaggedEnum {
    Unit,
    Item(bool),
    Pair(i32, i32),
    Tuple(i32, f32, String),
}

#[test]
fn basic_untagged_enum() -> Result<(), Error> {
    SingleContainer::<BasicUntaggedEnum>::expect("val = ", BasicUntaggedEnum::Unit)?;
    SingleContainer::<BasicUntaggedEnum>::expect("val = yes", BasicUntaggedEnum::Item(true))?;
    SingleContainer::<BasicUntaggedEnum>::expect("val = { 0 1 }", BasicUntaggedEnum::Pair(0, 1))?;
    SingleContainer::<BasicUntaggedEnum>::expect(
        "val = { 0 1.0 \"test\" }",
        BasicUntaggedEnum::Tuple(0, 1.0, String::from("test")),
    )?;

    Ok(())
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
enum ComplexUntaggedEnum {
    Newtype(i32),
    Struct(SingleContainer<Vec<i32>>),
    Array(Vec<bool>),
    Tuple(f32, f32, f32),
    Optional(Option<String>),
}

#[test]
fn complex_enum() -> Result<(), Error> {
    SingleContainer::<ComplexUntaggedEnum>::expect("val = 20", ComplexUntaggedEnum::Newtype(20))?;

    assert_eq!(
        from_str::<SingleContainer<ComplexUntaggedEnum>>("val = { val = { 0 1 2 3 } }")?.val,
        ComplexUntaggedEnum::Struct(SingleContainer::new(vec![0, 1, 2, 3]))
    );
    assert_eq!(
        from_str::<SingleContainer<ComplexUntaggedEnum>>("val = { yes no yes }")?.val,
        ComplexUntaggedEnum::Array(vec![true, false, true])
    );
    assert_eq!(
        from_str::<SingleContainer<ComplexUntaggedEnum>>("val = { 0.0 1.0 2.0 }")?.val,
        ComplexUntaggedEnum::Tuple(0.0, 1.0, 2.0)
    );
    assert_eq!(
        from_str::<SingleContainer<ComplexUntaggedEnum>>("val = \"test\"")?.val,
        ComplexUntaggedEnum::Optional(Some(String::from("test")))
    );
    assert_eq!(
        from_str::<SingleContainer<ComplexUntaggedEnum>>("val = ")?.val,
        ComplexUntaggedEnum::Optional(None)
    );

    Ok(())
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
enum InternallyTaggedEnum {
    Unit,
    Item { num: i64 },
}

#[test]
fn internally_tagged_enum() -> Result<(), Error> {
    SingleContainer::<InternallyTaggedEnum>::expect(
        "val = { type = Unit }",
        InternallyTaggedEnum::Unit,
    )?;
    SingleContainer::<InternallyTaggedEnum>::expect(
        "val = { type = Item num = 900 }",
        InternallyTaggedEnum::Item { num: 900 },
    )?;

    expect_error::<SingleContainer<InternallyTaggedEnum>>(
        "val = { type = Incorrect }",
        ErrorType::UnknownVariant,
    )?;
    expect_error::<SingleContainer<InternallyTaggedEnum>>(
        "val = { num = 900 }",
        ErrorType::MissingField,
    )?;
    expect_error::<SingleContainer<InternallyTaggedEnum>>("val = 900", ErrorType::InvalidType)?;
    expect_error::<SingleContainer<InternallyTaggedEnum>>(
        "val = { type = \"String\" }",
        ErrorType::UnexpectedTokenError,
    )?;

    Ok(())
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(tag = "t", content = "c")]
enum AdjacentlyTaggedEnum {
    Unit,
    Str(String),
    Option(Option<String>),
    Tuple(i32, i32, f32),
}

#[test]
pub fn adjacently_tagged_enum() -> Result<(), Error> {
    SingleContainer::<AdjacentlyTaggedEnum>::expect(
        "val = { t = Unit }",
        AdjacentlyTaggedEnum::Unit,
    )?;
    SingleContainer::<AdjacentlyTaggedEnum>::expect(
        "val = { t = Str c = \"test\" }",
        AdjacentlyTaggedEnum::Str(String::from("test")),
    )?;
    SingleContainer::<AdjacentlyTaggedEnum>::expect(
        "val = { t = Option c = }",
        AdjacentlyTaggedEnum::Option(None),
    )?;
    SingleContainer::<AdjacentlyTaggedEnum>::expect(
        "val = { t = Option c = \"test\" }",
        AdjacentlyTaggedEnum::Option(Some(String::from("test"))),
    )?;
    SingleContainer::<AdjacentlyTaggedEnum>::expect(
        "val = { t = Tuple c = { 1 2 3.0 } }",
        AdjacentlyTaggedEnum::Tuple(1, 2, 3.0),
    )?;

    expect_error::<SingleContainer<AdjacentlyTaggedEnum>>(
        "val = { t = Incorrect }",
        ErrorType::UnknownVariant,
    )?;
    expect_error::<SingleContainer<AdjacentlyTaggedEnum>>(
        "val = { t = Tuple c = }",
        ErrorType::UnexpectedTokenError,
    )?;
    expect_error::<SingleContainer<AdjacentlyTaggedEnum>>(
        "val = { c = {} }",
        ErrorType::UnexpectedTokenError,
    )?;

    Ok(())
}
