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
    ExpectValue,
    RootNotSingular,
    InvalidValue,
    NumberTooBig,
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
