use super::{
    KCommentBlock,
    KOption,
    util::{
        take_comment,
        take_continued_line,
        take_default,
        take_depends,
        take_help,
        take_optional,
        take_prompt,
        take_line_ending,
        take_type,
    },
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space0,
    combinator::map,
    multi::many1,
    sequence::{
        delimited,
        tuple,
    },
    IResult,
};

#[derive(Debug)]
pub struct KChoice<'a> {
    // option_type _is_ needed here :/
    pub prompt:      &'a str,
    pub options:     Vec<KOption<'a>>,
    pub optional:    bool,
    pub conditional: Option<&'a str>,
    pub defaults:    Option<Vec<(&'a str, Option<&'a str>)>>,
    pub depends:     Option<Vec<(&'a str, Option<&'a str>)>>,
    pub description: Option<&'a str>,
    pub help:        Option<&'a str>,
}

impl<'a> KChoice<'a> {
    pub fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let mut opt_prompt  = None;
        let mut description = None;
        let mut conditional = None;
        let mut help = None;
        let mut optional = false;
        let mut depends  = vec![];
        let mut defaults = vec![];
        let mut options  = vec![];

        let (input, _) = delimited(
            tuple((
                space0,
                tag("choice"),
                space0,
            )),
            tuple((
                take_continued_line,
                many1(alt((
                    map(take_line_ending,     |_| {}),
                    map(take_comment,         |_| {}),
                    map(take_depends,         |v| depends.push(v)),
                    map(take_default,         |v| defaults.push(v)),
                    map(take_optional,        |_| optional = true),
                    map(take_prompt,          |v| opt_prompt = Some(v)),
                    map(take_help,            |v| help = Some(v)),
                    map(KOption::parse,       |v| options.push(v)),
                    map(KCommentBlock::parse, |_| {}), // TODO: something useful with these?
                    map(take_type,            |(_opttype, desc, cond)| {
                        description = desc;
                        conditional = cond;
                    }),
                ))),
            )),
            tuple((
                space0,
                tag("endchoice"),
                space0,
            )),
        )(input)?;

        let prompt = match opt_prompt {
            Some(p) => p,
            None => {
                if description.is_none() {
                    eprintln!("EC_kchoice_no_prompt");
                }
                "PARSING SUCCESSFUL;MISSING PROMPT"
            }
        };
        Ok((input, Self{
                prompt,
                description,
                optional,
                conditional,
                defaults: if defaults.is_empty() { None } else { Some(defaults) },
                depends:  if depends.is_empty()  { None } else { Some(depends) },
                help,
                options,
        }))
    }
}
