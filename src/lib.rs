#[macro_use]
extern crate lazy_static;

mod types;
mod validate;

pub use self::types::{JsonError, JsonResult, JsonValue};
use self::validate::validate_number;

use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

pub fn parse(src: &str) -> JsonResult<JsonValue> {
    let mut ctx = JsonContext::new(src);

    let val = ctx.parse_value()?;

    if ctx.peek().is_none() {
        Ok(val)
    } else {
        Err(JsonError::RootNotSingular)
    }
}

struct JsonContext<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> JsonContext<'a> {
    fn new(src: &'a str) -> Self {
        let chars = src.chars().peekable();
        Self { chars }
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().cloned()
    }

    #[cfg(not(test))]
    fn consume(&mut self) -> Option<char> {
        self.chars.next()
    }

    #[cfg(test)]
    fn consume(&mut self) -> Option<char> {
        self.chars.next().map(|ch| dbg!(ch))
    }

    fn parse_value(&mut self) -> JsonResult<JsonValue> {
        self.parse_whitespace();
        let ch = self.peek().ok_or(JsonError::UnexpectedEnd)?;

        let val = match ch {
            'n' => self.parse_null(),
            't' => self.parse_true(),
            'f' => self.parse_false(),
            '"' => self.parse_string(),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            c if c == '-' || c.is_digit(10) => self.parse_number(),
            _ => Err(JsonError::InvalidValue),
        }?;

        if let Some(ch) = self.peek() {
            #[cfg(test)]
            dbg!(ch);
            if !",]}".contains(ch) && !is_whitespace(ch) {
                return Err(JsonError::InvalidValue);
            }
        }

        self.parse_whitespace();
        Ok(val)
    }

    fn parse_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if is_whitespace(ch) {
                self.consume();
            } else {
                break;
            }
        }
    }

    #[inline(always)]
    fn parse_literal(s: &'static str) -> impl Fn(&mut JsonContext) -> JsonResult<()> {
        move |ctx| {
            for b in s.chars() {
                let a = ctx.consume().ok_or(JsonError::UnexpectedEnd)?;
                if a != b {
                    return Err(JsonError::InvalidValue);
                }
            }
            Ok(())
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
        s.push(self.consume().unwrap());

        while let Some(ch) = self.peek() {
            if ch.is_digit(10) || ".eE-+".contains(ch) {
                s.push(ch);
                self.consume();
            } else {
                break;
            }
        }

        if validate_number(&s) {
            let num: f64 = s.parse().expect("illegal float number");
            if !num.is_infinite() {
                Ok(JsonValue::Number(num))
            } else {
                Err(JsonError::NumberTooBig)
            }
        } else {
            Err(JsonError::InvalidValue)
        }
    }

    fn parse_hex4(&mut self) -> JsonResult<char> {
        let mut ans: u16 = 0;

        for _ in 0..4 {
            let ch = self.consume().ok_or(JsonError::UnexpectedEnd)?;
            let t = ch.to_digit(16).ok_or(JsonError::InvalidValue)? as u16;
            ans = (ans << 4) | t;
        }

        Ok(unsafe { std::char::from_u32_unchecked(u32::from(ans)) })
    }

    fn parse_escape_char(&mut self) -> JsonResult<char> {
        let ch = self.consume().ok_or(JsonError::UnexpectedEnd)?;
        match ch {
            '"' => Ok('"'),
            '\\' => Ok('\\'),
            '/' => Ok('/'),
            'b' => Ok('\u{8}'),
            'f' => Ok('\u{c}'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            'u' => self.parse_hex4(),
            _ => Err(JsonError::InvalidValue),
        }
    }

    fn parse_string_raw(&mut self) -> JsonResult<String> {
        if '"' != self.consume().ok_or(JsonError::UnexpectedEnd)? {
            return Err(JsonError::InvalidValue);
        }

        let mut s = String::new();

        loop {
            match self.consume().ok_or(JsonError::UnexpectedEnd)? {
                '"' => return Ok(s),
                '\\' => s.push(self.parse_escape_char()?),
                c if is_unescaped_char(c) => s.push(c),
                _ => return Err(JsonError::InvalidValue),
            }
        }
    }

    fn parse_string(&mut self) -> JsonResult<JsonValue> {
        self.parse_string_raw().map(JsonValue::String)
    }

    fn parse_array(&mut self) -> JsonResult<JsonValue> {
        self.consume();
        self.parse_whitespace();

        let mut arr = <Vec<JsonValue>>::new();
        match self.peek().ok_or(JsonError::UnexpectedEnd)? {
            ']' => {
                self.consume();
                return Ok(JsonValue::Array(arr));
            }
            _ => {
                arr.push(self.parse_value()?);
            }
        };

        loop {
            match self.consume().ok_or(JsonError::UnexpectedEnd)? {
                ',' => arr.push(self.parse_value()?),
                ']' => return Ok(JsonValue::Array(arr)),
                _ => return Err(JsonError::InvalidValue),
            }
        }
    }

    fn parse_kv(&mut self) -> JsonResult<(String, JsonValue)> {
        let k = self.parse_string_raw()?;
        self.parse_whitespace();
        match self.consume().ok_or(JsonError::UnexpectedEnd)? {
            ':' => {
                let v = self.parse_value()?;
                Ok((k, v))
            }
            _ => Err(JsonError::MissingColon),
        }
    }

    fn parse_object(&mut self) -> JsonResult<JsonValue> {
        self.consume();
        self.parse_whitespace();

        let mut map = <HashMap<String, JsonValue>>::new();

        match self.peek().ok_or(JsonError::UnexpectedEnd)? {
            '}' => {
                self.consume();
                return Ok(JsonValue::Object(map));
            }
            _ => {
                let (k, v) = self.parse_kv()?;
                map.insert(k, v);
            }
        };

        loop {
            match self.consume().ok_or(JsonError::UnexpectedEnd)? {
                ',' => {
                    let (k, v) = self.parse_kv()?;
                    map.insert(k, v);
                }
                '}' => {
                    return Ok(JsonValue::Object(map));
                }
                _ => return Err(JsonError::InvalidValue),
            }
        }
    }
}

#[inline(always)]
fn is_unescaped_char(ch: char) -> bool {
    let n = ch as u32;
    (0x20..=0x21).contains(&n) || (0x23..=0x5B).contains(&n) || (0x5D..=0x10_FFFF).contains(&n)
}

#[inline(always)]
fn is_whitespace(ch: char) -> bool {
    if let ' ' | '\t' | '\n' | '\r' = ch {
        true
    } else {
        false
    }
}
