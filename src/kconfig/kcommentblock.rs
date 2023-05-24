use super::util::{
    take_comment,
    take_line_ending,
    parse_kstring,
    Dependency,
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{
        space0,
        space1,
    },
    combinator::map,
    multi::many0,
    sequence::{
        preceded,
        tuple,
    },
    IResult,
};

#[derive(Debug)]
pub struct KCommentBlock<'a> {
    pub prompt:  &'a str,
    pub depends: Option<Vec<Dependency<'a>>>,
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
                    map(Dependency::parse("depends on"), |v| depends.push(v)),
                ))),
            )),
        )(input)?;
        Ok((input, Self {
                prompt,
                depends: if depends.is_empty() { None } else { Some(depends) },
        }))
    }
}
