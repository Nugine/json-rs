use std::collections::HashMap;
use std::ops::Index;
use std::ops::IndexMut;
use std::string::ToString;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum JsonError {
    RootNotSingular,
    InvalidValue,
    NumberTooBig,
    MissingColon,
    UnexpectedEnd,
}

pub type JsonResult<T> = Result<T, JsonError>;

impl JsonValue {
    fn stringify_string_raw(s: &str, buf: &mut String) {
        buf.push('\"');
        for ch in s.chars() {
            match ch {
                '\"' => buf.push_str(r#"\""#),
                '\\' => buf.push_str(r#"\\"#),
                '\u{8}' => buf.push_str(r#"\b"#),
                '\u{c}' => buf.push_str(r#"\f"#),
                '\n' => buf.push_str(r#"\n"#),
                '\r' => buf.push_str(r#"\r"#),
                '\t' => buf.push_str(r#"\t"#),
                ch if !is_unescaped_char(ch) => {
                    buf.push_str(r#"\u"#);
                    let mut t = ch as u32;
                    for _ in 0..4 {
                        let c = unsafe {
                            std::char::from_u32_unchecked(match (t >> 12) & 0xf {
                                t @ 0...10 => u32::from(b'0') + t,
                                t @ 10...16 => u32::from(b'a') + (t - 10),
                                _ => unreachable!(),
                            })
                        };
                        buf.push(c);
                        t >>= 4;
                    }
                }
                ch => buf.push(ch),
            }
        }
        buf.push('\"')
    }

    fn stringify_to_buf(&self, buf: &mut String) {
        match self {
            JsonValue::Null => buf.push_str("null"),
            JsonValue::Boolean(true) => buf.push_str("true"),
            JsonValue::Boolean(false) => buf.push_str("false"),
            JsonValue::Number(num) => buf.push_str(&num.to_string()),
            JsonValue::String(ref s) => JsonValue::stringify_string_raw(s, buf),
            JsonValue::Array(ref arr) => {
                buf.push('[');
                if let Some(first) = arr.first() {
                    first.stringify_to_buf(buf);
                    for val in &arr[1..] {
                        buf.push(',');
                        val.stringify_to_buf(buf);
                    }
                }
                buf.push(']')
            }
            JsonValue::Object(ref map) => {
                buf.push('{');
                let mut iter = map.iter();
                for (k, v) in iter.by_ref().take(1) {
                    JsonValue::stringify_string_raw(k, buf);
                    buf.push(':');
                    v.stringify_to_buf(buf);
                }
                for (k, v) in iter {
                    buf.push(',');
                    JsonValue::stringify_string_raw(k, buf);
                    buf.push(':');
                    v.stringify_to_buf(buf);
                }
                buf.push('}')
            }
        }
    }
}

#[inline(always)]
pub fn is_unescaped_char(ch: char) -> bool {
    let n = ch as u32;
    (0x20..=0x21).contains(&n) || (0x23..=0x5B).contains(&n) || (0x5D..=0x10_FFFF).contains(&n)
}

#[inline(always)]
pub fn is_whitespace(ch: char) -> bool {
    if let ' ' | '\t' | '\n' | '\r' = ch {
        true
    } else {
        false
    }
}

impl JsonValue {
    pub fn as_num(&self) -> Option<&f64> {
        if let JsonValue::Number(ref n) = self {
            Some(n)
        } else {
            None
        }
    }

    pub fn as_slice(&self) -> Option<&[JsonValue]> {
        if let JsonValue::Array(ref arr) = self {
            Some(arr.as_slice())
        } else {
            None
        }
    }

    pub fn as_map(&self) -> Option<&HashMap<String, JsonValue>> {
        if let JsonValue::Object(ref map) = self {
            Some(map)
        } else {
            None
        }
    }

    pub fn stringify(&self) -> String {
        let mut buf = String::new();
        self.stringify_to_buf(&mut buf);
        buf
    }
}

impl Index<usize> for JsonValue {
    type Output = JsonValue;

    fn index(&self, index: usize) -> &JsonValue {
        if let JsonValue::Array(ref arr) = self {
            &arr[index]
        } else {
            panic!("json value is not an array")
        }
    }
}

impl IndexMut<usize> for JsonValue {
    fn index_mut(&mut self, index: usize) -> &mut JsonValue {
        if let JsonValue::Array(ref mut arr) = self {
            &mut arr[index]
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

impl IndexMut<&str> for JsonValue {
    fn index_mut<'a>(&mut self, index: &'a str) -> &mut JsonValue {
        if let JsonValue::Object(ref mut map) = self {
            map.get_mut(index).expect("key not found")
        } else {
            panic!("json value is not an object")
        }
    }
}

impl ToString for JsonValue {
    fn to_string(&self) -> String {
        self.stringify()
    }
}
