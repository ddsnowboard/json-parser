#[macro_use]
extern crate lazy_static;

use itertools::Itertools;
use std::cmp::min;
use std::collections::HashSet;

type NumberType = i32;

lazy_static! {
    static ref STRING_CHARACTERS: HashSet<char> =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890-@."
            .chars()
            .collect();
    static ref NUMBER_CHARACTERS: HashSet<char> = "1234567890".chars().collect();
}

macro_rules! boxer {
    ($type: expr, $($thing:expr);+) => {
        $type(vec![
        $(
             Box::new($thing)
        ),+
        ])
    }
}

macro_rules! choice {
    ($($thing:expr),+) => {
        boxer!(ChoiceParser, $($thing);+)
    }
}

macro_rules! sequence {
    ($($thing:expr),+) => {
        boxer!(SequenceParser, $($thing);+)
    }
}

#[derive(Debug)]
enum ASTNode<'i> {
    Number(NumberType),
    String(&'i str),
    Sequence(Vec<ASTNode<'i>>),
    Pair(Box<ASTNode<'i>>, Box<ASTNode<'i>>),
}

type ErrorType = String;
type ParseResult<'i> = Result<ParseOutput<'i>, ErrorType>;
type ParseOutput<'i> = (&'i str, Option<ASTNode<'i>>);

fn main() {
    let test_string = "\"apple\":123,";
    let test_string = "[\"apple\",123,567,\"beef\",[123,456,\"pants\"]]";
    let test_string = r#"{
            "pork": "prank",
            "frog": {"1": 1, "2": 2, "-2": -2, "three": "3"},
            "sing": -123,
            "frank": ["Ford", "BMW", "Fiat",-213   ,     204, [], ["apple", 200,[-5]]]
        }"#;
    let test_string = r#"{"employees":[  
    {"name":"Shyam", "email":"shyamjaiswal@gmail.com"},  
    {"name":"Bob", "email":"bob32@gmail.com"},  
    {"name":"Jai", "email":"jai87@gmail.com"}  
]}"#;
    let parser = choice!(ArrayParser(), ObjectParser());
    match parser.parse(test_string) {
        Ok((s, tree)) => println!("Rest of string is \"{}\", was {:?}", s, tree),
        Err(s) => println!("Error! {}", s),
    }
}

trait Parser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i>;
}

struct LiteralParser(&'static str);

impl Parser for LiteralParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        Ok((
            input.strip_prefix(self.0).ok_or_else(|| {
                format!(
                    "string \"{}\" did not start with \"{}\"",
                    &input[..min(input.len(), 10)],
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

struct WhitespaceParser();
impl Parser for WhitespaceParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        OptionParser(RepeatParser(choice!(
            LiteralParser(" "),
            LiteralParser("\n")
        )))
        .parse(input)
    }
}

struct SequenceParser(Vec<Box<dyn Parser>>);

impl Parser for SequenceParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        let mut output = Vec::new();
        let whitespace_parser = Box::new(WhitespaceParser()) as Box<dyn Parser>;
        let next_string: Result<&'i str, ErrorType> = self
            .0
            .iter()
            .intersperse(&whitespace_parser)
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

struct RepeatParser<T: Parser>(T);

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

struct StringParser();

impl Parser for StringParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        let delimeter_parser = LiteralParser("\"");
        let (string_start, _) = delimeter_parser.parse(input)?;
        let (string_end, node) = parse_character_string(string_start, &STRING_CHARACTERS);
        let (after_delimeter, _) = delimeter_parser.parse(string_end)?;
        Ok((after_delimeter, node))
    }
}

struct IntParser();

impl Parser for IntParser {
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
                &input[..min(10, input.len())]
            ))
        }
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

struct ChoiceParser(Vec<Box<dyn Parser>>);

impl Parser for ChoiceParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        self.0
            .iter()
            .filter_map(|parser| parser.parse(input).ok())
            .next()
            .ok_or_else(|| {
                format!(
                    "None of the options were satisfied at {}",
                    &input[..min(input.len(), 10)]
                )
            })
    }
}

struct CommaDelimitedSequenceParser<T: Parser>(T, &'static str, &'static str);

impl<T: Parser> Parser for CommaDelimitedSequenceParser<T> {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        let CommaDelimitedSequenceParser(element_parser, start_literal, end_literal) = self;
        let separator_parser =
            sequence!(WhitespaceParser(), LiteralParser(","), WhitespaceParser());
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

struct ArrayParser();

impl Parser for ArrayParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        CommaDelimitedSequenceParser(
            choice!(IntParser(), StringParser(), ArrayParser(), ObjectParser()),
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
            choice!(StringParser(), ArrayParser(), IntParser(), ObjectParser())
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

struct ObjectParser();
impl Parser for ObjectParser {
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i> {
        CommaDelimitedSequenceParser(KeyValueParser(), "{", "}").parse(input)
    }
}
