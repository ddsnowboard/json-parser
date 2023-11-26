use crate::ast_parser::math::ExponentParser;
use crate::ast_parser::*;
use crate::NumberType;
use crate::{boxer, sequence};

pub struct MultiplyDivideParser();

// The delimeter and a function returning the appropriate Operation enum for that delimeter
type OperationMapping<O> = (&'static str, fn(NumberType) -> O);

impl Parser for MultiplyDivideParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        let operation_mapping: Vec<OperationMapping<MultiplyDivideOperation>> = vec![
            (
                "*",
                MultiplyDivideOperation::Multiply as fn(i32) -> MultiplyDivideOperation,
            ),
            (
                "/",
                MultiplyDivideOperation::Divide as fn(i32) -> MultiplyDivideOperation,
            ),
        ];
        parse_infix_expression::<MultiplyDivideOperation, ExponentParser>(
            operation_mapping,
            input,
            ExponentParser,
        )
    }
}

pub struct AddSubtractParser();
impl Parser for AddSubtractParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        let operation_mapping: Vec<OperationMapping<AddSubtractOperation>> = vec![
            (
                "+",
                AddSubtractOperation::Add as fn(NumberType) -> AddSubtractOperation,
            ),
            (
                "-",
                AddSubtractOperation::Subtract as fn(NumberType) -> AddSubtractOperation,
            ),
        ];
        parse_infix_expression::<AddSubtractOperation, MultiplyDivideParser>(
            operation_mapping,
            input,
            MultiplyDivideParser,
        )
    }
}

fn try_with_delimeter<'i, P: Parser + 'static>(
    input: &'i str,
    delimeter: Option<&'static str>,
    get_component_parser: fn() -> P,
) -> Option<(&'i str, NumberType)> {
    let (next_string, Some(ASTNode::Sequence(mut exprs))) = match delimeter {
        Some(delimeter) => sequence!(LiteralParser(delimeter), get_component_parser()),
        None => sequence!(get_component_parser()),
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

trait Operation {
    fn execute(&self, accumulator: NumberType) -> NumberType;
}

enum MultiplyDivideOperation {
    Multiply(NumberType),
    Divide(NumberType),
}

impl Operation for MultiplyDivideOperation {
    fn execute(&self, accumulator: NumberType) -> NumberType {
        match self {
            Self::Multiply(n) => accumulator * n,
            Self::Divide(n) => accumulator / n,
        }
    }
}

enum AddSubtractOperation {
    Add(NumberType),
    Subtract(NumberType),
}

impl Operation for AddSubtractOperation {
    fn execute(&self, accumulator: NumberType) -> NumberType {
        match self {
            Self::Add(n) => accumulator + n,
            Self::Subtract(n) => accumulator - n,
        }
    }
}

fn parse_infix_expression<O: Operation, P: Parser + 'static>(
    operation_mapping: Vec<OperationMapping<O>>,
    input: &str,
    get_component_parser: fn() -> P,
) -> ParseResult<'_> {
    let mut operations = vec![];

    let (mut next_token, start_number) = try_with_delimeter(input, None, get_component_parser)
        .ok_or(format!(
            "string {} did not start with a number",
            prefix(input, 10)
        ))?;
    'token_walker: loop {
        for (delimeter, operation) in operation_mapping.iter() {
            if let Some((next_string, number)) =
                try_with_delimeter(next_token, Some(delimeter), get_component_parser)
            {
                next_token = next_string;
                operations.push(operation(number));
                continue 'token_walker;
            }
        }
        // If we get here, none of the options matched
        break;
    }
    let output_number = if !operations.is_empty() {
        operations
            .into_iter()
            .fold(start_number, |acc, op| op.execute(acc))
    } else {
        start_number
    };
    Ok((next_token, Some(ASTNode::Number(output_number))))
}
