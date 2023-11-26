use crate::ast_parser::math::infix::{AddSubtractParser, MultiplyDivideParser};
#[cfg(test)]
use crate::ast_parser::math::*;

#[test]
fn paren_does_not_get_literal() {
    assert!(ParenthesizedExpressionParser().parse("55").is_err());
}

#[test]
fn paren_gets_negative_literal() {
    assert!(ParenthesizedExpressionParser().parse("-55").is_err());
}

#[test]
fn paren_gets_paren_literal() {
    let actual = ParenthesizedExpressionParser().parse("(-55)a").unwrap();
    let expected = ("a", Some(ASTNode::Number(-55)));
    assert_eq!(actual, expected);
}

#[test]
fn expo_gets_literal() {
    let actual = ExponentParser().parse("2").unwrap();
    let expected = ("", Some(ASTNode::Number(2)));
    assert_eq!(actual, expected);
}

#[test]
fn expo_gets_parenthesized_exponent() {
    let actual = ExponentParser().parse("(2)^5").unwrap();
    let expected = ("", Some(ASTNode::Number(32)));
    assert_eq!(actual, expected);
}

#[test]
fn expo_gets_stacked_exponent() {
    let actual = ExponentParser().parse("(2)^3^2").unwrap();
    let expected = ("", Some(ASTNode::Number(64)));
    assert_eq!(actual, expected);
}

#[test]
fn multiplies_and_divides() {
    let cases: &[(&'static str, NumberType)] = &[
        ("5", 5),
        ("5 * 3", 15),
        ("5 * 4 * 2", 40),
        ("20 / 10 * 5", 10),
    ];
    for (s, expected) in cases {
        let actual = MultiplyDivideParser().parse(s).unwrap();
        let expected = ("", Some(ASTNode::Number(*expected)));
        assert_eq!(actual, expected);
    }
}

#[test]
fn adds_and_subtracts() {
    let cases: &[(&'static str, NumberType)] = &[
        ("5", 5),
        ("5 + 3", 8),
        ("5-3", 2),
        ("5 + 4 - 11", -2),
        ("-5 + 20", 15),
    ];
    for (s, expected) in cases {
        let actual = AddSubtractParser().parse(s).unwrap();
        let expected = ("", Some(ASTNode::Number(*expected)));
        assert_eq!(actual, expected);
    }
}

#[test]
fn adds_and_subtracts_with_complex_expressions() {
    let cases: &[(&'static str, NumberType)] = &[("5 - 2^5", -27), ("5 + 3*2", 11), ("5*2-3*2", 4)];
    for (s, expected) in cases {
        let actual = AddSubtractParser().parse(s).unwrap();
        let expected = ("", Some(ASTNode::Number(*expected)));
        assert_eq!(actual, expected);
    }
}

#[test]
fn adds_and_subtracts_a_lot() {
    let input_string = "-5 + 20";
    let output = 15;
    (1..200).for_each(|_| {
        // There's a bit of randomness in how the HashMap is arranged, and this test would have
        // caught a bug when we were using it
        let actual = AddSubtractParser().parse(input_string).unwrap();
        let expected = ("", Some(ASTNode::Number(output)));
        assert_eq!(actual, expected);
    })
}
