mod expr;
mod util;

use util::{
    cleanup_raw_help,
    cleanup_raw_line,
    parse_kstring,
    push_optvec,
    take_block,
    take_comment,
    take_continued_line,
    take_def_bool,
    take_def_tristate,
    take_default,
    take_depends,
    take_help,
    take_imply,
    take_line_ending,
    take_mainmenu,
    take_name,
    take_optional,
    take_prompt,
    take_range,
    take_selects,
    take_source_kconfig,
    take_type,
    take_visible,
};

use nom::{
    branch::alt,
    bytes::complete::{
        tag,
    },
    character::complete::{
        line_ending,
        space0,
        space1,
    },
    combinator::{
        map,
    },
    multi::{
        many0,
        many1,
    },
    sequence::{
        delimited,
        preceded,
        tuple,
    },
    IResult,
};

#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub enum OptionType {
    Bool,
    Tristate,
    Str,
    #[default]
    Int,
    Hex,
}

impl std::fmt::Display for OptionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionType::Tristate      => write!(f, "\"tristate\""),
            OptionType::Bool          => write!(f, "\"bool\""),
            OptionType::Str           => write!(f, "\"str\""),
            OptionType::Int           => write!(f, "\"int\""),
            OptionType::Hex           => write!(f, "\"hex\""),
        }
    }
}

#[derive(Debug, Default)]
pub struct KConfig<'a> {
    mainmenu: Option<&'a str>,
    blocks:   Option<Vec<(&'a str, KConfig<'a>)>>,
    choices:  Option<Vec<KChoice<'a>>>,
    configs:  Option<Vec<&'a str>>,
    menus:    Option<Vec<KMenu<'a>>>,
    options:  Option<Vec<KOption<'a>>>,
}

impl<'a> KConfig<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let mut mainmenu = None;
        let mut blocks  = vec![];
        let mut choices = vec![];
        let mut configs = vec![];
        let mut menus   = vec![];
        let mut options = vec![];

        let (input, _) = many0(alt((
            map(take_line_ending,     |_| {}),
            map(take_comment,         |_| {}),
            map(take_block,           |v| blocks.push(v)),
            map(take_source_kconfig,  |v| configs.push(v)),
            map(take_mainmenu,        |v| mainmenu = Some(v)),
            map(KOption::parse,       |v| options.push(v)),
            map(KMenu::parse,         |v| menus.push(v)),
            map(KChoice::parse,       |v| choices.push(v)),
            map(KCommentBlock::parse, |_| {}), // TODO: something useful with these?
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
            for (_cond, block) in blocks {
                options.extend(block.collect_options());
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

#[derive(Debug, Default)]
pub struct KChoice<'a> {
    // option_type _is_ needed here :/
    prompt:      &'a str,
    options:     Vec<KOption<'a>>,
    optional:    bool,
    conditional: Option<&'a str>,
    defaults:    Option<Vec<(&'a str, Option<&'a str>)>>,
    depends:     Option<Vec<(&'a str, Option<&'a str>)>>,
    description: Option<&'a str>,
    help:        Option<&'a str>,
}

impl<'a> KChoice<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
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

#[derive(Debug, Default)]
pub struct KMenu<'a> {
    description: &'a str,
    blocks:  Option<Vec<(&'a str, KConfig<'a>)>>,
    choices: Option<Vec<KChoice<'a>>>,
    configs: Option<Vec<&'a str>>,
    depends: Option<Vec<(&'a str, Option<&'a str>)>>,
    menus:   Option<Vec<KMenu<'a>>>,
    options: Option<Vec<KOption<'a>>>,
    visible: Option<Vec<&'a str>>,
}

impl<'a> KMenu<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
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
                    map(take_comment,         |_| {}),
                    map(take_block,           |v| blocks.push(v)),
                    map(KChoice::parse,       |v| choices.push(v)),
                    map(KMenu::parse,         |v| menus.push(v)),
                    map(KOption::parse,       |v| options.push(v)),
                    map(take_visible,         |v| visible.push(v)),
                    map(take_depends,         |v| depends.push(v)),
                    map(take_source_kconfig,  |v| configs.push(v)),
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
            for (_name, block) in blocks {
                options.extend(block.collect_options());
            }
        }

        options
    }
}

