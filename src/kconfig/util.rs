use super::{
    KConfig,
    expr::{
        parse_expr,
    }
};

use nom::{
    branch::alt,
    bytes::complete::{
        is_a,
        tag,
        take,
        take_until,
        take_while1,
    },
    character::complete::{
        anychar,
        line_ending,
        satisfy,
        space0,
        space1,
    },
    combinator::{
        map,
        not,
        peek,
        recognize,
        opt,
    },
    multi::{
        many0,
        many1,
        many_till,
    },
    sequence::{
        preceded,
        tuple,
        delimited,
        terminated,
    },
    IResult,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum OptionType {
    Tristate,
    Bool,
    Hex,
    Int,
    Str,
}

impl OptionType {
    pub fn parse(input: &str) -> IResult<&str, OptionType> {
        let (input, _) = space0(input)?;
        alt((
            map(tag("tristate"), |_| OptionType::Tristate),
            map(tag("bool"),     |_| OptionType::Bool),
            map(tag("hex"),      |_| OptionType::Hex),
            map(tag("int"),      |_| OptionType::Int),
            map(tag("string"),   |_| OptionType::Str),
        ))(input)
    }
}

#[derive(Debug, PartialEq)]
pub enum Delimiter<'a> {
    SingleQuote(&'a str),
    DoubleQuote(&'a str),
    //Parentheses(&'a str),
    //DollarParentheses(&'a str),
}

#[derive(Debug, PartialEq)]
pub struct ConstantSymbol<'a> {
    pub delimiter: Delimiter<'a>,

    // Once the string has been processed through macros, it should not be reprocessed.
    processed:     bool,
}

impl<'a> ConstantSymbol<'a> {
    pub fn parse(input: &'a str) -> IResult<&'a str, Self> {
        // Grab the matching delimiter tag, but do not move the input position forward. This allows
        // for better readability later when we call the `delimited` parser instead of using the
        // `terminated` parser.
        // The types being included in the variable names is to provide visibility to anyone
        // reading this code. For ease of use when parsing, `end_delim` must be a `char`, but the
        // `begin_delim` is more than one `char` in the case of DollarParantheses. It has been
        // easiest to simply keep them as different types.
        let (_, (begin_delim_str, delimiter_type, end_delim_char)) = preceded(space0, alt((
            map(tag("'"),  |begin_delim| ( begin_delim, Delimiter::SingleQuote(""),       '\'' )),
            map(tag("\""), |begin_delim| ( begin_delim, Delimiter::DoubleQuote(""),       '"'  )),
            //map(tag("("),  |begin_delim| ( begin_delim, Delimiter::Parentheses(""),       ')'  )),
            //map(tag("$("), |begin_delim| ( begin_delim, Delimiter::DollarParentheses(""), ')'  )),
        )))(input)?;

        // Here we prepare for a custom parser that will match only when it encounters a backslash
        // that is escaping our `end_delim`. This is primarily encountered in strings which are
        // escaping quotes, however it also comes up when processing macros
        let escaped_delim = &*format!("\\{}", end_delim_char);

        let (input, content) = delimited(
            tag(begin_delim_str),
            recognize(
                many0(
                    alt((
                        take_while1(|c| c != '\\' && c != end_delim_char),
                        tag(escaped_delim),
                        tag("\\"),
                    )),
                ),
            ),
            nom::character::complete::char(end_delim_char),
        )(input)?;

        let delimiter = match delimiter_type {
            Delimiter::SingleQuote(_)       => Delimiter::SingleQuote(content),
            Delimiter::DoubleQuote(_)       => Delimiter::DoubleQuote(content),
            //Delimiter::Parentheses(_)       => Delimiter::Parentheses(content),
            //Delimiter::DollarParentheses(_) => Delimiter::DollarParentheses(content),
        };

        Ok((input, Self{
            delimiter,
            processed: false,
        }))
    }
}

#[derive(Debug, PartialEq)]
pub struct Symbol<'a> {
    pub name: &'a str,
}

impl<'a> Symbol<'a> {
    pub fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (input, name) = is_a("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_")(input)?;
        Ok((input, Self{
            name,
        }))
    }
}

#[derive(Debug)]
pub struct Expression<'a> {
    pub val: &'a str,  // NOTE: transition hack before we switch to expr::Expr
}

