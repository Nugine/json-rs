#[macro_use]
extern crate lazy_static;

mod ctx;
mod types;
mod validate;

pub use self::types::{JsonError, JsonResult, JsonValue};

use self::ctx::JsonContext;

pub fn parse(src: &str) -> JsonResult<JsonValue> {
    let mut ctx = JsonContext::new(src);

    let val = ctx.parse_value()?;

    if ctx.peek().is_none() {
        Ok(val)
    } else {
        Err(JsonError::RootNotSingular)
    }
}

pub fn stringify(value: &JsonValue) -> String {
    value.stringify()
}
