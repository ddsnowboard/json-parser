use crate::NumberType;
use itertools::Itertools;
use std::cmp::min;
use std::collections::HashSet;

mod math;
mod tests;
pub use crate::ast_parser::math::IntParser;

lazy_static! {
    static ref STRING_CHARACTERS: HashSet<char> =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890-@."
            .chars()
            .collect();
    static ref NUMBER_CHARACTERS: HashSet<char> = "1234567890".chars().collect();
}

#[macro_export]
macro_rules! boxer {
    ($type: expr, $($thing:expr);+) => {
        $type(vec![
        $(
             Box::new($thing)
        ),+
        ])
    }
}

#[macro_export]
macro_rules! choice {
    ($($thing:expr),+) => {
        boxer!($crate::ChoiceParser, $($thing);+)
    }
}

#[macro_export]
macro_rules! sequence {
    ($($thing:expr),+) => {
        boxer!($crate::SequenceParser, $($thing);+)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ASTNode<'i> {
    Number(NumberType),
    String(&'i str),
    Sequence(Vec<ASTNode<'i>>),
    Mapping(Vec<(ASTNode<'i>, ASTNode<'i>)>),
    Pair(Box<ASTNode<'i>>, Box<ASTNode<'i>>),
    Boolean(bool),
    Null,
}

fn make_pair<'a>(key: &'a str, value: ASTNode<'a>) -> ASTNode<'a> {
    ASTNode::Pair(Box::new(ASTNode::String(key)), Box::new(value))
}

fn prefix(s: &str, n: usize) -> &str {
    &s[..min(s.len(), n)]
}

type ErrorType = String;
type ParseResult<'i> = Result<ParseOutput<'i>, ErrorType>;
type ParseOutput<'i> = (&'i str, Option<ASTNode<'i>>);

pub trait Parser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i>;
}

struct LiteralParser(&'static str);

impl Parser for LiteralParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        Ok((
            input.strip_prefix(self.0).ok_or_else(|| {
                format!(
                    "string \"{}\" did not start with \"{}\"",
                    prefix(input, 10),
                    self.0
                )
            })?,
            None,
        ))
    }
}

struct OptionParser<T: Parser>(T);
impl<T: Parser> Parser for OptionParser<T> {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        let OptionParser(parser) = self;
        match parser.parse(input) {
            ret @ Ok(_) => ret,
            _ => Ok((input, None)),
        }
    }
}

pub struct WhitespaceParser();
impl Parser for WhitespaceParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        OptionParser(RepeatParser(choice!(
            LiteralParser(" "),
            LiteralParser("\n")
        )))
        .parse(input)
    }
}

pub struct SequenceParser(pub Vec<Box<dyn Parser>>);

impl Parser for SequenceParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        let mut output = Vec::new();
        let whitespace_parser = Box::new(WhitespaceParser()) as Box<dyn Parser>;
        let next_string: Result<&'i str, ErrorType> = Itertools::intersperse(
            self.0.iter(),
            &whitespace_parser,
        )
        .try_fold(input, |new_input, parser| {
            let (next_string, node) = parser.parse(new_input)?;
            if let Some(node) = node {
                output.push(node);
            }
            Ok(next_string)
        });
        Ok((next_string?, Some(ASTNode::Sequence(output))))
    }
}

pub struct RepeatParser<T: Parser>(T);

impl<T: Parser> Parser for RepeatParser<T> {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        let mut seq = Vec::new();
        let mut curr_string = input;
        while let Ok((nxt, node)) = self.0.parse(curr_string) {
            if let Some(node) = node {
                seq.push(node);
            }
            curr_string = nxt;
        }
        Ok((
            curr_string,
            if !seq.is_empty() {
                Some(ASTNode::Sequence(seq))
            } else {
                None
            },
        ))
    }
}

pub struct StringParser();

impl Parser for StringParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        let delimeter_parser = LiteralParser("\"");
        let (string_start, _) = delimeter_parser.parse(input)?;
        let (string_end, node) = parse_character_string(string_start, &STRING_CHARACTERS);
        let (after_delimeter, _) = delimeter_parser.parse(string_end)?;
        Ok((after_delimeter, node))
    }
}

fn parse_character_string<'a>(
    input: &'a str,
    available_characters: &'_ HashSet<char>,
) -> ParseOutput<'a> {
    let n_chars = input
        .chars()
        .take_while(|c| available_characters.contains(c))
        .count();
    (&input[n_chars..], Some(ASTNode::String(&input[..n_chars])))
}

