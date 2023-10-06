use crate::ast_parser;
use crate::ast_parser::ASTNode;
use crate::ast_parser::Parser;
use crate::boxer;
use crate::NumberType;
use crate::{choice, sequence};
use std::collections::HashMap;

mod tests;

pub fn loads(s: &str) -> Result<JSONElement, String> {
    let (rest_of_string, node) = sequence!(
        ast_parser::WhitespaceParser(),
        choice!(
            ast_parser::ArrayParser(),
            ast_parser::IntParser(),
            ast_parser::BooleanParser(),
            ast_parser::StringParser(),
            ast_parser::NullParser(),
            ast_parser::ObjectParser()
        ),
        ast_parser::WhitespaceParser()
    )
    .parse(s)?;
    if !rest_of_string.is_empty() {
        Err(format!("Trailing data: {}", rest_of_string))
    } else if let Some(ASTNode::Sequence(mut container)) = node {
        // We have to extract it from the
        // sequence we used to consume the whitespace
        convert(&container.pop().unwrap())
    } else {
        Ok(JSONElement::Null)
    }
}

fn convert(node: &ASTNode) -> Result<JSONElement, String> {
    match node {
        ASTNode::Number(n) => Ok(JSONElement::Number(*n)),
        ASTNode::String(s) => Ok(JSONElement::String(s.to_string())),
        ASTNode::Boolean(b) => Ok(JSONElement::Boolean(*b)),
        ASTNode::Pair(_, _) => Err("Can't have top-level pair".to_string()),
        ASTNode::Sequence(items) => Ok(JSONElement::Array(
            items
                .iter()
                .map(|item| convert(item))
                .collect::<Result<Vec<_>, String>>()?,
        )),
        ASTNode::Mapping(pairs) => {
            let keyvals: Result<HashMap<String, JSONElement>, String> = pairs
                .iter()
                .map(|(key, value)| {
                    if let ASTNode::String(s) = key {
                        Ok((s.to_string(), convert(value)?))
                    } else {
                        Err(format!("Key {:?} was not a string", key))
                    }
                })
                .collect();
            keyvals.map(JSONElement::Object)
        }
        ASTNode::Null => Ok(JSONElement::Null),
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum JSONElement {
    Object(HashMap<String, JSONElement>),
    Array(Vec<JSONElement>),
    String(String),
    Number(NumberType),
    Boolean(bool),
    Null,
}
