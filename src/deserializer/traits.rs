use std::marker::PhantomData;

use serde::{
    de::{Error, MapAccess, Visitor},
    Deserialize,
};

/*
use crate::parse_error::ParseError;

pub struct DuplicateKeys<T> {
    values: Vec<T>,
}

impl<T> DuplicateKeys<T> {
    const _CL_SENTINEL: usize = 0xdeadbeef;
}
struct TestStruct {
    item: Vec<i32>,
    test: String,
}

struct TestStructVisitor<T>(PhantomData<T>);

impl<'de, T> Visitor<'de> for TestStructVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = TestStruct;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a TestStruct")
    }

    fn visit_map<A>(self, mut map: A) -> Result<TestStruct, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut item_vec: Vec<i32> = Vec::new();
        let mut test: Option<String> = None;

        while let Some((key, value)) = map.next_entry()? {
            match key {
                "item" => Err(item_vec.push(value)),
                "test" => test = Some(value),
                _ => ParseError::unknown_field(key, &[]),
            }?;
        }

        Ok(TestStruct {
            item: item_vec,
            test: String::from(""),
        })
    }
}

impl TestStructVisitor<TestStruct> {
    pub fn test() -> TestStructVisitor<TestStruct> { return TestStructVisitor<TestStruct>(PhantomData); }
}*/