pub struct ChoiceParser(pub Vec<Box<dyn Parser>>);

impl Parser for ChoiceParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        self.0
            .iter()
            .filter_map(|parser| parser.parse(input).ok())
            .next()
            .ok_or_else(|| {
                format!(
                    "None of the options were satisfied at {}",
                    prefix(input, 10)
                )
            })
    }
}

struct DelimitedSequenceParser<T: Parser>(&'static str, T, &'static str, &'static str);

impl<T: Parser> Parser for DelimitedSequenceParser<T> {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        let DelimitedSequenceParser(delimeter, element_parser, start_literal, end_literal) = self;
        let separator_parser = sequence!(
            WhitespaceParser(),
            LiteralParser(delimeter),
            WhitespaceParser()
        );
        // Turns out the empty array is a whole separate production in the real grammar, so I can
        // feel OK doing this
        // I used a sequence instead of just a single literal to allow for whitespace
        if let Ok((next, _)) =
            sequence!(LiteralParser(start_literal), LiteralParser(end_literal)).parse(input)
        {
            return Ok((next, Some(ASTNode::Sequence(vec![]))));
        }
        let (mut current_location, _) =
            sequence!(LiteralParser(start_literal), WhitespaceParser()).parse(input)?;

        let mut next_element = |include_comma: bool| -> Result<Option<ASTNode>, ErrorType> {
            if include_comma {
                current_location = &mut separator_parser.parse(current_location)?.0;
            }
            let (next_pointer, el) = element_parser.parse(current_location)?;
            current_location = next_pointer;
            Ok(el)
        };

        let mut elements = vec![];
        if let Some(el) = next_element(false)? {
            elements.push(el);
        }
        while let Ok(el) = next_element(true) {
            if let Some(el) = el {
                elements.push(el);
            }
        }
        let (after, _) =
            sequence!(WhitespaceParser(), LiteralParser(end_literal)).parse(current_location)?;
        Ok((after, Some(ASTNode::Sequence(elements))))
    }
}

pub struct ArrayParser();

impl Parser for ArrayParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        DelimitedSequenceParser(
            ",",
            choice!(
                BooleanParser(),
                IntParser(),
                StringParser(),
                ArrayParser(),
                ObjectParser(),
                NullParser()
            ),
            "[",
            "]",
        )
        .parse(input)
    }
}

struct KeyValueParser();
impl Parser for KeyValueParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        let (next_string, Some(ASTNode::Sequence(mut keyval))) = sequence!(
            StringParser(),
            LiteralParser(":"),
            choice!(
                StringParser(),
                ArrayParser(),
                IntParser(),
                ObjectParser(),
                BooleanParser(),
                NullParser()
            )
        )
        .parse(input)?
        else {
            panic!("Sequence did not return a sequence");
        };

        let value = keyval.pop();
        let key = keyval.pop();
        let (key, value) = key.zip(value).unwrap();
        Ok((
            next_string,
            Some(ASTNode::Pair(Box::new(key), Box::new(value))),
        ))
    }
}

pub struct ObjectParser();
impl Parser for ObjectParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        let (next_string, Some(ASTNode::Sequence(items))) =
            DelimitedSequenceParser(",", KeyValueParser(), "{", "}").parse(input)?
        else {
            panic!("DelimitedSequenceParser did not return a node");
        };
        Ok((
            next_string,
            Some(ASTNode::Mapping(
                items
                    .into_iter()
                    .map(|node| {
                        if let ASTNode::Pair(key, value) = node {
                            (*key, *value)
                        } else {
                            panic!("KeyValueParser did not return a vec of Pairs")
                        }
                    })
                    .collect(),
            )),
        ))
    }
}

pub struct BooleanParser();
impl Parser for BooleanParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        if let Ok((next_string, _)) = LiteralParser("true").parse(input) {
            Ok((next_string, Some(ASTNode::Boolean(true))))
        } else if let Ok((next_string, _)) = LiteralParser("false").parse(input) {
            Ok((next_string, Some(ASTNode::Boolean(false))))
        } else {
            Err(format!(
                "{} was neither \"true\" nor \"false\"",
                prefix(input, 10)
            ))
        }
    }
}

pub struct NullParser();
impl Parser for NullParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        if let Ok((next_string, _)) = LiteralParser("null").parse(input) {
            Ok((next_string, Some(ASTNode::Null)))
        } else {
            Err(format!("{} did not match \"null\"", prefix(input, 10)))
        }
    }
}