impl<'a> Expression<'a> {
    pub fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (input, e) = recognize(parse_expr)(input)?;
        Ok((input, Self{
            val: e,
        }))
    }
}

#[derive(Debug)]
pub struct Condition<'a> {
    pub expression: Expression<'a>,
}

impl<'a> Condition<'a> {
    pub fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (input, c) = preceded(
            tuple((
                special_space,
                tag("if"),
                special_space,
            )),
            recognize(parse_expr)
        )(input)?;
        Ok((input, Self{
            expression: Expression{ val: c },
        }))
    }
}

#[derive(Debug)]
pub struct Prompt<'a> {
    pub text:       ConstantSymbol<'a>,
    pub condition:  Option<Condition<'a>>,
}

impl<'a> Prompt<'a> {
    pub fn parse(str_match: &str) -> impl Fn(&'a str) -> IResult<&'a str, Prompt<'a>> + '_ {
        move |input: &str| -> IResult<&str, Self> {
            let (input, (text, condition)) = preceded(
                tuple((
                    space0,
                    tag(str_match),
                    space0,
                )),
                tuple((
                    ConstantSymbol::parse,
                    opt(Condition::parse),
                )),
            )(input)?;

            Ok((input, Self {
                text,
                condition,
            }))
        }
    }
}

#[derive(Debug)]
pub struct Dependency<'a> {
    pub expression: Expression<'a>,
    pub condition:  Option<Condition<'a>>,
}

impl<'a> Dependency<'a> {
    pub fn parse(str_match: &str) -> impl Fn(&'a str) -> IResult<&'a str, Dependency<'a>> + '_ {
        move |input: &str| -> IResult<&str, Self> {
            let (input, (expression, condition)) = preceded(
                tuple((
                    space0,
                    tag(str_match),
                    space1,
                )),
                tuple((
                    Expression::parse,
                    opt(Condition::parse),
                )),
            )(input)?;

            Ok((input, Self {
                expression,
                condition,
            }))
        }
    }
}

#[derive(Debug)]
pub enum RangeType<'a> {
    Int(Int),
    Hex(Hex),
    Symbol(Symbol<'a>),
}

#[derive(Debug)]
pub struct Range<'a> {
    pub start: RangeType<'a>,
    pub end:   RangeType<'a>,
    pub condition: Option<Condition<'a>>,
}

impl<'a> Range<'a> {
    pub fn parse(str_match: &str) -> impl Fn(&'a str) -> IResult<&'a str, Range<'a>> + '_ {
        move |input: &str| -> IResult<&str, Self> {
            let (input, ((start, end), condition)) = preceded(
                tuple((
                    space0,
                    tag(str_match),
                    space1,
                )),
                tuple((
                    alt((
                        map(tuple((Hex::parse,    space1, Hex::parse)),    |(start, _, end)| (RangeType::Hex(start),    RangeType::Hex(end))    ),
                        map(tuple((Int::parse,    space1, Int::parse)),    |(start, _, end)| (RangeType::Int(start),    RangeType::Int(end))    ),
                        map(tuple((Symbol::parse, space1, Symbol::parse)), |(start, _, end)| (RangeType::Symbol(start), RangeType::Symbol(end)) ),
                    )),
                    opt(Condition::parse),
                )),
            )(input)?;
            Ok((input, Self {
                start,
                end,
                condition,
            }))
        }
    }
}

#[derive(Debug)]
pub struct Help<'a> {
    pub text: Vec<&'a str>,
}

impl<'a> Help<'a> {
    pub fn parse(str_match: &str) -> impl Fn(&'a str) -> IResult<&'a str, Help> + '_ {
        move |input: &str| -> IResult<&str, Self> {
            //let (input, _) = space0(input)?;
            //let (input, _) = tag("help")(input)?;
            //let (input, _) = many1(tuple((space0, line_ending)))(input)?;
            //let (_, raw_ws) = space1(input)?;
            //let ws = count_whitespace(raw_ws);
            //recognize(many1(alt((
            //    tag("\n"),
            //    take_while_help(ws),
            //))))(input)
            let (input, _) = tuple((
                    space0,
                    tag(str_match),
                    many1(tuple((
                        space0,
                        tag("\n"),
                    ))),
            ))(input)?;

            let (_, raw_whitespace) = space1(input)?;
            let initial_whitespace = count_whitespace(raw_whitespace);

            let (input, text) = many1(alt((
                    take_while_help(initial_whitespace),
                    tag("\n"),
            )))(input)?;

            Ok((input, Self {
                text,
            }))
        }
    }
}