#[derive(Debug, Default)]
pub struct KCommentBlock<'a> {
    description: &'a str,
    depends: Option<Vec<(&'a str, Option<&'a str>)>>,
}

impl<'a> KCommentBlock<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
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

#[derive(Debug, Default)]
pub struct KOption<'a> {
    pub name:         &'a str,         // This field must always exist
    pub option_type:  OptionType,      // This may be inferred from `def_bool` or `def_tristate`
    pub description:  Option<&'a str>, // this field comes from a prompt declared after the `type`
    pub prompt:       Option<&'a str>, // prompt exists as its own key
    pub conditional:  Option<&'a str>, // This conditional is from the end of description and prompt
    pub help:         Option<&'a str>, // Raw help text, with leading whitespace on each line
    pub depends:      Option<Vec<(&'a str, Option<&'a str>)>>, // These are strong dependencies
    pub selects:      Option<Vec<(&'a str, Option<&'a str>)>>, // These select options directly, avoiding the dependency graph
    pub implies:      Option<Vec<(&'a str, Option<&'a str>)>>, // This signifies a feature can provided to the implied option
    pub defaults:     Option<Vec<(&'a str, Option<&'a str>)>>, // This gives a list of defaults to use, with optional condition
    pub def_bool:     Option<Vec<(&'a str, Option<&'a str>)>>, // This is shorthand for `bool` type, then parses a `defaults`
    pub def_tristate: Option<Vec<(&'a str, Option<&'a str>)>>, // This is shorthand for `tristate` type, then parses a `defaults`
    pub range:        Option<Vec<((&'a str, &'a str), Option<&'a str>)>>, // Only valid for `hex` and `int` types
}

impl<'a> KOption<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let mut opt_option_type = None;

        let mut description = None;
        let mut prompt      = None;
        let mut conditional = None;
        let mut help        = None;

        let mut range        = vec![];
        let mut depends      = vec![];
        let mut selects      = vec![];
        let mut implies      = vec![];
        let mut defaults     = vec![];
        let mut def_bool     = vec![];
        let mut def_tristate = vec![];

        let (input, (name, _)) = preceded(
            tuple((
                space0,
                alt((tag("config"), tag("menuconfig"))),
                space1,
            )),
            tuple((
                take_name,
                many1(alt((
                    map(take_comment,      |_| {}),
                    map(take_line_ending,  |_| {}),
                    map(take_default,      |v| defaults.push(v)),
                    map(take_depends,      |v| depends.push(v)),
                    map(take_selects,      |v| selects.push(v)),
                    map(take_imply,        |v| implies.push(v)),
                    map(take_def_bool,     |v| def_bool.push(v)),
                    map(take_def_tristate, |v| def_tristate.push(v)),
                    map(take_range,        |v| range.push(v)),
                    map(take_help,         |v| {
                        if let Some(_) = help {
                            eprintln!("EC_help_overridden");
                        }
                        help = Some(v);
                    }),
                    map(take_prompt,       |v| {
                        if let Some(_) = prompt {
                            eprintln!("EC_prompt_overridden");
                        }
                        prompt = Some(v);
                    }),
                    map(take_type,         |(opttype, desc, cond)| {
                        if let Some(option_type) = opt_option_type {
                            if option_type == opttype {
                                // This branch indicates the option has the same type declared twice
                                eprintln!("EC_type_duplicate");
                            } else {
                                // This means the type of this option CHANGED, not good at all.
                                eprintln!("EC_type_overridden");
                            }
                        }
                        opt_option_type = Some(opttype);

                        if desc.is_some() {
                            if description.is_some() {
                                eprintln!("EC_description_overridden");
                            }
                            description = desc;
                        }

                        if cond.is_some() {
                            if conditional.is_some() {
                                eprintln!("EC_conditional_overridden");
                            }
                            conditional = cond;
                        }
                    }),
                    map(tuple((space1, tag("modules"))), |_| {}), // NOTE: only shows up once in MODULES option
                ))),
            )),
        )(input)?;

        let option_type = match opt_option_type {
            Some(option_type) => option_type,
            None => {
                if def_bool.len() > 0 {
                    OptionType::Bool
                } else if def_tristate.len() > 0 {
                    OptionType::Tristate
                } else {
                    // Currently there are ~3 dozen options that do not have a type definition
                    // They are all `int` types. We can detect and warn here without breaking
                    eprintln!("EC_missing_type");
                    OptionType::Int
                }
            }
        };

        if def_bool.len() > 0 && option_type != OptionType::Bool {
            eprintln!("EC_type_mismatch");
        }
        if def_tristate.len() > 0 && option_type != OptionType::Tristate {
            eprintln!("EC_type_mismatch");
        }
        if range.len() > 0 && (option_type != OptionType::Int && option_type != OptionType::Hex){
            eprintln!("EC_range_in_wrong_type");
        }
        //println!("SAMMAS {}", name);
        Ok((input, Self{
                name,
                option_type,
                description,
                prompt,
                conditional,
                help,
                range:    if range.is_empty()    { None } else { Some(range)   },
                depends:  if depends.is_empty()  { None } else { Some(depends) },
                selects:  if selects.is_empty()  { None } else { Some(selects) },
                implies:  if implies.is_empty()  { None } else { Some(implies) },
                defaults: if defaults.is_empty() { None } else { Some(defaults)},
                def_bool: if def_bool.is_empty() { None } else { Some(def_bool)},
                def_tristate: if def_tristate.is_empty() { None } else { Some(def_tristate) },
        }))
    }
}

