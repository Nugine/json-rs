#[macro_use]
extern crate lazy_static;

mod types;
mod validate;

pub use self::types::{JsonError, JsonResult, JsonValue};
use self::validate::validate_number;

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
        let ch = self.peek().ok_or(JsonError::ExpectValue)?;

        let val = match ch {
            'n' => self.parse_null(),
            't' => self.parse_true(),
            'f' => self.parse_false(),
            '"' => self.parse_string(),
            '[' => self.parse_array(),
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
                match ctx.consume() {
                    None => return Err(JsonError::InvalidValue),
                    Some(a) => {
                        if a != b {
                            return Err(JsonError::InvalidValue);
                        }
                    }
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
        let mut i = 4;
        let mut ans: u16 = 0;
        for ch in self.chars.by_ref().take(4) {
            let t = ch.to_digit(16).ok_or(JsonError::InvalidValue)? as u16;
            ans = (ans << 4) | t;
            i -= 1;
        }
        if i == 0 {
            Ok(unsafe { std::char::from_u32_unchecked(ans as u32) })
        } else {
            Err(JsonError::InvalidValue)
        }
    }

    fn parse_escape_char(&mut self) -> JsonResult<char> {
        let ch = self.consume().ok_or(JsonError::InvalidValue)?;
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

    fn parse_string(&mut self) -> JsonResult<JsonValue> {
        self.consume();
        let mut s = String::new();

        while let Some(ch) = self.consume() {
            match ch {
                '"' => return Ok(JsonValue::String(s)),
                '\\' => s.push(self.parse_escape_char()?),
                c if is_unescaped_char(c) => s.push(c),
                _ => return Err(JsonError::InvalidValue),
            }
        }

        Err(JsonError::InvalidValue)
    }

    fn parse_array(&mut self) -> JsonResult<JsonValue> {
        self.consume();
        let mut arr = <Vec<JsonValue>>::new();
        self.parse_whitespace();
        match self.peek() {
            None => return Err(JsonError::InvalidValue),
            Some(']') => {
                self.consume();
                return Ok(JsonValue::Array(arr));
            }
            Some(_) => {
                arr.push(self.parse_value()?);
            }
        };

        while let Some(ch) = self.consume() {
            match ch {
                ',' => arr.push(self.parse_value()?),
                ']' => return Ok(JsonValue::Array(arr)),
                _ => return Err(JsonError::InvalidValue),
            }
        }

        Err(JsonError::InvalidValue)
    }
}

#[inline(always)]
fn is_unescaped_char(ch: char) -> bool {
    let n = ch as u32;
    (0x20..=0x21).contains(&n) || (0x23..=0x5B).contains(&n) || (0x5D..=0x10FFFF).contains(&n)
}

#[inline(always)]
fn is_whitespace(ch: char) -> bool {
    if let ' ' | '\t' | '\n' | '\r' = ch {
        true
    } else {
        false
    }
}
