use json_rs::{JsonError, JsonValue};

macro_rules! expect {
    ($src:expr, $res:expr) => {
        assert_eq!(json_rs::parse($src), $res);
    };
}

macro_rules! expect_num {
    ($src:expr,$num:expr) => {
        expect!($src, Ok(JsonValue::Number($num)));
    };
}

macro_rules! expect_err {
    ($src:expr,$err:expr) => {
        expect!($src, Err($err));
    };
}

#[test]
fn test_parse() {
    expect!(" l ", Err(JsonError::InvalidValue));
    expect!(" ", Err(JsonError::ExpectValue));
    expect!("", Err(JsonError::ExpectValue));
}

#[test]
fn test_parse_null() {
    expect!("null", Ok(JsonValue::Null));
    expect!(" null ", Ok(JsonValue::Null));
    expect_err!("nul", JsonError::InvalidValue);
    expect!(" nulll", Err(JsonError::InvalidValue));
    expect!(" null n", Err(JsonError::RootNotSingular));
}

#[test]
fn test_parse_bool() {
    expect!("true", Ok(JsonValue::Boolean(true)));
    expect!(" true ", Ok(JsonValue::Boolean(true)));
    expect!(" true t", Err(JsonError::RootNotSingular));

    expect!("false", Ok(JsonValue::Boolean(false)));
    expect!(" false ", Ok(JsonValue::Boolean(false)));
    expect!(" false t", Err(JsonError::RootNotSingular));
}

#[test]
fn test_parse_num() {
    expect_num!("0", 0.0);
    expect_num!("0 ", 0.0);
    expect_num!(" 0 ", 0.0);
    expect_num!("-0", 0.0);
    expect_num!("-0.0", 0.0);
    expect_num!("1", 1.0);
    expect_num!("-1", -1.0);
    expect_num!("1.5", 1.5);
    expect_num!("-1.5", -1.5);
    expect_num!("3.1416", 3.1416);
    expect_num!("1E10", 1E10);
    expect_num!("1e10", 1e10);
    expect_num!("1E+10", 1E+10);
    expect_num!("1E-10", 1E-10);
    expect_num!("-1E10", -1E10);
    expect_num!("-1e10", -1e10);
    expect_num!("-1E+10", -1E+10);
    expect_num!("-1E-10", -1E-10);
    expect_num!("1.234E+10", 1.234E+10);
    expect_num!("1.234E-10", 1.234E-10);
    expect_num!("1e-10000", 0.0); /* must underflow */

    expect_err!("+0+ ", JsonError::InvalidValue);
    expect_err!("+0", JsonError::InvalidValue);
    expect_err!("+1", JsonError::InvalidValue);
    expect_err!(".123", JsonError::InvalidValue);
    expect_err!("1.", JsonError::InvalidValue);
    expect_err!("INF", JsonError::InvalidValue);
    expect_err!("inf", JsonError::InvalidValue);
    expect_err!("NAN", JsonError::InvalidValue);
    expect_err!("nan", JsonError::InvalidValue);

    /* the smallest number > 1 */
    expect_num!("1.0000000000000002", 1.0000000000000002);
    /* minimum denormal */
    expect_num!("4.9406564584124654e-324", 4.9406564584124654e-324);
    expect_num!("-4.9406564584124654e-324", -4.9406564584124654e-324);
    /* Max subnormal double */
    expect_num!("2.2250738585072009e-308", 2.2250738585072009e-308);
    expect_num!("-2.2250738585072009e-308", -2.2250738585072009e-308);
    /* Min normal positive double */
    expect_num!("2.2250738585072014e-308", 2.2250738585072014e-308);
    expect_num!("-2.2250738585072014e-308", -2.2250738585072014e-308);
    /* Max double */
    expect_num!("1.7976931348623157e+308", 1.7976931348623157e+308);
    expect_num!("-1.7976931348623157e+308", -1.7976931348623157e+308);
}
