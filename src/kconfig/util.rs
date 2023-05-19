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

use super::{
    OptionType,
    KConfig,
    expr::{
        Expr,
        expr,
    }
};

pub fn push_optvec<T>(opt_vec: &mut Option<Vec<T>>, val: T) {
    // this pattern seems wrong to break into a function... maybe its fine
    if let Some(ref mut vec) = opt_vec {
        vec.push(val);
    } else {
        *opt_vec = Some(vec![val]);
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

pub fn prefix_spaces(n: usize) -> String {
    let mut result = String::with_capacity(n);
    for _ in 0..n {
        result.push(' ');
    }
    result
}

pub fn cleanup_raw_help(text: &str) -> String {
    // Preserve the whitespace structure while trimming the text in the help functions
    let init_ws = count_whitespace(text);
    let mut help = String::new();
    for l in text.split('\n') {
        let line_ws = count_whitespace(l);
        if line_ws < init_ws {
            help += l.trim_start();
            help += "\n";
        } else {
            let padding = line_ws - init_ws;
            help += &prefix_spaces(padding);
            help += l.trim_start();
            help += "\n";
        }
    }
    help.trim_end().to_string()
}

pub fn take_help(input: &str) -> IResult<&str, &str> {
    let (input, _) = space0(input)?;
    let (input, _) = tag("help")(input)?;
    let (input, _) = many1(tuple((space0, line_ending)))(input)?;
    let (_, raw_ws) = space1(input)?;
    let ws = count_whitespace(raw_ws);
    recognize(many1(alt((
        tag("\n"),
        take_while_help(ws),
    ))))(input)
}

pub fn take_while_help(min_ws: usize) -> impl Fn(&str) -> IResult<&str, &str> {
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

pub fn cleanup_raw_line(text: &str) -> String {
    let mut result = String::new();
    for l in text.split('\n') {
        let mut cleaned_line = l.trim_start().to_string();
        if cleaned_line.chars().last() == Some('\\') {
            cleaned_line.pop();
        }
        if !result.is_empty() {
            result.push_str(&" ");
        }
        result.push_str(&cleaned_line.trim_end());
    }
    result
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

pub fn parse_opttype(input: &str) -> IResult<&str, OptionType> {
    alt((
        map(tag("bool"),     |_| OptionType::Bool),
        map(tag("hex"),      |_| OptionType::Hex),
        map(tag("int"),      |_| OptionType::Int),
        map(tag("string"),   |_| OptionType::Str),
        map(tag("tristate"), |_| OptionType::Tristate),
    ))(input)
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

pub fn take_type(input: &str) -> IResult<&str, (OptionType, Option<&str>, Option<&str>)> {
    let (input, _) = space0(input)?;
    let (input, opttype) = parse_opttype(input)?;
    let (input, description) = opt(parse_kstring)(input)?;
    let (input, conditional) = opt(take_cond)(input)?;
    Ok((input, (opttype, description, conditional)))
}

pub fn take_line_ending(input: &str) -> IResult<&str, &str> {
    recognize(many1(tuple((space0, line_ending))))(input)
}

pub fn take_tagged_line<'a>(input: &'a str, str_match: &str) -> IResult<&'a str, &'a str> {
    let (input, _) = tuple((space0, tag(str_match), space1))(input)?;
    take_continued_line(input)
}

pub fn take_named_line<'a>(input: &'a str, str_match: &str) -> IResult<&'a str, (&'a str, Option<&'a str>)> {
    let (input, expr) = preceded(
        tuple((
            space0,
            tag(str_match),
            space1,
        )),
        recognize(take_expr),
    )(input)?;
    let (input, cond) = opt(take_cond)(input)?;
    Ok((input, (expr, cond)))
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

pub fn take_imply(input: &str) -> IResult<&str, (&str, Option<&str>)> {
    take_named_line(input, "imply")
}

pub fn take_default(input: &str) -> IResult<&str, (&str, Option<&str>)> {
    take_named_line(input, "default")
}

pub fn take_def_tristate(input: &str) -> IResult<&str, (&str, Option<&str>)> {
    take_named_line(input, "def_tristate")
}

pub fn take_def_bool(input: &str) -> IResult<&str, (&str, Option<&str>)> {
    take_named_line(input, "def_bool")
}

pub fn take_depends(input: &str) -> IResult<&str, (&str, Option<&str>)> {
    take_named_line(input, "depends on")
}

pub fn take_range(input: &str) -> IResult<&str, ((&str, &str), Option<&str>)> {
    let (input, _) = tuple((
        space0,
        tag("range"),
        space1,
    ))(input)?;
    let (input, (start, _, end)) = alt((
        tuple((take_signed_int, space1, take_signed_int)),
        tuple((take_hex,        space1, take_hex)),
        tuple((take_name,       space1, take_name)),
    ))(input)?;
    let (input, cond) = opt(take_cond)(input)?;
    Ok((input, ((start, end), cond)))
}

pub fn take_prompt(input: &str) -> IResult<&str, &str> {
    take_tagged_line(input, "prompt")
}

pub fn take_expr(input: &str) -> IResult<&str, Expr> {
    expr(input)
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

pub fn take_selects(input: &str) -> IResult<&str, (&str, Option<&str>)> {
    take_named_line(input, "select")
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