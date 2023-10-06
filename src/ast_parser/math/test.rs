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
