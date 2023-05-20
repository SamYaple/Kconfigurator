use super::util::{
    parse_kstring,
    special_space,
    take_hex,
    take_name,
    take_signed_int,
};

use nom::{
    IResult,
    bytes::complete::{
        tag,
        take_while1,
    },
    combinator::{
        opt,
        map,
        recognize,
    },
    sequence::{
        delimited,
        preceded,
        tuple,
    },
    branch::alt,
    multi::many0,
};

#[derive(Debug, PartialEq)]
pub enum Expr<'a> {
    Var(&'a str),
    Not(Box<Expr<'a>>),
    And(Box<Expr<'a>>, Box<Expr<'a>>),
    Or(Box<Expr<'a>>, Box<Expr<'a>>),
}

fn take_parens(input: &str) -> IResult<&str, &str> {
    let (input, ret) = recognize(delimited(
        alt((tag("$("), tag("("))),
        recognize(many0(alt((
            take_parens,
            take_while1(|c| c != '$' && c != '(' && c != ')' && c != '\\'),
            alt((
                tag("\\$"),
                tag("\\("),
                tag("\\)"),
                tag("\\"),
                tag("$"),
                tag("("),
            )),
        )))),
        tag(")"),
    ))(input)?;
    Ok((input, ret))
}

fn take_operation(input: &str) -> IResult<&str, &str> {
    preceded(
        tuple((
            special_space,
            alt((
                tag("<="),
                tag(">="),
                tag("!="),
                tag("<"),
                tag(">"),
                tag("="),
            )),
            special_space,
        )),
        alt((
            take_hex,        // opttype: `hex`
            take_signed_int, // opttype: `int`
            parse_kstring,
            take_name, // TODO: This should take a kstring, hex, or bool...
        )),
    )(input)
}

fn take_state(input: &str) -> IResult<&str, &str> {
    preceded(
        tag("="),
        alt((
            tag("y"),
            tag("m"),
            tag("n"),
        )),
    )(input)
}

fn var(input: &str) -> IResult<&str, Expr> {
    map(
        recognize(
            tuple((
                alt((
                    take_name,
                    take_hex,        // opttype: `hex`
                    take_signed_int, // opttype: `int`
                    take_parens,
                    parse_kstring,
                )),
                opt(alt((
                    take_state,
                    take_operation,
                ))),
            ))
        ),
        |var_name: &str| Expr::Var(var_name),
    )(input)
}

fn parens(input: &str) -> IResult<&str, Expr> {
    delimited(
        tuple((special_space, tag("("), special_space)),
        parse_expr,
        tuple((special_space, tag(")"), special_space)),
    )(input)
}

fn factor(input: &str) -> IResult<&str, Expr> {
    alt((
        var,
        parens,
        map(
            preceded(tuple((special_space, tag("!"), special_space)), factor),
            |e| Expr::Not(Box::new(e)),
        ),
    ))(input)
}

fn term(input: &str) -> IResult<&str, Expr> {
    let (input, init) = factor(input)?;
    let (input, terms) = many0(
        preceded(tuple((special_space, tag("&&"), special_space)), term)
    )(input)?;

    let result = terms.into_iter().fold(init, |acc, e| Expr::And(Box::new(acc), Box::new(e)));

    Ok((input, result))
}

pub fn parse_expr(input: &str) -> IResult<&str, Expr> {
    let (input, init) = term(input)?;
    let (input, terms) = many0(
        preceded(tuple((special_space, tag("||"), special_space)), term)
    )(input)?;

    let result = terms.into_iter().fold(init, |acc, e| Expr::Or(Box::new(acc), Box::new(e)));

    Ok((input, result))
}
