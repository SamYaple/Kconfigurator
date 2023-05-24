use super::{
    KConfig,
    expr::{
        Expr,
        parse_expr,
    }
};

use nom::{
    branch::alt,
    bytes::complete::{
        tag,
        take,
        take_until,
        take_while1,
    },
    character::complete::{
        anychar,
        line_ending,
        multispace1,
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
            map(tag("bool"),     |_| OptionType::Bool),
            map(tag("hex"),      |_| OptionType::Hex),
            map(tag("int"),      |_| OptionType::Int),
            map(tag("string"),   |_| OptionType::Str),
            map(tag("tristate"), |_| OptionType::Tristate),
        ))(input)
    }
}


impl std::fmt::Display for OptionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionType::Tristate => write!(f, "tristate"),
            OptionType::Bool     => write!(f, "bool"),
            OptionType::Hex      => write!(f, "hex"),
            OptionType::Int      => write!(f, "int"),
            OptionType::Str      => write!(f, "str"),
        }
    }
}

#[derive(Debug)]
pub struct Expression<'a> {
    pub val: &'a str,  // NOTE: transition hack before we switch to expr::Expr
}

impl<'a> Expression<'a> {
    pub fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (input, e) = recognize(take_expr)(input)?;
        Ok((input, Self{
            val: e,
        }))
    }
}

impl std::fmt::Display for Expression<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.val)?;
        Ok(())
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
            recognize(take_expr)
        )(input)?;
        Ok((input, Self{
            expression: Expression{ val: c },
        }))
    }
}

impl std::fmt::Display for Condition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "if {}", self.expression)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Prompt<'a> {
    pub text:       &'a str,
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
                    parse_kstring,
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

impl std::fmt::Display for Prompt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)?;
        if let Some(condition) = &self.condition {
            write!(f, " {}", condition)?;
        }
        Ok(())
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

impl std::fmt::Display for Dependency<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expression)?;
        if let Some(condition) = &self.condition {
            write!(f, " {}", condition)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ReverseDependency<'a> {
    pub symbol:    &'a str,
    pub condition: Option<Condition<'a>>,
}

impl<'a> ReverseDependency<'a> {
    pub fn parse(str_match: &str) -> impl Fn(&'a str) -> IResult<&'a str, ReverseDependency> + '_ {
        move |input: &str| -> IResult<&str, Self> {
            let (input, (symbol, condition)) = preceded(
                tuple((
                    space0,
                    tag(str_match),
                    space1,
                )),
                tuple((
                    take_name,
                    opt(Condition::parse),
                )),
            )(input)?;

            Ok((input, Self {
                symbol,
                condition,
            }))
        }
    }
}

impl std::fmt::Display for ReverseDependency<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol)?;
        if let Some(condition) = &self.condition {
            write!(f, " {}", condition)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Range<'a> {
    pub start:     &'a str,
    pub end:       &'a str,
    pub condition: Option<Condition<'a>>,
}

