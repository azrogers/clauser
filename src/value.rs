use std::borrow::{Borrow, Cow};
use std::fmt::Debug;

use crate::{
    error::Error,
    reader::Reader,
    types::{CollectionType, RealType},
};

/// A variant that represents a Clausewitz source file as a tree of types and values.
/// [from_reader](`Value::from_reader`) and [from_str](`Value::from_str`) can be used to deserialize
/// a Clausewitz source file into a Value tree.
///
/// Using [Value] to deserialize is slower and uses more memory than using [Deserializer](`clauser::de::Deserializer``).
/// Unlike [Deserializer](`clauser::de::Deserializer`), [Value] allocates values on the heap (though strings are not copied).
/// It also makes no guarantees about the validity of the data, as long as it's parseable.
///
/// You should only use [Value] for situations where the schema of the data can't be known beforehand.
#[derive(Debug, PartialEq)]
pub enum ValueBase<T>
where
    T: std::fmt::Debug + ToOwned,
{
    None,
    Integer(i64),
    Decimal(f64),
    Boolean(bool),
    String(T),
    Identifier(T),
    Object(Vec<(T, ValueBase<T>)>),
    Array(Vec<ValueBase<T>>),
}

pub type Value<'src> = ValueBase<&'src str>;
pub type ValueOwned = ValueBase<String>;
pub type ValueCow<'src> = ValueBase<Cow<'src, str>>;

impl<T> ToOwned for ValueBase<T>
where
    ValueOwned: Borrow<ValueBase<T>>,
    T: Into<String> + std::fmt::Debug + ToOwned,
    for<'a> &'a T: Into<String>,
{
    type Owned = ValueOwned;

    /// Creates a copy of a [ValueBase] variant where all strings are owned.
    fn to_owned(&self) -> Self::Owned {
        match self {
            ValueBase::None => ValueOwned::None,
            ValueBase::Integer(v) => ValueOwned::Integer(*v),
            ValueBase::Decimal(v) => ValueOwned::Decimal(*v),
            ValueBase::Boolean(v) => ValueOwned::Boolean(*v),
            ValueBase::String(v) => ValueOwned::String(v.into()),
            ValueBase::Identifier(v) => ValueOwned::Identifier(v.into()),
            ValueBase::Array(v) => ValueOwned::Array(v.into_iter().map(|v| v.to_owned()).collect()),
            ValueBase::Object(v) => ValueOwned::Object(
                v.into_iter()
                    .map(|(k, v)| (k.into(), v.to_owned()))
                    .collect(),
            ),
        }
    }
}

impl<'reader, 'src: 'reader, T> ValueBase<T>
where
    T: std::fmt::Debug + ToOwned + From<&'src str>,
{
    /// Recursively reads a source file from a [Reader] into a [Value].
    pub fn from_reader(reader: &'reader mut Reader<'src>) -> Result<ValueBase<T>, Error> {
        let mut values = Vec::new();

        while let Some((name, _)) = reader.next_property()? {
            values.push((name.into(), Self::next_from_reader(reader)?))
        }

        return Ok(Self::Object(values));
    }

    /// Recursively reads the next available value from a [Reader] into a [Value].
    pub fn next_from_reader(reader: &'reader mut Reader<'src>) -> Result<ValueBase<T>, Error> {
        let next = reader.peek_next_type()?;

        if next.is_none() {
            return Ok(Self::None);
        }

        let next = next.unwrap();

        match next {
            RealType::Boolean => Ok(Self::Boolean(reader.read_boolean()?)),
            RealType::Identifier => Ok(Self::Identifier(reader.read_identifier()?.into())),
            RealType::String => Ok(Self::String(reader.read_string()?.into())),
            RealType::Number => Ok({
                let number = reader.read_number_as_str()?;
                match number.contains(".") {
                    true => Self::Decimal(reader.parse_number(number)?),
                    false => Self::Integer(reader.parse_number(number)?),
                }
            }),
            RealType::ObjectOrArray => {
                let collection_type = reader.try_discern_array_or_map()?;

                if collection_type.is_none() {
                    return Ok(Self::None);
                }

                match collection_type.unwrap() {
                    CollectionType::Array => {
                        reader.begin_collection()?;

                        let mut values = Vec::new();
                        while !reader.is_collection_ended()? {
                            values.push(Self::next_from_reader(reader)?)
                        }

                        reader.end_collection()?;

                        Ok(Self::Array(values))
                    }
                    CollectionType::Object => {
                        reader.begin_collection()?;

                        let mut values = Vec::new();

                        while let Some((name, _)) = reader.next_property()? {
                            values.push((name.into(), Self::next_from_reader(reader)?));
                        }

                        reader.end_collection()?;

                        Ok(Self::Object(values))
                    }
                }
            }
        }
    }

    /// Parses the given string into a [Value].
    pub fn from_str(s: &'src str) -> Result<Value<'src>, Error> {
        let mut reader = Reader::new(s);
        Value::from_reader(&mut reader)
    }
}
