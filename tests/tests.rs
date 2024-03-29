use json_rs::{JsonError, JsonValue};

macro_rules! expect {
    ($src:expr, $res:expr) => {
        assert_eq!(json_rs::parse($src), $res);
    };
}

macro_rules! expect_ok {
    ($src:expr,$val:expr) => {
        expect!($src, Ok($val));
    };
}

macro_rules! expect_err {
    ($src:expr,$err:expr) => {
        expect!($src, Err($err));
    };
}

macro_rules! expect_num {
    ($src:expr,$num:expr) => {
        expect_ok!($src, JsonValue::Number($num));
    };
}

macro_rules! expect_str {
    ($src:expr,$str:expr) => {
        expect_ok!($src, JsonValue::String($str.to_owned()));
    };
}

macro_rules! expect_array {
    ($src:expr,$slice:expr) => {{
        let val = json_rs::parse($src).expect("failed");
        assert_eq!(val.as_slice().expect("not an array"), $slice);
    }};
}

macro_rules! expect_val {
    ($src:expr, [$($path:expr$(,)?)+], $val:expr) => {{
        let val = json_rs::parse($src).expect("failed");
        assert_eq!(val$([$path])+, $val);
    }};
}

#[test]
fn test_parse() {
    expect_err!(" l ", JsonError::InvalidValue);
    expect_err!(" ", JsonError::UnexpectedEnd);
    expect_err!("", JsonError::UnexpectedEnd);
}

#[test]
fn test_parse_null() {
    expect_ok!("null", JsonValue::Null);
    expect_ok!(" null ", JsonValue::Null);
    expect_err!("nul", JsonError::UnexpectedEnd);
    expect_err!(" nulll", JsonError::InvalidValue);
    expect_err!(" null n", JsonError::RootNotSingular);
}

#[test]
fn test_parse_bool() {
    expect_ok!("true", JsonValue::Boolean(true));
    expect_ok!(" true ", JsonValue::Boolean(true));
    expect_err!(" true t", JsonError::RootNotSingular);

    expect_ok!("false", JsonValue::Boolean(false));
    expect_ok!(" false ", JsonValue::Boolean(false));
    expect_err!(" false t", JsonError::RootNotSingular);
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
    expect_err!("0123", JsonError::InvalidValue);
    expect_err!("0m", JsonError::InvalidValue);
    expect_err!("123.", JsonError::InvalidValue);

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

    expect_err!("1e+400", JsonError::NumberTooBig);
    expect_err!("-1e+400", JsonError::NumberTooBig);
}

#[test]
fn test_parse_str() {
    expect_err!(r#"""#, JsonError::UnexpectedEnd);
    expect_err!(r#""\""#, JsonError::UnexpectedEnd);
    expect_err!("\"\u{22}\"", JsonError::InvalidValue);
    expect_err!(r#""\u""#, JsonError::InvalidValue);
    expect_err!("   \"\"  \"\" ", JsonError::RootNotSingular);

    expect_str!(r#""\\""#, r#"\"#);
    expect_str!(r#""\t""#, "\t");
    expect_str!(r#""\n""#, "\n");
    expect_str!(r#""\u1234ab""#, "\u{1234}ab");
}

#[test]
fn test_parse_array() {
    use JsonValue::{Array, Null, Number};

    expect_array!(
        r#"[ null , false , true , 123 , "abc" ]"#,
        &[
            JsonValue::Null,
            JsonValue::Boolean(false),
            JsonValue::Boolean(true),
            JsonValue::Number(123.0),
            JsonValue::String("abc".to_owned())
        ]
    );

    expect_array!(
        r#"[ [ ] , [ 0 ] , [ 0 , 1 ] , [ 0 , 1 , 2 ] ]"#,
        &[
            Array(vec![]),
            Array(vec![Number(0.)]),
            Array(vec![Number(0.), Number(1.),]),
            Array(vec![Number(0.), Number(1.), Number(2.),]),
        ]
    );

    expect_array!("[null,[null]]\n", &[Null, Array(vec![Null])]);
    expect_array!("[null   ,   [null]]\n", &[Null, Array(vec![Null])]);
    expect_array!("[null\t,\t[null]]\n", &[Null, Array(vec![Null])]);

    expect_err!("[null     ,     [null,]]\n", JsonError::InvalidValue);

    expect_err!("[", JsonError::UnexpectedEnd);

    expect_err!("[nulll]", JsonError::InvalidValue);
}

#[test]
fn test_parse_object() {
    expect_err!("{:1,", JsonError::InvalidValue);
    expect_err!("{1:1,", JsonError::InvalidValue);
    expect_err!("{true:1,", JsonError::InvalidValue);
    expect_err!("{false:1,", JsonError::InvalidValue);
    expect_err!("{null:1,", JsonError::InvalidValue);
    expect_err!("{[]:1,", JsonError::InvalidValue);
    expect_err!("{{}:1,", JsonError::InvalidValue);
    expect_err!(r#"{"a":1]"#, JsonError::InvalidValue);
    expect_err!(r#"{"a":1 "b""#, JsonError::InvalidValue);

    expect_err!(r#"{"a"}"#, JsonError::MissingColon);
    expect_err!(r#"{"a","b"}"#, JsonError::MissingColon);

    expect_err!(r#"{"a":1,"#, JsonError::UnexpectedEnd);
    expect_err!(r#"{"a":{}"#, JsonError::UnexpectedEnd);
    expect_err!(r#"{"a":1"#, JsonError::UnexpectedEnd);

    expect_val!(r#"{"a":null,"b":null}"#, ["a"], JsonValue::Null);
    expect_val!(r#"{"a":{"b":null}}"#, ["a", "b"], JsonValue::Null);
}
