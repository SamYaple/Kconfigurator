use super::util::{
    take_comment,
    take_line_ending,
    parse_kstring,
    take_expr,
    take_cond,
    parse_option,
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{
        space0,
        space1,
    },
    combinator::{
        map,
        recognize,
    },
    multi::many0,
    sequence::{
        preceded,
        tuple,
    },
    IResult,
};

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
        let (input, c) = take_cond(input)?;
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
pub struct Depends<'a> {
    pub expression: Expression<'a>,
    pub condition:  Option<Condition<'a>>,
}

impl<'a> Depends<'a> {
    pub fn parse(input: &'a str) -> IResult<&'a str, Self> {
        parse_option("depends on")(input)
    }
}

impl std::fmt::Display for Depends<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expression)?;
        if let Some(condition) = &self.condition {
            write!(f, " {}", condition)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct KCommentBlock<'a> {
    pub prompt:  &'a str,
    pub depends: Option<Vec<Depends<'a>>>,
}

impl std::fmt::Display for KCommentBlock<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: make sure prompt gets wrapped in quotes without hardcode
        writeln!(f, "comment {}", self.prompt)?;
        if let Some(depends) = &self.depends {
            for dep in depends {
                writeln!(f, "\t{}", dep)?;
            }
        }
        Ok(())
    }
}

impl<'a> KCommentBlock<'a> {
    pub fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let mut depends = vec![];

        let (input, (prompt, _)) = preceded(
            tuple((
                space0,
                tag("comment"),
                space1,
            )),
            tuple((
                parse_kstring,
                many0(alt((
                    map(take_line_ending, |_| {}),
                    map(take_comment,     |_| {}),
                    map(Depends::parse,   |v| depends.push(v)),
                ))),
            )),
        )(input)?;
        Ok((input, Self {
                prompt,
                depends: if depends.is_empty() { None } else { Some(depends) },
        }))
    }
}
