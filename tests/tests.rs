use json_rs::{JsonError, JsonValue};

#[test]
fn test_parse() {
    assert_eq!(json_rs::parse(" l "), Err(JsonError::InvalidValue));
    assert_eq!(json_rs::parse(" "), Err(JsonError::ExpectValue));
    assert_eq!(json_rs::parse(""), Err(JsonError::ExpectValue));
}

#[test]
fn test_parse_null() {
    assert_eq!(json_rs::parse("null"), Ok(JsonValue::Null));
    assert_eq!(json_rs::parse(" null "), Ok(JsonValue::Null));
    assert_eq!(json_rs::parse(" nulll"), Err(JsonError::InvalidValue));
    assert_eq!(json_rs::parse(" null n"), Err(JsonError::RootNotSingular));
}

#[test]
fn test_parse_bool() {
    assert_eq!(json_rs::parse("true"), Ok(JsonValue::Boolean(true)));
    assert_eq!(json_rs::parse(" true "), Ok(JsonValue::Boolean(true)));
    assert_eq!(json_rs::parse(" true t"), Err(JsonError::RootNotSingular));

    assert_eq!(json_rs::parse("false"), Ok(JsonValue::Boolean(false)));
    assert_eq!(json_rs::parse(" false "), Ok(JsonValue::Boolean(false)));
    assert_eq!(json_rs::parse(" false t"), Err(JsonError::RootNotSingular));
}
