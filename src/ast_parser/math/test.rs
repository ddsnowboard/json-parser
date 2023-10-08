#[cfg(test)]
use crate::ast_parser::math::*;

#[test]
fn paren_gets_literal() {
    let actual = ParenthesizedExpressionParser().parse("55a").unwrap();
    let expected = ("a", Some(ASTNode::Number(55)));
    assert_eq!(actual, expected);
}

#[test]
fn paren_gets_negative_literal() {
    let actual = ParenthesizedExpressionParser().parse("-55a").unwrap();
    let expected = ("a", Some(ASTNode::Number(-55)));
    assert_eq!(actual, expected);
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
        // ("5", 5),
        // ("5 * 3", 15),
        // ("5 * 4 * 2", 40),
        ("20 / 10 * 5", 10),
    ];
    for (s, expected) in cases {
        let actual = MultiplyDivideParser().parse(s).unwrap();
        let expected = ("", Some(ASTNode::Number(*expected)));
        assert_eq!(actual, expected);
    }
}
