use super::{
    KCommentBlock,
    KOption,
    OptionType,
    Dependency,
    Help,
    Prompt,
    util::{
        take_comment,
        take_line_ending,
    },
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space0,
    combinator::{
        map,
        opt,
    },
    multi::many1,
    sequence::{
        delimited,
        tuple,
    },
    IResult,
};

#[derive(Debug)]
pub struct KChoice<'a> {
    pub option_type: OptionType,
    pub prompts:     Vec<Prompt<'a>>,
    pub options:     Vec<KOption<'a>>,
    pub optional:    bool,
    pub defaults:    Option<Vec<Dependency<'a>>>,
    pub depends:     Option<Vec<Dependency<'a>>>,
    pub help:        Option<Help<'a>>,
}

impl<'a> KChoice<'a> {
    pub fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let mut opt_option_type = None;
        let mut opt_prompt_from_type = None;
        let mut help = None;
        let mut optional = false;
        let mut depends  = vec![];
        let mut defaults = vec![];
        let mut options  = vec![];
        let mut prompts  = vec![];

        let type_line_parser = tuple((
            OptionType::parse,
            opt(Prompt::parse("")),
        ));

        let (input, _) = delimited(
            tuple((
                space0,
                tag("choice"),
                space0,
            )),
            many1(alt((
                map(take_line_ending,                |_| {}),
                map(take_comment,                    |_| {}),
                map(tuple((space0, tag("optional"))), |_| optional = true),
                map(Help::parse("help"),             |v| help = Some(v)),
                map(KOption::parse,                  |v| options.push(v)),
                map(KCommentBlock::parse,            |_| {}), // TODO: something useful with these?
                map(Prompt::parse("prompt"),         |v| prompts.push(v)),
                map(Dependency::parse("default"),    |v| defaults.push(v)),
                map(Dependency::parse("depends on"), |v| depends.push(v)),
                map(type_line_parser,  |(opttype, opt_prompt)| {
                    opt_option_type = Some(opttype);
                    opt_prompt_from_type = opt_prompt;
                }),
            ))),
            tuple((
                space0,
                tag("endchoice"),
                space0,
            )),
        )(input)?;

        // CODE VOMMIT DO NOT COMMIT!!
        // it do work tho
        let mut opt_types = vec![];
        let mut tmptype = OptionType::Int; 
        for opt in &options {
            opt_types.push(opt.option_type);
            tmptype = opt.option_type;
        }
        for optt in opt_types {
            if optt != tmptype {
                eprintln!("EC_kchoice_options_differ_in_type");
            }
        }
        let option_type = match opt_option_type {
            Some(option_type) => {
                if option_type != tmptype {
                    eprintln!("EC_kchoice_options_differ_in_type");
                }
                option_type
            },
            None => tmptype,
        };

        if let Some(prompt) = opt_prompt_from_type {
            prompts.push(prompt);
        }

        Ok((input, Self{
                option_type,
                optional,
                prompts,
                defaults: if defaults.is_empty() { None } else { Some(defaults) },
                depends:  if depends.is_empty()  { None } else { Some(depends)  },
                help,
                options,
        }))
    }
}
