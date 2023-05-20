use super::util::{
    take_comment,
    take_depends,
    take_line_ending,
    parse_kstring,
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
    pub description: &'a str,
    pub depends: Option<Vec<(&'a str, Option<&'a str>)>>,
}

impl<'a> KCommentBlock<'a> {
    pub fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let mut depends = vec![];

        let (input, (description, _)) = preceded(
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
                    map(take_depends,     |v| depends.push(v)),
                ))),
            )),
        )(input)?;
        Ok((input, Self {
                description,
                depends: if depends.is_empty() { None } else { Some(depends) },
        }))
    }
}
