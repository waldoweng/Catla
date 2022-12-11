extern crate pom;
use pom::parser::*;
use pom::Parser;

use std::collections::LinkedList;
use std::str::{self, FromStr};
use std::time::Duration;

#[derive(Debug, PartialEq, Clone)]
pub struct Tag {
    pub key: String,
    pub value: String,
}

fn new(k: String, v: String) -> Tag {
    Tag { key: k, value: v }
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Num(f64),
    Selector(String, Vec<Tag>, Duration),
    SubQuery {
        expr: Box<Expression>,
        offset: Duration,
        range: Duration,
        step: Duration,
    },
    BinaryOperation {
        operator: String,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    AggregationExpr {
        aggregator: String,
        group: Vec<String>,
        expr: Box<Expression>,
    },
    CallExpr {
        function: String,
        args: LinkedList<Expression>,
    },
}

fn space() -> Parser<u8, ()> {
    one_of(b" \t\r\n").repeat(0..).discard()
}

fn number() -> Parser<u8, f64> {
    let integer = one_of(b"123456789") - one_of(b"0123456789").repeat(0..) | sym(b'0');
    let frac = sym(b'.') + one_of(b"0123456789").repeat(1..);
    let exp = one_of(b"eE") + one_of(b"+-").opt() + one_of(b"0123456789").repeat(1..);
    let number = sym(b'-').opt() + integer + frac.opt() + exp.opt();
    number
        .collect()
        .convert(str::from_utf8)
        .convert(|s| f64::from_str(&s))
}

fn identifier() -> Parser<u8, String> {
    let integer = one_of(b"0123456789");
    let identifier = (one_of(b"abcdefghijklmnopqrstuvwxyz")
        | one_of(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"))
        + (one_of(b"abcdefghijklmnopqrstuvwxyz")
            | one_of(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ")
            | integer
            | sym(b'_'))
        .repeat(0..)
        - space();
    identifier
        .collect()
        .convert(str::from_utf8)
        .map(str::to_string)
}

fn string() -> Parser<u8, String> {
    let special_char = sym(b'\\')
        | sym(b'/')
        | sym(b'"')
        | sym(b'b').map(|_| b'\x08')
        | sym(b'f').map(|_| b'\x0C')
        | sym(b'n').map(|_| b'\n')
        | sym(b'r').map(|_| b'\r')
        | sym(b't').map(|_| b'\t');
    let escape_sequence = sym(b'\\') * special_char;
    let string = sym(b'"') * (none_of(b"\\\"") | escape_sequence).repeat(0..) - sym(b'"');
    string.convert(String::from_utf8)
}

fn tag_filter() -> Parser<u8, Tag> {
    (identifier() - sym(b'=') - space() + string()).map(|(k, v)| new(k, v)) - space()
}

fn operator() -> Parser<u8, String> {
    (one_of(b"+-/*%"))
        .collect()
        .convert(str::from_utf8)
        .map(str::to_string)
        - space()
}

fn selector() -> Parser<u8, (String, Vec<Tag>)> {
    ((identifier() - sym(b'{') - space() + list(space() * tag_filter(), sym(b',')) - sym(b'}'))
        | (sym(b'{').map(|_| "".to_string()) - space() + list(space() * tag_filter(), sym(b','))
            - sym(b'}'))
        | identifier().map(|id| (id, vec![])))
        - space()
}

fn binary() -> Parser<u8, ((Expression, String), Expression)> {
    call(expression) + operator() + call(expression) - space()
}

fn expression() -> Parser<u8, Expression> {
    (number().map(|num| Expression::Num(num))
        | selector().map(|(name, tags)| Expression::Selector(name, tags, Duration::from_secs(1)))
        | binary().map(|((lhs, op), rhs)| Expression::BinaryOperation {
            operator: op,
            left: Box::new(lhs),
            right: Box::new(rhs),
        }))
        - space()
}

pub fn promql() -> Parser<u8, Expression> {
    space() * expression() - end()
}

#[test]
fn operator_combinator() {
    let input = br"+ ";
    let operator_result = operator().parse(input);
    assert_eq!(Ok("+".to_string()), operator_result);
}

#[test]
fn identifier_combinator() {
    let input = br"node_cpu_usage_second";
    let identifier_result = identifier().parse(input);
    assert_eq!(Ok("node_cpu_usage_second".to_string()), identifier_result);
}

#[test]
fn tag_filter_combinator() {
    let input = br#"instance="127.0.0.1""#;
    let result = tag_filter().parse(input);
    assert_eq!(
        Ok(new("instance".to_string(), "127.0.0.1".to_string())),
        result
    )
}

#[test]
fn selector_combinator() {
    let input = br#"node_cpu_usage_second{instance="127.0.0.1"}"#;
    let result = selector().parse(input);
    assert_eq!(
        Ok((
            "node_cpu_usage_second".to_string(),
            vec![new("instance".to_string(), "127.0.0.1".to_string())]
        )),
        result
    )
}

#[test]
fn selector_combinator2() {
    let input = br#"{instance="127.0.0.1"}"#;
    let result = selector().parse(input);
    assert_eq!(
        Ok((
            "".to_string(),
            vec![new("instance".to_string(), "127.0.0.1".to_string())]
        )),
        result
    )
}

#[test]
fn selector_combinator3() {
    let input = br#"{instance="127.0.0.1", instance="127.0.0.2"}"#;
    let result = selector().parse(input);
    assert_eq!(
        Ok((
            "".to_string(),
            vec![
                new("instance".to_string(), "127.0.0.1".to_string()),
                new("instance".to_string(), "127.0.0.2".to_string())
            ]
        )),
        result
    )
}

#[test]
fn binary_combinator() {
    let input = br#"{instance="127.0.0.1"} / {instance="127.0.0.2"}"#;
    let result = binary().parse(input);
    assert_eq!(
        Ok((
            (
                Expression::Selector(
                    "".to_string(),
                    vec![new("instance".to_string(), "127.0.0.1".to_string())],
                    Duration::from_secs(1)
                ),
                "/".to_string()
            ),
            Expression::Selector(
                "".to_string(),
                vec![new("instance".to_string(), "127.0.0.2".to_string())],
                Duration::from_secs(1)
            )
        )),
        result
    )
}

#[test]
fn number_expression_combinator() {
    let input = br#"42"#;
    let result = expression().parse(input);
    assert_eq!(Ok(Expression::Num(42.0)), result)
}

#[test]
fn selector_expression_combinator() {
    let input = br#"{instance="127.0.0.1"}"#;
    let result = expression().parse(input);
    assert_eq!(
        Ok(Expression::Selector(
            "".to_string(),
            vec![new("instance".to_string(), "127.0.0.1".to_string())],
            Duration::from_secs(1)
        )),
        result
    )
}
