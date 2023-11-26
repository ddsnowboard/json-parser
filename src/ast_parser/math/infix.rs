use crate::ast_parser::math::ExponentParser;
use crate::ast_parser::*;
use crate::NumberType;
use crate::{boxer, sequence};
use std::collections::HashMap;

pub struct MultiplyDivideParser();
impl MultiplyDivideParser {}

impl Parser for MultiplyDivideParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        enum Operation {
            Start(NumberType),
            Multiply(NumberType),
            Divide(NumberType),
        }

        let operation_mapping = HashMap::from([
            (None, Operation::Start as fn(i32) -> Operation),
            (Some("*"), Operation::Multiply as fn(i32) -> Operation),
            (Some("/"), Operation::Divide as fn(i32) -> Operation),
        ]);

        let mut operations = vec![];
        let mut next_token = input;
        'token_walker: loop {
            for (delimeter, operation) in operation_mapping.iter() {
                if let Some((next_string, number)) = try_with_delimeter(next_token, *delimeter) {
                    next_token = next_string;
                    operations.push(operation(number));
                    continue 'token_walker;
                }
            }
            // If we get here, none of the options matched
            break;
        }
        if !operations.is_empty() {
            let output_number = operations.into_iter().fold(1, |acc, op| match op {
                Operation::Start(n) => n,
                Operation::Multiply(n) => n * acc,
                Operation::Divide(n) => acc / n,
            });
            Ok((next_token, Some(ASTNode::Number(output_number))))
        } else {
            Err(format!(
                "string \"{}\" did not start with an exponential expression",
                prefix(input, 10)
            ))
        }
    }
}

fn try_with_delimeter<'i>(
    input: &'i str,
    delimeter: Option<&'static str>,
) -> Option<(&'i str, NumberType)> {
    let (next_string, Some(ASTNode::Sequence(mut exprs))) = match delimeter {
        Some(delimeter) => sequence!(LiteralParser(delimeter), ExponentParser()),
        None => sequence!(ExponentParser()),
    }
    .parse(input)
    .ok()?
    else {
        panic!("Sequence did not return sequence");
    };
    let Some(ASTNode::Number(number)) = exprs.pop() else {
        panic!("MultiplyDivideParser did not return a number")
    };
    Some((next_string, number))
}
