#[macro_use]
extern crate lazy_static;

mod ast_parser;
mod rustifier;

use crate::ast_parser::*;
use crate::rustifier::*;

type NumberType = i32;

fn main() {
    let actual = loads(
        r#"[
    "Dog",
    2, 
    false, 
    ["frank"], 
    {"sing": 55},
    null,
    ]"#,
    )
    .unwrap();
    let expected = JSONElement::Array(vec![]);
    assert_eq!(actual, expected);
}
