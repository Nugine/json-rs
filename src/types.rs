use std::collections::HashMap;
use std::ops::Index;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

#[derive(Debug, PartialEq)]
pub enum JsonError {
    RootNotSingular,
    InvalidValue,
    NumberTooBig,
    MissingColon,
    UnexpectedEnd,
}

pub type JsonResult<T> = Result<T, JsonError>;

impl JsonValue {
    pub fn as_slice(&self) -> Option<&[JsonValue]> {
        if let JsonValue::Array(ref v) = self {
            Some(v.as_slice())
        } else {
            None
        }
    }
}

impl Index<usize> for JsonValue {
    type Output = JsonValue;

    fn index(&self, index: usize) -> &JsonValue {
        if let JsonValue::Array(ref v) = self {
            &v[index]
        } else {
            panic!("json value is not an array")
        }
    }
}

impl Index<&str> for JsonValue {
    type Output = JsonValue;
    fn index<'a>(&self, index: &'a str) -> &JsonValue {
        if let JsonValue::Object(ref map) = self {
            &map[index]
        } else {
            panic!("json value is not an object")
        }
    }
}
