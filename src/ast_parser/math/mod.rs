use crate::ast_parser::*;
use crate::NumberType;
use crate::{boxer, choice, sequence};

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
                let out_node @ Some(ASTNode::Number(_)) = list.pop() else {
                    panic!("IntParser did not return a number");
                };
                out_node
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
            OptionParser(sequence!(
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
                let Some(ASTNode::Sequence(mut inner_list)) = list.pop() else {
                    panic!("Parenthesized expression was not a number");
                };
                let Some(ASTNode::Number(exponent)) = inner_list.pop() else {
                    panic!("Parenthesized expression was not a number");
                };
                let Some(ASTNode::Number(base)) = list.pop() else {
                    panic!("Parenthesized expression was not a number");
                };
                ASTNode::Number(base.pow(exponent.try_into().map_err(|e| format!("{}", e))?))
            }
            _ => panic!("List should be 1 or 2 long!"),
        };
        Ok((output_string, Some(output_node)))
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
