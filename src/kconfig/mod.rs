use nom::branch::alt;
use nom::bytes::complete::{tag, take, take_until, take_while1};
use nom::character::complete::{line_ending, multispace0, multispace1, satisfy, space0, space1};
use nom::combinator::{eof, map, not, peek, recognize};
use nom::multi::{many0, many1, many_till};
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug, PartialEq, Default)]
enum OptionType {
    #[default]
    Uninitialized, // TODO: This exists to be a default. Re-evaluate that reasoning.
    Bool,
    Tristate,
    Str,
    Int,
    Hex,
}

impl std::fmt::Display for OptionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionType::Uninitialized => write!(f, "Uninitialized"),
            OptionType::Tristate      => write!(f, "tristate"),
            OptionType::Bool          => write!(f, "bool"),
            OptionType::Str           => write!(f, "str"),
            OptionType::Int           => write!(f, "int"),
            OptionType::Hex           => write!(f, "hex"),
        }
    }
}

#[derive(Debug, Default)]
pub struct KConfig<'a> {
    options:  Option<Vec<KOption<'a>>>,
    choices:  Option<Vec<KChoice<'a>>>,
    configs:  Option<Vec<&'a str>>,
    blocks:   Option<Vec<(&'a str, KConfig<'a>)>>,
    menus:    Option<Vec<KMenu<'a>>>,
    mainmenu: Option<&'a str>,
}

impl<'a> KConfig<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let mut k = Self::default();
        let (input, _) = many0(alt((
            map(take_block,           |val| push_optvec(&mut k.blocks,  val)),
            map(KChoice::parse,       |val| push_optvec(&mut k.choices, val)),
            map(KMenu::parse,         |val| push_optvec(&mut k.menus,   val)),
            map(KOption::parse,       |val| push_optvec(&mut k.options, val)),
            map(take_source_kconfig,  |val| push_optvec(&mut k.configs, val)),
            map(take_mainmenu,        |val| k.mainmenu = Some(val)),
            map(KCommentBlock::parse, |_|   {}), // TODO: something useful with these?
            map(take_comment,         |_|   {}),
            map(take_line_ending,     |_|   {}),
        )))(input)?;
        Ok((input, k))
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
            for (_name, block) in blocks {
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
    options:     Vec<KOption<'a>>,
    prompt:      &'a str,
    depends:     Option<Vec<&'a str>>,
    defaults:    Option<Vec<&'a str>>,
    help:        Option<&'a str>,
    optional:    bool,
    option_type: OptionType,
    description: Option<&'a str>,
}

impl<'a> KChoice<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let mut k = Self::default();

        let (input, _) = space0(input)?;
        let (input, _) = tag("choice")(input)?;
        let (input, _) = space0(input)?;
        let (input, _) = many1(line_ending)(input)?;
        let (input, _) = many1(alt((
            map(KOption::parse,       |val| k.options.push(val)),
            map(take_depends,         |val| push_optvec(&mut k.depends,  val)),
            map(take_default,         |val| push_optvec(&mut k.defaults, val)),
            map(take_optional,        |_|   k.optional = false),
            map(take_prompt,          |val| k.prompt = val),
            map(take_help,            |val| k.help = Some(val)),
            map(KCommentBlock::parse, |_|   {}), // TODO: something useful with these?
            map(take_comment,         |_|   {}),
            map(take_line_ending,     |_|   {}),
            map(take_type,            |(opttype, desc)| {
                k.option_type = opttype;
                let desc = desc.trim_end(); // NOTE: This feels suspect here... this whole block does
                if !desc.is_empty() {
                    k.description = Some(desc);
                }
            }),
        )))(input)?;
        let (input, _) = space0(input)?;
        let (input, _) = tag("endchoice")(input)?;
        Ok((input, k))
    }
}

#[derive(Debug, Default)]
pub struct KMenu<'a> {
    options:     Option<Vec<KOption<'a>>>,
    menus:       Option<Vec<KMenu<'a>>>,
    choices:     Option<Vec<KChoice<'a>>>,
    configs:     Option<Vec<&'a str>>,
    blocks:      Option<Vec<(&'a str, KConfig<'a>)>>,
    description: &'a str,
    depends:     Option<Vec<&'a str>>,
    visible:     Option<Vec<&'a str>>,
}

impl<'a> KMenu<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let mut k = Self::default();

        let (input, _) = tag("menu")(input)?;
        let (input, description) = take_continued_line(input)?;
        k.description = description;
        let (input, _) = many1(alt((
            map(take_block,           |val| push_optvec(&mut k.blocks,  val)),
            map(KChoice::parse,       |val| push_optvec(&mut k.choices, val)),
            map(KMenu::parse,         |val| push_optvec(&mut k.menus,   val)),
            map(KOption::parse,       |val| push_optvec(&mut k.options, val)),
            map(take_visible,         |val| push_optvec(&mut k.visible, val)),
            map(take_depends,         |val| push_optvec(&mut k.depends, val)),
            map(take_source_kconfig,  |val| push_optvec(&mut k.configs, val)),
            map(KCommentBlock::parse, |_|   {}), // TODO: something useful with these?
            map(take_comment,         |_|   {}),
            map(take_line_ending,     |_|   {}),
        )))(input)?;
        let (input, _) = tag("endmenu")(input)?;
        Ok((input, k))
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
    depends:     Option<Vec<&'a str>>,
}

