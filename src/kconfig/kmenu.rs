use super::{
    KChoice,
    KCommentBlock,
    KOption,
    Dependency,
    Prompt,
    Block,
    Annotation,
    util::{
        take_continued_line,
        take_line_ending,
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
pub struct KMenu<'a> {
    pub description: &'a str,
    pub blocks:  Option<Vec<Block<'a>>>,
    pub choices: Option<Vec<KChoice<'a>>>,
    pub configs: Option<Vec<Prompt<'a>>>,
    pub depends: Option<Vec<Dependency<'a>>>,
    pub menus:   Option<Vec<KMenu<'a>>>,
    pub options: Option<Vec<KOption<'a>>>,
    pub visible: Option<Vec<Dependency<'a>>>,
}

impl<'a> KMenu<'a> {
    pub fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let mut blocks  = vec![];
        let mut choices = vec![];
        let mut configs = vec![];
        let mut depends = vec![];
        let mut menus   = vec![];
        let mut options = vec![];
        let mut visible = vec![];

        let (input, (description, _)) = delimited(
            tuple((
                space0,
                tag("menu"),
                space0,
            )),
            tuple((
                take_continued_line,
                many1(alt((
                    map(take_line_ending,     |_| {}),
                    map(Annotation::parse,    |_| {}),
                    map(Block::parse,         |v| blocks.push(v)),
                    map(KChoice::parse,       |v| choices.push(v)),
                    map(KMenu::parse,         |v| menus.push(v)),
                    map(KOption::parse,       |v| options.push(v)),
                    map(Dependency::parse("visible if"), |v| visible.push(v)),
                    map(Dependency::parse("depends on"), |v| depends.push(v)),
                    map(Prompt::parse("source"),         |v| configs.push(v)),
                    map(KCommentBlock::parse, |_| {}), // TODO: something useful with these?
                ))),
            )),
            tuple((
                space0,
                tag("endmenu"),
                space0,
            )),
        )(input)?;

        Ok((input, Self{
                description,
                blocks:  if blocks.is_empty()   { None } else { Some(blocks) },
                choices: if choices.is_empty()  { None } else { Some(choices) },
                configs: if configs.is_empty()  { None } else { Some(configs) },
                depends: if depends.is_empty()  { None } else { Some(depends) },
                menus:   if menus.is_empty()    { None } else { Some(menus) },
                options: if options.is_empty()  { None } else { Some(options) },
                visible: if visible.is_empty()  { None } else { Some(visible) },
        }))
    }

    pub fn collect_options(&self) -> Vec<&KOption<'a>> {
        let mut options: Vec<&KOption<'a>> = Vec::new();

        if let Some(opts) = &self.options {
            options.extend(opts.iter());
        }

        if let Some(choices) = &self.choices {
            for choice in choices {
                options.extend(choice.options.iter());
            }
        }

        if let Some(menus) = &self.menus {
            for menu in menus {
                options.extend(menu.collect_options());
            }
        }

        if let Some(blocks) = &self.blocks {
            for block in blocks {
                options.extend(block.config.collect_options());
            }
        }

        options
    }
}