fn escape_quoted(input: &str) -> String {
    let mut result = String::new();
    result.push('"');

    for c in input.chars() {
        match c {
            '"' | '\\' => result.push('\\'),
            _ => {}
        }
        result.push(c);
    }

    result.push('"');
    result
}

impl std::fmt::Display for KOption<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // playing with macros
        macro_rules! print_if_some {
            ($field:ident) => {
                if let Some(value) = &self.$field {
                    let quoted = escape_quoted(&cleanup_raw_line(value));
                    writeln!(f, "      {}: {}", stringify!($field), quoted)?;
                }
            };
        }
        macro_rules! print_if_some_list_cond {
            ($field:ident) => {
                if let Some(values) = &self.$field {
                    writeln!(f, "      {}:", stringify!($field))?;
                    for (expr, cond) in values {
                        let esc_expr = escape_quoted(&cleanup_raw_line(expr));
                        writeln!(f, "        - expression: {}", esc_expr)?;
                        if let Some(c) = cond {
                            let esc_c = escape_quoted(&cleanup_raw_line(c));
                            writeln!(f, "          condition:  {}", esc_c)?;
                        }
                    }
                }
            };
        }

        writeln!(f, "    - name: {}", escape_quoted(self.name))?;
        writeln!(f, "      type: {}", self.option_type)?;

        print_if_some!(description);
        print_if_some!(prompt);
        print_if_some!(conditional);

        print_if_some_list_cond!(depends);
        print_if_some_list_cond!(defaults);
        print_if_some_list_cond!(selects);
        print_if_some_list_cond!(implies);
        print_if_some_list_cond!(def_bool);
        print_if_some_list_cond!(def_tristate);

        if let Some(values) = &self.range {
            for ((begin, end), cond) in values {
                writeln!(f, "      range:")?;
                writeln!(f, "        - begin: {begin}")?;
                writeln!(f, "          end:   {end}")?;
                if let Some(c) = cond {
                    let esc_c = escape_quoted(&cleanup_raw_line(c));
                    writeln!(f, "          condition: {}", esc_c)?;
                }
            }
        }

        if let Some(text) = &self.help {
            writeln!(f, "      help: |")?;
            for l in cleanup_raw_help(text).split('\n') {
                writeln!(f, "        {}", l)?;
            }
        }
        Ok(())
    }
}

pub fn load_from_file(path_string: String) -> String {
    let pathed = std::path::Path::new(&path_string).to_path_buf();
    match std::fs::read_to_string(pathed) {
        Ok(content) => return content,
        Err(e) => {
            panic!("Failed to open '{}' with error '{}'", path_string, e);
        }
    }
}


// TODO convert this to take complete
pub fn take_kconfig(input: &str) -> KConfig {
    //let (remaining, config) = KConfig::parse(input).unwrap();
    match KConfig::parse(input) {
        Ok((remaining, config)) => {
            if remaining != "" {
                panic!("SAMMAS ERROR Unprocessed input:```\n{}'\n```", remaining);
            }
            return config;
        }
        Err(error) => {
            panic!("SAMMAS ERROR Proper error:\n{:?}\n\n", error);
        }
    }
}
