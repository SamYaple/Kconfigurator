use super::{
    KChoice,
    KCommentBlock,
    KMenu,
    KOption,
    Prompt,
    Block,
    Annotation,
    util::{
        take_line_ending,
    },
};

use nom::{
    branch::alt,
    combinator::map,
    multi::many0,
    IResult,
};

#[derive(Debug)]
pub struct KConfig<'a> {
    pub mainmenu: Option<Prompt<'a>>,
    pub blocks:   Option<Vec<Block<'a>>>,
    pub choices:  Option<Vec<KChoice<'a>>>,
    pub configs:  Option<Vec<Prompt<'a>>>,
    pub menus:    Option<Vec<KMenu<'a>>>,
    pub options:  Option<Vec<KOption<'a>>>,
}

impl<'a> KConfig<'a> {
    pub fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let mut mainmenu = None;
        let mut blocks  = vec![];
        let mut choices = vec![];
        let mut configs = vec![];
        let mut menus   = vec![];
        let mut options = vec![];

        let (input, _) = many0(alt((
            map(take_line_ending,     |_| {}),
            map(Annotation::parse,    |_| {}),
            map(Block::parse,         |v| blocks.push(v)),
            map(Prompt::parse("source"),   |v| configs.push(v)),
            map(Prompt::parse("mainmenu"), |v| mainmenu = Some(v)),
            map(KOption::parse,       |v| options.push(v)),
            map(KMenu::parse,         |v| menus.push(v)),
            map(KChoice::parse,       |v| choices.push(v)),
            map(KCommentBlock::parse, |_c| {}), //eprintln!("\n```\n{}```\n", _c)),
        )))(input)?;
        Ok((input, Self{
                mainmenu,
                blocks:  if blocks.is_empty()   { None } else { Some(blocks) },
                choices: if choices.is_empty()  { None } else { Some(choices) },
                configs: if configs.is_empty()  { None } else { Some(configs) },
                menus:   if menus.is_empty()    { None } else { Some(menus) },
                options: if options.is_empty()  { None } else { Some(options) },
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

        if let Some(blocks) = &self.blocks {
            for block in blocks {
                options.extend(block.config.collect_options());
            }
        }

        if let Some(menus) = &self.menus {
            for menu in menus {
                options.extend(menu.collect_options());
            }
        }

        options
    }
}
