use crate::ast_parser::*;
use crate::NumberType;
use crate::{boxer, choice, sequence};
use std::cmp::min;

mod test;

struct IntLiteralParser();

impl Parser for IntLiteralParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        let (sign, numeral_start) = {
            let (string_start, _) = OptionParser(LiteralParser("-")).parse(input)?;
            // If we didn't move forward, then these will be equal and there was no negative sign
            (string_start == input, string_start)
        };

        let (next_string, Some(ASTNode::String(number))) =
            parse_character_string(numeral_start, &NUMBER_CHARACTERS)
        else {
            panic!("parse_character_string returned something other than a ASTNode::String!");
        };
        if !number.is_empty() {
            Ok((
                next_string,
                Some(ASTNode::Number(
                    number
                        .parse::<NumberType>()
                        .map_err(|err| format!("{}", err))?
                        * (if !sign { -1 } else { 1 }),
                )),
            ))
        } else {
            Err(format!(
                "{} did not start with an integer literal",
                prefix(input, 10)
            ))
        }
    }
}

struct ParenthesizedExpressionParser();
impl Parser for ParenthesizedExpressionParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        let (output_string, node) = choice!(
            IntLiteralParser(),
            sequence!(LiteralParser("("), IntParser(), LiteralParser(")"))
        )
        .parse(input)?;
        let output_node = match node {
            number @ Some(ASTNode::Number(_)) => number,
            Some(ASTNode::Sequence(mut list)) => {
                if let out_node @ Some(ASTNode::Number(_)) = list.pop() {
                    out_node
                } else {
                    panic!("IntParser did not return a number");
                }
            }
            _ => node,
        };
        Ok((output_string, output_node))
    }
}

struct ExponentParser();
impl Parser for ExponentParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        let (output_string, node) = sequence!(
            ParenthesizedExpressionParser(),
            RepeatParser(sequence!(
                LiteralParser("^"),
                ParenthesizedExpressionParser()
            ))
        )
        .parse(input)?;
        let Some(ASTNode::Sequence(mut list)) = node else {
            panic!("Sequence did not return a sequence");
        };
        let output_node = match list.len() {
            1 => list.pop().unwrap(),
            2 => {
                let Some(ASTNode::Sequence(stacked_exponents)) = list.pop() else {
                    panic!("Parenthesized expression was not a number");
                };
                let stacked_exponents: Vec<u32> = stacked_exponents
                    .into_iter()
                    .map(|node| {
                        if let ASTNode::Sequence(mut exprs) = node {
                            if let Some(ASTNode::Number(n)) = exprs.pop() {
                                n.try_into()
                            } else {
                                panic!("ParenthesizedExpressionParser() did not reutnr a number")
                            }
                        } else {
                            panic!("Sequence did not return a list")
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| format!("{:?}", e))?;
                let Some(ASTNode::Number(base)) = list.pop() else {
                    panic!("Parenthesized expression was not a number");
                };
                ASTNode::Number(
                    stacked_exponents
                        .into_iter()
                        .fold(base, |acc, exponent| acc.pow(exponent)),
                )
            }
            _ => panic!("List should be 1 or 2 long!"),
        };
        Ok((output_string, Some(output_node)))
    }
}

struct MultiplyDivideParser();
impl MultiplyDivideParser {
    fn try_with_delimeter<'i>(
        input: &'i str,
        delimeter: &'static str,
    ) -> Option<(&'i str, NumberType, NumberType)> {
        let (next_string, Some(ASTNode::Sequence(mut exprs))) = sequence!(
            ExponentParser(),
            LiteralParser(delimeter),
            MultiplyDivideParser()
        )
        .parse(input)
        .ok()?
        else {
            panic!("Sequence did not return sequence");
        };
        let Some(ASTNode::Number(right)) = exprs.pop() else {
            panic!("MultiplyDivideParser did not return a number")
        };
        let Some(ASTNode::Number(left)) = exprs.pop() else {
            panic!("MultiplyDivideParser did not return a number")
        };
        Some((next_string, left, right))
    }
}
impl Parser for MultiplyDivideParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        if let Some((next_string, left, right)) = Self::try_with_delimeter(input, "*") {
            Ok((next_string, Some(ASTNode::Number(left * right))))
        } else if let Some((next_string, left, right)) = Self::try_with_delimeter(input, "/") {
            Ok((next_string, Some(ASTNode::Number(left / right))))
        } else {
            ExponentParser().parse(input)
        }
    }
}

pub struct IntParser();
impl Parser for IntParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        IntLiteralParser().parse(input)
    }
}

fn unpack_number(node: ASTNode) -> NumberType {
    if let ASTNode::Number(n) = node {
        n
    } else {
        panic!("Got a {:?} instead of a Number", node);
    }
}
