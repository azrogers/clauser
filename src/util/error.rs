use std::fmt::{self, Debug, Display, Write};

use pad::PadStr;

#[cfg(feature = "serde")]
use serde::{
    de::{self, Expected, Unexpected},
    ser::{self, SerializeMap},
    Serialize,
};

use super::text_helpers::{count_tabs_before, CharHelper, StringBuilder};

const ERROR_CONTEXT_MAX_LINES: usize = 5;

fn join_list(list: &'static [&'static str]) -> String {
    let vec: Vec<String> = list.into_iter().map(|s| s.to_string()).collect();
    vec.join(", ")
}

#[cfg(feature = "serde")]
fn unexpected_to_string(unexp: &Unexpected) -> String {
    match unexp {
        Unexpected::Bool(val) => format!("unexpected bool value {}", val),
        Unexpected::Unsigned(val) => format!("unexpected unsigned value {}", val),
        Unexpected::Signed(val) => format!("unexpected signed value {}", val),
        Unexpected::Float(val) => format!("unexpected float value {}", val),
        Unexpected::Char(val) => format!("unexpected char value {}", val),
        Unexpected::Str(val) => format!("unexpected str value {}", val),
        Unexpected::Other(message) => format!("unexpected value: {}", message),
        _ => format!("unexpected {} value", unexp),
    }
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorType {
    /// Failed to tokenize string.
    TokenizerError,
    /// An operation was attempted that doesn't match the parser's internal state - for example, calling
    /// EndReadObject on a ClReader without calling BeginReadObject.
    DepthMismatchError,
    /// The reader expected to find a token of a certain type but instead found another.
    UnexpectedTokenError,
    /// An operation to read a number failed because the number was an invalid format.
    InvalidNumberError,
    /// An unknown key was encountered while reading an object.
    UnknownKeyError,
    /// File contains a different type than what was expected.
    TypeMismatchError,
    /// Something that isn't currently supported.
    Unsupported,
    /// The parser is in an invalid state (for example, an invalid pointer)
    InvalidState,
    /// Some unknown error has been encountered.
    Unknown,
    /// Invalid type encountered while deserializing.
    InvalidType,
    /// Collection of invalid length encountered while deserializing.
    InvalidLength,
    /// Some sort of invalid value was encountered while deserializing.
    InvalidValue,
    /// The deserializer encountered an enum with an unrecognized value.
    UnknownVariant,
    /// The deserializer encountered a field with an unrecognized name.
    UnknownField,
    /// The deserializer expected a required field in the input but it was not found.
    MissingField,
    /// More than one of the same field was present in the input.
    DuplicateField,
}

pub struct ErrorContext {
    /// The lines leading up to and including the line the error was on.
    lines: Vec<String>,
    /// The position transformed into a (line, col) pair.
    location: (usize, usize),
}

impl ErrorContext {
    pub fn new(lines: Vec<String>, location: (usize, usize)) -> ErrorContext {
        ErrorContext { lines, location }
    }

    pub fn from_chars(
        text: &str,
        chars: &Vec<char>,
        position: usize,
        max_lines: usize,
    ) -> ErrorContext {
        let helper = CharHelper(chars);

        let lines = helper
            .find_line_and_context(position, max_lines)
            .iter()
            .map(|(start, len)| (&text[*start..(*start + *len)]).to_owned())
            .collect();

        let location = helper.position_to_line_col(position);

        ErrorContext { lines, location }
    }
}

pub trait ErrorContextProvider {
    /// Returns the line that the given position is on as well as up to `max_lines` previous lines.
    fn get_line_context(&self, position: usize, max_lines: usize) -> Option<ErrorContext>;
}

pub struct Error {
    pub error_type: ErrorType,
    pub position: Option<usize>,
    pub message: String,
    context: Option<ErrorContext>,
}

impl Error {
    pub fn new(
        context: Option<&impl ErrorContextProvider>,
        error_type: ErrorType,
        position: usize,
        message: impl ToString,
    ) -> Error {
        Error {
            error_type,
            position: Some(position),
            message: message.to_string(),
            context: context.and_then(|p| p.get_line_context(position, ERROR_CONTEXT_MAX_LINES)),
        }
    }

    pub fn new_unanchored(error_type: ErrorType, message: impl ToString) -> Error {
        Error {
            error_type,
            position: None,
            message: message.to_string(),
            context: None,
        }
    }

    pub fn with_context(&self, context: &impl ErrorContextProvider, position: usize) -> Error {
        Error {
            error_type: self.error_type.clone(),
            position: Some(position),
            context: context.get_line_context(position, ERROR_CONTEXT_MAX_LINES),
            message: self.message.clone(),
        }
    }

    /// Does this error currently have a `context` provided?
    pub fn has_context(&self) -> bool {
        self.context.is_some()
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.position.is_none() {
            return write!(
                f,
                "ErrorType::{:?} encountered at an unknown position: {}",
                self.error_type, self.message
            );
        }

        let position = self.position.unwrap();

        if let Some(line_context) = &self.context {
            let (line, col) = line_context.location;

            let line_num_magnitude =
                isize::max(f32::floor(f32::log10(line as f32) + 1.0) as isize, 0) as usize;

            f.write_char('\n')?;

            let mut builder = StringBuilder::new();
            let mut current_line_num =
                isize::max((line - line_context.lines.len()) as isize + 1, 0) as usize;

            for l in &line_context.lines {
                let padded_str = current_line_num
                    .to_string()
                    .pad_to_width_with_char(line_num_magnitude, ' ')
                    + ". ";
                builder.append(&padded_str);
                builder.append(&l.trim_end());
                builder.append_char('\n');

                current_line_num = current_line_num + 1;
            }

            // we don't know the tab width of the terminal so best we can do is just shove all of em at the start
            let tab_count = match line_context.lines.last() {
                Some(line) => count_tabs_before(line, col),
                None => 0,
            };

            let col_pos = isize::max(col as isize - 1 - tab_count as isize, 0) as usize;
            let display_offset = line_num_magnitude + 2; // width of line number plus ". "

            // offset to show position in line
            let offset_str = std::iter::repeat(" ")
                .take(display_offset)
                .chain(std::iter::repeat("\t").take(tab_count))
                .chain(std::iter::repeat(".").take(col_pos))
                .collect::<String>()
                + "^";

            builder.append(&offset_str);
            f.write_str(&builder.to_string())?;
            f.write_char('\n')?;

            return write!(
                f,
                "ErrorType::{:?} encountered at line {} column {}: {}",
                self.error_type, line, col, self.message
            );
        }

        write!(
            f,
            "ErrorType::{:?} encountered at index {}: {}",
            self.error_type, position, self.message
        )
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Clauser parse error at position {}: {}",
            self.position
                .and_then(|p| Some(p.to_string()))
                .unwrap_or(String::from("unknown")),
            self.message
        )
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let prop_count: usize =
            2 + self.position.map_or(0, |_| 1) + self.context.as_ref().map_or(0, |_| 2);
        let mut map = serializer.serialize_map(Some(prop_count))?;
        map.serialize_entry("error_type", &self.error_type)?;
        map.serialize_entry("message", &self.message)?;
        if let Some(pos) = self.position {
            map.serialize_entry("index", &pos)?;
        }
        if let Some(context) = &self.context {
            map.serialize_entry("context", &context.lines)?;
            map.serialize_entry("location", &context.location)?;
        }
        map.end()
    }
}

