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

type JsonResult<T> = Result<T, JsonError>;

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
            ctx.chars
                .by_ref()
                .take(s.len())
                .zip(s.chars())
                .try_for_each(|(a, b)| {
                    if a != b {
                        Err(JsonError::InvalidValue)
                    } else {
                        Ok(())
                    }
                })
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
}
