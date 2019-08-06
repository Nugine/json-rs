#[macro_use]
extern crate lazy_static;

mod validate;

use self::validate::validate_number;

use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

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
}

pub type JsonResult<T> = Result<T, JsonError>;

pub fn parse(src: &str) -> JsonResult<JsonValue> {
    let mut ctx = JsonContext {
        chars: src.chars().peekable(),
    };

    ctx.parse_whitespace();

    ctx.chars
        .peek()
        .cloned()
        .ok_or(JsonError::ExpectValue)
        .and_then(|ch| match ch {
            'n' => ctx.parse_null(),
            't' => ctx.parse_true(),
            'f' => ctx.parse_false(),
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '-' => ctx.parse_number(),
            _ => Err(JsonError::InvalidValue),
        })
        .and_then(|v| {
            if let Some(ch) = ctx.chars.peek() {
                if let ' ' | '\t' | '\n' | '\r' = ch {
                    ctx.chars.next();
                    ctx.parse_whitespace();
                } else {
                    return Err(JsonError::InvalidValue);
                }
            }
            if ctx.chars.peek().is_none() {
                Ok(v)
            } else {
                Err(JsonError::RootNotSingular)
            }
        })
}

struct JsonContext<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> JsonContext<'a> {
    fn parse_whitespace(&mut self) {
        while let Some(ch) = self.chars.peek() {
            if let ' ' | '\t' | '\n' | '\r' = ch {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    #[inline(always)]
    fn parse_literal(s: &'static str) -> impl Fn(&mut JsonContext) -> JsonResult<()> {
        move |ctx| {
            let mut cnt = 0;
            let iter = ctx.chars.by_ref().take(s.len()).zip(s.chars());

            for (a, b) in iter {
                if a != b {
                    return Err(JsonError::InvalidValue);
                } else {
                    cnt += 1;
                }
            }

            if cnt != s.len() {
                Err(JsonError::InvalidValue)
            } else {
                Ok(())
            }
        }
    }

    fn parse_null(&mut self) -> JsonResult<JsonValue> {
        JsonContext::parse_literal("null")(self).map(|_| JsonValue::Null)
    }

    fn parse_true(&mut self) -> JsonResult<JsonValue> {
        JsonContext::parse_literal("true")(self).map(|_| JsonValue::Boolean(true))
    }

    fn parse_false(&mut self) -> JsonResult<JsonValue> {
        JsonContext::parse_literal("false")(self).map(|_| JsonValue::Boolean(false))
    }

    fn parse_number(&mut self) -> JsonResult<JsonValue> {
        let mut s = String::new();
        s.push(self.chars.next().unwrap());

        while let Some(&ch) = self.chars.peek() {
            if let '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '.' | 'e' | 'E'
            | '-' | '+' = ch
            {
                s.push(ch);
                self.chars.next();
            } else {
                break;
            }
        }

        if validate_number(&s) {
            Ok(JsonValue::Number(s.parse().unwrap()))
        } else {
            Err(JsonError::InvalidValue)
        }
    }
}