impl std::error::Error for Error {}

#[cfg(feature = "serde")]
impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::new_unanchored(ErrorType::Unknown, msg.to_string())
    }

    fn invalid_type(unexp: Unexpected, exp: &dyn Expected) -> Self {
        Error::new_unanchored(
            ErrorType::InvalidType,
            format!(
                "invalid type: {}, expected {}",
                unexpected_to_string(&unexp),
                exp
            ),
        )
    }

    fn invalid_value(unexp: Unexpected, exp: &dyn Expected) -> Self {
        Error::new_unanchored(
            ErrorType::InvalidValue,
            format!(
                "invalid value: {}, expected {}",
                unexpected_to_string(&unexp),
                exp
            ),
        )
    }

    fn invalid_length(len: usize, exp: &dyn Expected) -> Self {
        Error::new_unanchored(
            ErrorType::InvalidLength,
            format!("invalid length {}, expected {}", len, exp),
        )
    }

    fn unknown_variant(variant: &str, expected: &'static [&'static str]) -> Self {
        let message = match expected.is_empty() {
            true => format!("unknown variant {}, there are no variants", variant),
            false => format!(
                "unknown variant {}, expected one of {}",
                variant,
                join_list(expected)
            ),
        };

        Error::new_unanchored(ErrorType::UnknownVariant, message)
    }

    fn unknown_field(field: &str, expected: &'static [&'static str]) -> Self {
        let message = match expected.is_empty() {
            true => format!("unknown variant {}, there are no fields", field),
            false => format!(
                "unknown field {}, expected one of {}",
                field,
                join_list(expected)
            ),
        };

        Error::new_unanchored(ErrorType::UnknownField, message)
    }

    fn missing_field(field: &'static str) -> Self {
        Error::new_unanchored(
            ErrorType::MissingField,
            format!("missing field {} in input", field),
        )
    }

    fn duplicate_field(field: &'static str) -> Self {
        Error::new_unanchored(
            ErrorType::DuplicateField,
            format!("duplicate field {} in input", field),
        )
    }
}

pub type ParseResult<T> = Result<Option<T>, Error>;
pub type ParseCompleteResult<T> = Result<T, Error>;
