#[macro_use]
extern crate lazy_static;

mod ast_parser;
mod rustifier;

use crate::ast_parser::*;
use crate::rustifier::*;
use std::collections::HashMap;

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
    let expected = JSONElement::Array(vec![
        JSONElement::String(String::from("Dog")),
        JSONElement::Number(2),
        JSONElement::Boolean(false),
        JSONElement::Array(vec![JSONElement::String(String::from("frank"))]),
        JSONElement::Object(HashMap::from([(
            String::from("sing"),
            JSONElement::Number(55),
        )])),
        JSONElement::Null,
    ]);
    assert_eq!(actual, expected);
}