pub fn count_whitespace(s: &str) -> usize {
    let mut count = 0;
    for c in s.chars() {
        if c == '\t' {
            let spaces = 8 - (count % 8);
            count += spaces;
        } else {
            count += 1;
        }
    }
    count
}

fn take_while_help(min_ws: usize) -> impl Fn(&str) -> IResult<&str, &str> {
    move |input: &str| -> IResult<&str, &str> {
        // First we need to record the amount of whitespace (tab==8, space=1) on the initial line
        let (_, raw_ws) = space1(input)?;
        let ws = count_whitespace(raw_ws);

        // Now we will take all characters from that line until and including the '\n' or EOF
        let (input, line) = preceded(
            space0,
            alt((
                take_until("\n"),          // Take until newline
                recognize(many1(anychar)), // There is no '\n', take any char until eof
            )),
        )(input)?;
        if ws < min_ws && !line.is_empty() { // if current line `ws` is less that `min_ws` the block has ended
            Err(nom::Err::Error(
                nom::error::Error{
                    input: input,
                    code: nom::error::ErrorKind::Tag,
                }
            ))
        } else {
            Ok((input, line))
        }
    }
}

pub fn take_line_ending(input: &str) -> IResult<&str, &str> {
    recognize(many1(tuple((space0, line_ending))))(input)
}

pub fn special_space(input: &str) -> IResult<&str, &str> {
    recognize(many0(alt((
        space1,
        tag("\\\n"),
    ))))(input)
}

pub fn take_comment(input: &str) -> IResult<&str, &str> {
    let (input, _) = space0(input)?;
    recognize(tuple((tag("#"), take_until("\n"))))(input)
}

pub fn take_continued_line(input: &str) -> IResult<&str, &str> {
    // This parser will take all bytes until it encounters a newline which is not escaped. or a
    // comment
    let (input, _) = space0(input)?;
    recognize(alt((
        map(tag("\n"), |_| ()), // Simplest case of the first char being a newline
        map(
            many_till(
                take(1usize),
                alt((
                    peek(tag("#")), // This is now a comment block and we can exit
                    recognize(tuple((
                        not(satisfy(|c| c == '\\')), // Make sure the next char isn't a \
                        take(1usize),                // Take whatever it was to move pos
                        peek(line_ending),           // Exit many_till if the next char is a newline
                    ))),
                )),
            ),
            |_| (),
        ),
    )))(input)
}

#[derive(Debug)]
pub struct Block<'a> {
    pub config:    KConfig<'a>,
    pub condition: Condition<'a>,
}

impl<'a> Block<'a> {
    pub fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (input, (condition, config)) = terminated(
            tuple((
                Condition::parse,
                KConfig::parse,
            )),
            tuple((
                space0,
                tag("endif"),
            )),
        )(input)?;
        Ok((input, Self {
            config,
            condition,
        }))
    }
}

#[derive(Debug, PartialEq)]
pub struct Int {
    pub val: i128,
}

impl Int {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        let (input, int) = recognize(tuple((
            opt(tag("-")),
            is_a("0123456789"),
        )))(input)?;

        match int.parse::<i128>() {
            Ok(val) => Ok((input, Self {
                val,
            })),
            Err(_)  => Err(nom::Err::Error(
                nom::error::Error::new(input, nom::error::ErrorKind::TooLarge)
            )),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Hex {
    pub val: u128,
}

impl Hex {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        let (input, hex) = preceded(tag("0x"), is_a("0123456789abcdefABCDEF"))(input)?;

        // Trim leading 0's as they will not affect our returned answer
        let hex = hex.trim_start_matches("0");

        // A length greater than 32 would overflow a u128
        if hex.len() > 32 {
            return Err(nom::Err::Error(
                nom::error::Error::new(input, nom::error::ErrorKind::TooLarge)
            ));
        }

        let val = hex.as_bytes()
            .iter()
            .rev()
            .enumerate()
            .map(|(idx, &v)| {
                ((v as char).to_digit(16).unwrap_or(0) as u128) << (idx * 4)
            })
            .sum();

        Ok((input, Self {
            val,
        }))
    }
}
