#[macro_use]
extern crate lazy_static;

mod ast_parser;

use crate::ast_parser::*;

fn main() {
    println!("HI{:?}", ASTNode::Number(5));
}