impl<'a> KCommentBlock<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let mut k = Self::default();

        let (input, _) = space0(input)?;
        let (input, _) = tag("comment")(input)?;
        let (input, _) = space1(input)?;
        let (input, description) = take_until("\n")(input)?;
        k.description = description;
        let (input, _) = line_ending(input)?;
        let (input, _) = many0(alt((
            map(take_depends,     |val| push_optvec(&mut k.depends, val)),
            map(take_comment,     |_|   {}),
            map(take_line_ending, |_|   {}),
        )))(input)?;
        Ok((input, k))
    }
}

#[derive(Debug, Default)]
pub struct KOption<'a> {
    pub name:         &'a str,
    pub range:        Option<&'a str>,
    option_type:  OptionType,
    pub description:  Option<&'a str>,
    pub depends:      Option<Vec<&'a str>>,
    pub selects:      Option<Vec<&'a str>>,
    pub help:         Option<&'a str>,
    pub def_bool:     Option<Vec<&'a str>>,
    pub def_tristate: Option<Vec<&'a str>>,
    pub implies:      Option<Vec<&'a str>>,
    pub defaults:     Option<Vec<&'a str>>,
    pub prompt:       Option<&'a str>, // NOTE: See SECCOMP option in arch/Kconfig; this might be a bug?
}

impl<'a> KOption<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let mut k = Self::default();

        let (input, _) = space0(input)?;
        let (input, _) = alt((tag("config"), tag("menuconfig")))(input)?;
        let (input, _) = space1(input)?;
        let (input, name) = take_name(input)?;
        k.name = name;
        let (input, _) = many1(alt((
            map(take_default,      |val| push_optvec(&mut k.defaults, val)),
            map(take_depends,      |val| push_optvec(&mut k.depends, val)),
            map(take_selects,      |val| push_optvec(&mut k.selects, val)),
            map(take_imply,        |val| push_optvec(&mut k.implies, val)),
            map(take_def_bool,     |val| push_optvec(&mut k.def_bool,val)),
            map(take_def_tristate, |val| push_optvec(&mut k.def_tristate, val)),
            map(take_range,        |val| k.range  = Some(val)),
            map(take_help,         |val| k.help   = Some(val)),
            map(take_prompt,       |val| k.prompt = Some(val)),
            map(take_comment,      |_|   {}),
            map(take_line_ending,  |_|   {}),
            map(take_type,         |(opttype, desc)| {
                k.option_type = opttype;
                let desc = desc.trim_end(); // NOTE: This feels suspect here... this whole block does
                if !desc.is_empty() {
                    k.description = Some(desc);
                }
            }),
            map(tuple((space1, tag("modules"))), |_| {}), // NOTE: only shows up once in MODULES option
        )))(input)?;

        if k.option_type == OptionType::Uninitialized {
            if let Some(_) = k.def_bool {
                k.option_type = OptionType::Bool;
            } else if let Some(_) = k.def_tristate {
                k.option_type = OptionType::Tristate;
            } else {
                k.option_type = OptionType::Int;
                // TODO make this more correct logic....
                //panic!("option_type was never found for Option '{}'", option.name);
            };
        };

        Ok((input, k))
    }
}

impl std::fmt::Display for KOption<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // playing with macros
        macro_rules! bf {
            ($field:ident) => {
                if let Some(value) = &self.$field {
                    writeln!(f, "{: >12}: {}", stringify!($field), value)?;
                }
            };
        }
        macro_rules! df {
            ($field:ident) => {
                if let Some(value) = &self.$field {
                    writeln!(f, "{: >12}: {:?}", stringify!($field), value)?;
                }
            };
        }

        writeln!(f, "KOption {{")?;
        writeln!(f, "{: >12}: {}", "name", self.name)?;
        writeln!(f, "{: >12}: {}", "type", self.option_type)?;

        bf!(range);
        bf!(description);
        df!(depends);
        df!(selects);
        bf!(help);
        df!(def_bool);
        df!(def_tristate);
        df!(implies);
        df!(defaults);
        bf!(prompt);

        write!(f, "}}")
    }
}

fn push_optvec<T>(opt_vec: &mut Option<Vec<T>>, val: T) {
    // this pattern seems wrong to break into a function... maybe its fine
    if let Some(ref mut vec) = opt_vec {
        vec.push(val);
    } else {
        *opt_vec = Some(vec![val]);
    }
}

fn is_digit(chr: u8) -> bool {
    // matches ASCII digits 0-9
    chr >= 0x30 && chr <= 0x39
}

fn is_uppercase(chr: u8) -> bool {
    // matches ASCII uppercase letters A-Z
    chr >= 0x41 && chr <= 0x5A
}

