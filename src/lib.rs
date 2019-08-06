#[macro_use]
extern crate lazy_static;

mod types;
mod validate;

pub use self::types::{JsonError, JsonResult, JsonValue};
use self::validate::validate_number;

use std::iter::Peekable;
use std::str::Chars;

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
            c if c == '-' || c.is_digit(10) => ctx.parse_number(),
            '"' => ctx.parse_string(),
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
            if ch.is_digit(10) || ".eE-+".contains(ch) {
                s.push(ch);
                self.chars.next();
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
        let ch = self.chars.next().ok_or(JsonError::InvalidValue)?;
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
        self.chars.next();
        let mut s = String::new();

        while let Some(ch) = self.chars.next() {
            match ch {
                '"' => return Ok(JsonValue::String(s)),
                '\\' => s.push(self.parse_escape_char()?),
                c if is_unescaped_char(c) => s.push(c),
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
