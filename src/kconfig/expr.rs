use super::{
    Symbol,
    Hex,
    Int,
    util::{
        parse_kstring,
        special_space,
    },
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
pub enum VarType<'a> {
    Hex(Hex),
    Int(Int),
    Str(&'a str),
    Symbol(Symbol<'a>),
}

#[derive(Debug, PartialEq)]
pub enum Expr<'a> {
    Var(VarType<'a>),
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

fn take_operation(input: &str) -> IResult<&str, VarType> {
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
            map(Hex::parse,    |v| VarType::Hex(v)),
            map(Int::parse,    |v| VarType::Int(v)),
            map(parse_kstring, |v| VarType::Str(v)),
            map(Symbol::parse, |v| VarType::Symbol(v)),
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
                    recognize(Symbol::parse),
                    recognize(Hex::parse),
                    recognize(Int::parse),
                    take_parens,
                    parse_kstring,
                )),
                opt(alt((
                    take_state,
                    recognize(take_operation),
                ))),
            ))
        ),
        |var_name: &str| Expr::Var(VarType::Str(var_name)),
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
