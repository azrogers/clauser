use std::include_str;

use clauser::{error::Error, value::Value};

const denmark: &str = include_str!("history/countries/DEN - Denmark.txt");

#[test]
pub fn denmark_value() -> Result<(), Error> {
    let value = Value::from_str(denmark)?;

    Ok(())
}