impl<'a> Range<'a> {
    pub fn parse(str_match: &str) -> impl Fn(&'a str) -> IResult<&'a str, Range<'a>> + '_ {
        move |input: &str| -> IResult<&str, Self> {
            let (input, ((start, _, end), condition)) = preceded(
                tuple((
                    space0,
                    tag(str_match),
                    space1,
                )),
                tuple((
                    alt((
                        tuple((take_signed_int, space1, take_signed_int)),
                        tuple((take_hex,        space1, take_hex)),
                        tuple((take_name,       space1, take_name)),
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

impl std::fmt::Display for Range<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.start, self.end)?;
        if let Some(condition) = &self.condition {
            write!(f, " {}", condition)?;
        }
        Ok(())
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

impl std::fmt::Display for Help<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in &self.text {
            write!(f, "  {}", line)?;
        }
        Ok(())
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

pub fn is_hex(chr: u8) -> bool {
    // matches ASCII digits A-Fa-f0-9
    (chr >= 0x41 && chr <= 0x46) || (chr >= 0x61 && chr <= 0x66) || (chr >= 0x30 && chr <= 0x39)
}

pub fn is_digit(chr: u8) -> bool {
    // matches ASCII digits 0-9
    chr >= 0x30 && chr <= 0x39
}

pub fn is_uppercase(chr: u8) -> bool {
    // matches ASCII uppercase letters A-Z
    chr >= 0x41 && chr <= 0x5A
}

pub fn is_lowercase(chr: u8) -> bool {
    // matches ASCII lowercase letters a-z
    chr >= 0x61 && chr <= 0x7A
}

// TODO: Fixup this function to match only uppercase followed by all of these matches
pub fn is_config_name(c: char) -> bool {
    is_uppercase(c as u8) || is_digit(c as u8) || c == '_' || is_lowercase(c as u8)
}

pub fn take_name(input: &str) -> IResult<&str, &str> {
    take_while1(is_config_name)(input)
}

pub fn parse_kstring(input: &str) -> IResult<&str, &str> {
    // NOTE: this will take newlines and other chars which are not valid
    preceded(
        space0,
        alt((
            // we check for double qoute strings, careful to preserve escaped \"
            delimited(
                tag("\""),
                recognize(many0(alt((
                    take_while1(|c| c != '\\' && c != '"'),
                    tag("\\\""), // We have encoutered an escaped quote -- \"
                    tag("\\"),   // We didn't find an end to our string, take -- \\
                )))),
                tag("\""),
            ),
            // we check for single qoute strings, careful to preserve escaped \'
            delimited(
                tag("'"),
                recognize(many0(alt((
                    take_while1(|c| c != '\\' && c != '\''),
                    tag("\\'"), // We have encoutered an escaped quote -- \'
                    tag("\\"),  // We didn't find an end to our string, take -- \\
                )))),
                tag("'"),
            ),
        ))
    )(input)
}

pub fn take_line_ending(input: &str) -> IResult<&str, &str> {
    recognize(many1(tuple((space0, line_ending))))(input)
}

pub fn take_tagged_line<'a>(input: &'a str, str_match: &str) -> IResult<&'a str, &'a str> {
    let (input, _) = tuple((space0, tag(str_match), space1))(input)?;
    take_continued_line(input)
}

pub fn take_mainmenu(input: &str) -> IResult<&str, &str> {
    take_tagged_line(input, "mainmenu")
}

pub fn take_source_kconfig(input: &str) -> IResult<&str, &str> {
    take_tagged_line(input, "source")
}

pub fn take_visible(input: &str) -> IResult<&str, &str> {
    let (input, _) = tuple((
        space0,
        tag("visible if"),
        space1,
    ))(input)?;
    let (input, cond) = recognize(take_expr)(input)?;
    Ok((input, cond))
}

pub fn take_expr(input: &str) -> IResult<&str, Expr> {
    parse_expr(input)
}

pub fn take_cond(input: &str) -> IResult<&str, &str> {
    preceded(
        tuple((
            special_space,
            tag("if"),
            special_space,
        )),
        recognize(take_expr)
    )(input)
}

pub fn special_space(input: &str) -> IResult<&str, &str> {
    recognize(many0(alt((
        space1,
        tag("\\\n"),
    ))))(input)
}

pub fn take_optional(input: &str) -> IResult<&str, bool> {
    map(tuple((space0, tag("optional"))), |_| true)(input)
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

pub fn take_block(input: &str) -> IResult<&str, (&str, KConfig)> {
    let (input, condition) = take_cond(input)?;
    let (input, _) = multispace1(input)?;
    let (input, config) = KConfig::parse(input)?;
    let (input, _) = tag("endif")(input)?;
    Ok((input, (condition, config)))
}

pub fn take_signed_int(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        opt(tag("-")),
        take_while1(|c| is_digit(c as u8)),
    )))(input)
}

pub fn take_hex(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        tag("0x"),
        take_while1(|c| is_hex(c as u8)),
    )))(input)
}
