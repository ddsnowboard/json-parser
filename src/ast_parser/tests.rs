#[cfg(test)]
mod numbers {
    use crate::ast_parser::math::infix::AddSubtractParser;
    use crate::ast_parser::*;

    #[test]
    fn parses_positive_number() {
        let actual = IntParser().parse("123,456").unwrap();
        let expected = (",456", Some(ASTNode::Number(123)));
        assert_eq!(actual, expected);
    }

    #[test]
    fn parses_negative_number() {
        let actual = IntParser().parse("-123,456").unwrap();
        let expected = (",456", Some(ASTNode::Number(-123)));
        assert_eq!(actual, expected);
    }

    #[test]
    fn fails_for_bare_negative_sign() {
        let Err(_) = IntParser().parse("-") else {
            panic!("Should have returned Err");
        };
    }

    #[test]
    fn fails_for_empty_string() {
        let Err(_) = IntParser().parse("-") else {
            panic!("Should have returned Err");
        };
    }

    #[test]
    fn parses_math() {
        let actual = AddSubtractParser()
            .parse("(200 * (55+11) - 4^2)^2")
            .unwrap();
        let expected = ("", Some(ASTNode::Number(173817856)));
        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod literals {
    use crate::ast_parser::*;
    fn check_passes(start_string: &str, end_string: &str, literal: &'static str) {
        let actual = LiteralParser(literal).parse(start_string).unwrap();
        let expected = (end_string, None);
        assert_eq!(actual, expected);
    }

    #[test]
    fn parses_single_character_literal() {
        check_passes("APPLE", "PPLE", "A");
    }

    #[test]
    fn parses_multi_character_literal() {
        check_passes("APPLE", "LE", "APP");
    }

    #[test]
    fn fails() {
        let Err(_) = LiteralParser("FRANK").parse("GEORGE") else {
            panic!("Should have failed");
        };
    }
}

#[cfg(test)]
mod option {
    use crate::ast_parser::*;
    fn check_matches(start_string: &str, end_string: &str, number: NumberType) {
        let actual = OptionParser(IntParser()).parse(start_string).unwrap();
        let expected = (end_string, Some(ASTNode::Number(number)));
        assert_eq!(actual, expected);
    }

    #[test]
    fn check_takes_one() {
        check_matches("500", "", 500);
    }

    #[test]
    fn check_takes_only_one() {
        check_matches("500-50", "-50", 500);
    }

    #[test]
    fn check_returns_success_when_absent() {
        let string = "GEORGE";
        let actual = OptionParser(IntParser()).parse(string).unwrap();
        let expected = (string, None);
        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod whitespace {
    use crate::ast_parser::*;

    fn check(start_string: &str, end_string: &str) {
        let actual = WhitespaceParser().parse(start_string).unwrap();
        let expected = (end_string, None);
        assert_eq!(actual, expected);
    }

    #[test]
    fn takes_one_space() {
        check(" a", "a");
    }

    #[test]
    fn takes_many_spaces() {
        check("    a", "a");
    }

    #[test]
    fn takes_newline() {
        check("\n\na", "a");
    }

    #[test]
    fn takes_mixed() {
        check("\n  \n  a", "a");
    }

    #[test]
    fn passes_on_nothing() {
        check("a", "a");
    }
}

#[cfg(test)]
mod keyvalue {
    use crate::ast_parser::*;

    #[test]
    fn parses_a_pair() {
        let test_string = "\"apple\":123,";
        let actual = KeyValueParser().parse(test_string).unwrap();
        let expected = (",", Some(make_pair("apple", ASTNode::Number(123))));
        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod sequence {
    use crate::ast_parser::*;
    use crate::{boxer, sequence};

    #[test]
    fn parses_a_pair() {
        let test_string = "\"apple\":123,";
        let actual = sequence!(KeyValueParser(), LiteralParser(","))
            .parse(test_string)
            .unwrap();
        let expected = (
            "",
            Some(ASTNode::Sequence(vec![ASTNode::Pair(
                Box::new(ASTNode::String("apple")),
                Box::new(ASTNode::Number(123)),
            )])),
        );
        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod array {
    use crate::ast_parser::*;

    #[test]
    fn parses_an_array() {
        let test_string = "[\"apple\",123,567,\"beef\",[123,456,\"pants\"]]";
        let actual = ArrayParser().parse(test_string).unwrap();
        let expected = (
            "",
            Some(ASTNode::Sequence(vec![
                ASTNode::String("apple"),
                ASTNode::Number(123),
                ASTNode::Number(567),
                ASTNode::String("beef"),
                ASTNode::Sequence(vec![
                    ASTNode::Number(123),
                    ASTNode::Number(456),
                    ASTNode::String("pants"),
                ]),
            ])),
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn parses_an_array_with_trailing_comma() {
        let test_string = "[\"apple\",123,567,\"beef\",[123,456,\"pants\"],]";
        let actual = ArrayParser().parse(test_string).unwrap();
        let expected = (
            "",
            Some(ASTNode::Sequence(vec![
                ASTNode::String("apple"),
                ASTNode::Number(123),
                ASTNode::Number(567),
                ASTNode::String("beef"),
                ASTNode::Sequence(vec![
                    ASTNode::Number(123),
                    ASTNode::Number(456),
                    ASTNode::String("pants"),
                ]),
            ])),
        );
        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod object {
    use crate::ast_parser::*;

    #[test]
    fn parses_an_object_with_whitespace_and_objects() {
        let test_string = r#"{
            "pork": "prank",
            "frog": {"1": 1, "2": 2, "-2": -2, "three": "3"},
            "sing": -123,
            "frank": ["Ford", "BMW", "Fiat",-213   ,     204, [], ["apple", 200,[-5]]],
            "song": false,
            "nothing": null
        }GREETINGS"#;
        let actual = ObjectParser().parse(test_string).unwrap();
        let expected = (
            "GREETINGS",
            Some(ASTNode::Mapping(vec![
                (ASTNode::String("pork"), ASTNode::String("prank")),
                (
                    ASTNode::String("frog"),
                    ASTNode::Mapping(vec![
                        (ASTNode::String("1"), ASTNode::Number(1)),
                        (ASTNode::String("2"), ASTNode::Number(2)),
                        (ASTNode::String("-2"), ASTNode::Number(-2)),
                        (ASTNode::String("three"), ASTNode::String("3")),
                    ]),
                ),
                (ASTNode::String("sing"), ASTNode::Number(-123)),
                (
                    ASTNode::String("frank"),
                    ASTNode::Sequence(vec![
                        ASTNode::String("Ford"),
                        ASTNode::String("BMW"),
                        ASTNode::String("Fiat"),
                        ASTNode::Number(-213),
                        ASTNode::Number(204),
                        ASTNode::Sequence(vec![]),
                        ASTNode::Sequence(vec![
                            ASTNode::String("apple"),
                            ASTNode::Number(200),
                            ASTNode::Sequence(vec![ASTNode::Number(-5)]),
                        ]),
                    ]),
                ),
                (ASTNode::String("song"), ASTNode::Boolean(false)),
                (ASTNode::String("nothing"), ASTNode::Null),
            ])),
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn parses_an_object_with_an_array() {
        let test_string = r#"{"employees":[  
    {"name":"Shyam", "email":"shyamjaiswal@gmail.com"},  
    {"name":"Bob", "email":"bob32@gmail.com"},  
    {"name":"Jai", "email":"jai87@gmail.com"}  
]}"#;
        let actual = ObjectParser().parse(test_string).unwrap();
        let expected = (
            "",
            Some(ASTNode::Mapping(vec![(
                ASTNode::String("employees"),
                ASTNode::Sequence(vec![
                    ASTNode::Mapping(vec![
                        (ASTNode::String("name"), ASTNode::String("Shyam")),
                        (
                            ASTNode::String("email"),
                            ASTNode::String("shyamjaiswal@gmail.com"),
                        ),
                    ]),
                    ASTNode::Mapping(vec![
                        (ASTNode::String("name"), ASTNode::String("Bob")),
                        (ASTNode::String("email"), ASTNode::String("bob32@gmail.com")),
                    ]),
                    ASTNode::Mapping(vec![
                        (ASTNode::String("name"), ASTNode::String("Jai")),
                        (ASTNode::String("email"), ASTNode::String("jai87@gmail.com")),
                    ]),
                ]),
            )])),
        );
        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod boolean {
    use crate::ast_parser::*;

    fn check(string: &str, value: bool) {
        let actual = BooleanParser().parse(string).unwrap();
        let expected = ("", Some(ASTNode::Boolean(value)));
        assert_eq!(actual, expected);
    }

    #[test]
    fn parses_true() {
        check("true", true);
    }

    #[test]
    fn parses_false() {
        check("false", false);
    }
}

#[cfg(test)]
mod null {
    use crate::ast_parser::*;

    #[test]
    fn parses_null() {
        let actual = NullParser().parse("null").unwrap();
        let expected = ("", Some(ASTNode::Null));
        assert_eq!(actual, expected);
    }
}