fn is_lowercase(chr: u8) -> bool {
    // matches ASCII lowercase letters a-z
    chr >= 0x61 && chr <= 0x7A
}

// TODO: Fixup this function to match only uppercase followed by all of these matches
fn is_config_name(c: char) -> bool {
    is_uppercase(c as u8) || is_digit(c as u8) || c == '_' || is_lowercase(c as u8)
}

fn take_name(input: &str) -> IResult<&str, &str> {
    take_while1(is_config_name)(input)
}

fn parse_opttype(input: &str) -> IResult<&str, OptionType> {
    alt((
        map(tag("bool"),     |_| OptionType::Bool),
        map(tag("hex"),      |_| OptionType::Hex),
        map(tag("int"),      |_| OptionType::Int),
        map(tag("string"),   |_| OptionType::Str),
        map(tag("tristate"), |_| OptionType::Tristate),
    ))(input)
}

fn take_type(input: &str) -> IResult<&str, (OptionType, &str)> {
    let (input, _) = space0(input)?;
    let (input, opttype) = parse_opttype(input)?;
    let (input, val) = take_continued_line(input)?;
    Ok((input, (opttype, val)))
}

fn take_line_ending(input: &str) -> IResult<&str, &str> {
    recognize(many1(tuple((space0, line_ending))))(input)
}

fn take_tagged_line<'a>(input: &'a str, str_match: &str) -> IResult<&'a str, &'a str> {
    let (input, _) = tuple((space0, tag(str_match), space1))(input)?;
    take_continued_line(input)
}

fn take_mainmenu(input: &str) -> IResult<&str, &str> {
    take_tagged_line(input, "mainmenu")
}

fn take_visible(input: &str) -> IResult<&str, &str> {
    take_tagged_line(input, "visible")
}

fn take_source_kconfig(input: &str) -> IResult<&str, &str> {
    take_tagged_line(input, "source")
}

fn take_imply(input: &str) -> IResult<&str, &str> {
    take_tagged_line(input, "imply")
}

fn take_range(input: &str) -> IResult<&str, &str> {
    take_tagged_line(input, "range")
}

fn take_prompt(input: &str) -> IResult<&str, &str> {
    take_tagged_line(input, "prompt")
}

fn take_default(input: &str) -> IResult<&str, &str> {
    take_tagged_line(input, "default")
}

fn take_def_tristate(input: &str) -> IResult<&str, &str> {
    take_tagged_line(input, "def_tristate")
}

fn take_def_bool(input: &str) -> IResult<&str, &str> {
    take_tagged_line(input, "def_bool")
}

fn take_depends(input: &str) -> IResult<&str, &str> {
    take_tagged_line(input, "depends on")
}

fn take_selects(input: &str) -> IResult<&str, &str> {
    take_tagged_line(input, "select")
}

fn take_optional(input: &str) -> IResult<&str, bool> {
    map(tuple((space0, tag("optional"))), |_| true)(input)
}

fn take_comment(input: &str) -> IResult<&str, &str> {
    let (input, _) = space0(input)?;
    recognize(tuple((tag("#"), take_until("\n"))))(input)
}


fn take_continued_line(input: &str) -> IResult<&str, &str> {
    // This parser will take all bytes until it encounters a newline which is not escaped.
    let (input, _) = space0(input)?;
    recognize(alt((
        map(tag("\n"), |_| ()), // Simplest case of the first char being a newline
        map(
            many_till(
                take(1usize),
                tuple((
                    not(satisfy(|c| c == '\\')), // Make sure the next char isn't a \
                    take(1usize),                // Take whatever it was to move pos
                    peek(line_ending),           // Take only '\n' or '\r\n'
                )),
            ),
            |_| (),
        ),
    )))(input)
}

fn take_help(input: &str) -> IResult<&str, &str> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("help")(input)?;
    let (input, _) = take_line_ending(input)?;
    recognize(many_till(
        take(1usize),
        peek(tuple((
            alt((map(line_ending, |_| ()), map(eof, |_| ()))),
            alt((
                not(satisfy(|c| c == ' ' || c == '\t' || c == '\n' || c == '\r')),
                map(eof, |_| ()),
                map(tuple((multispace1, map(KChoice::parse, |_| ()))), |_| ()),
            )),
        ))),
    ))(input)
}

fn take_block(input: &str) -> IResult<&str, (&str, KConfig)> {
    let (input, _) = tuple((space0, tag("if"), space1))(input)?;
    let (input, condition) = take_continued_line(input)?;
    let (input, config) = KConfig::parse(input)?;
    let (input, _) = tag("endif")(input)?;
    Ok((input, (condition, config)))
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
                //eprintln!("SAMMAS ERROR Unprocessed input:\n{}\n", remaining);
                panic!("SAMMAS ERROR Unprocessed input:\n{}\n", remaining);
            }
            return config;
        }
        Err(error) => {
            panic!("SAMMAS ERROR Proper error:\n{:?}\n\n", error);
        }
    }
}
