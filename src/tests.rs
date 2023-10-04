#[cfg(test)]
mod numbers {
    use crate::*;

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
}

#[cfg(test)]
mod literals {
    use crate::*;
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
    use crate::*;
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
    use crate::*;

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
    use crate::*;

    #[test]
    fn parses_a_pair() {
        let test_string = "\"apple\":123,";
        let actual = KeyValueParser().parse(test_string).unwrap();
        let expected = (
            ",",
            Some(ASTNode::Pair(
                Box::new(ASTNode::String("apple")),
                Box::new(ASTNode::Number(123)),
            )),
        );
        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod sequence {
    use crate::*;

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
    use crate::*;

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
    use crate::*;

    #[test]
    fn parses_an_object_with_whitespace_and_objects() {
        let test_string = r#"{
            "pork": "prank",
            "frog": {"1": 1, "2": 2, "-2": -2, "three": "3"},
            "sing": -123,
            "frank": ["Ford", "BMW", "Fiat",-213   ,     204, [], ["apple", 200,[-5]]]
        }GREETINGS"#;
        let actual = ObjectParser().parse(test_string).unwrap();
        let expected = (
            "GREETINGS",
            Some(ASTNode::Sequence(vec![
                pair!("pork", ASTNode::String("prank")),
                pair!(
                    "frog",
                    ASTNode::Sequence(vec![
                        pair!("1", ASTNode::Number(1)),
                        pair!("2", ASTNode::Number(2)),
                        pair!("-2", ASTNode::Number(-2)),
                        pair!("three", ASTNode::String("3")),
                    ])
                ),
                pair!("sing", ASTNode::Number(-123)),
                pair!(
                    "frank",
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
                            ASTNode::Sequence(vec![ASTNode::Number(-5)])
                        ]),
                    ])
                ),
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
            Some(ASTNode::Sequence(vec![pair!(
                "employees",
                ASTNode::Sequence(vec![
                    ASTNode::Sequence(vec![
                        pair!("name", ASTNode::String("Shyam")),
                        pair!("email", ASTNode::String("shyamjaiswal@gmail.com")),
                    ]),
                    ASTNode::Sequence(vec![
                        pair!("name", ASTNode::String("Bob")),
                        pair!("email", ASTNode::String("bob32@gmail.com")),
                    ]),
                    ASTNode::Sequence(vec![
                        pair!("name", ASTNode::String("Jai")),
                        pair!("email", ASTNode::String("jai87@gmail.com")),
                    ]),
                ])
            )])),
        );
    }
}
