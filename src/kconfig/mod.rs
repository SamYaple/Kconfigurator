use nom::branch::alt;
use nom::bytes::complete::{tag, take, take_until, take_while1};
use nom::character::complete::{line_ending, multispace0, multispace1, satisfy, space0, space1};
use nom::combinator::{eof, map, not, peek, recognize};
use nom::multi::{many0, many1, many_till};
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
enum ConfigToken {
    Block((String, KConfig)),
    Comment(String),
    KChoice(KChoice),
    KMenu(KMenu),
    KOption(KOption),
    NewLine,
    Source(String),
    MainMenu(String),
    KCommentBlock(KCommentBlock),
}

#[derive(Debug, Default)]
pub struct KConfig<'a> {
    pub options:  Option<Vec<KOption<'a>>>,
    choices:  Option<Vec<KChoice<'a>>>,
    configs:  Option<Vec<&'a str>>,
    blocks:   Option<Vec<(&'a str, KConfig<'a>)>>,
    menus:    Option<Vec<KMenu<'a>>>,
    mainmenu: Option<&'a str>,
}

impl KConfig<'_> {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, tokens) = many1(alt((
            map(KCommentBlock::parse, |val| ConfigToken::KCommentBlock(val)),
            map(KOption::parse,       |val| ConfigToken::KOption(val)),
            map(KMenu::parse,         |val| ConfigToken::KMenu(val)),
            map(KChoice::parse,       |val| ConfigToken::KChoice(val)),
            map(take_source_kconfig,  |val| ConfigToken::Source(val)),
            map(take_mainmenu,        |val| ConfigToken::MainMenu(val)),
            map(take_comment,         |val| ConfigToken::Comment(val)),
            map(take_block,           |val| ConfigToken::Block(val)),
            map(take_line_ending,     |_|   ConfigToken::NewLine),
        )))(input)?;

        let mut k = Self::default();
        for token in tokens {
            match token {
                ConfigToken::Block(block)    => push_optvec(&mut k.blocks,  block),
                ConfigToken::KChoice(choice) => push_optvec(&mut k.choices, choice),
                ConfigToken::KOption(option) => push_optvec(&mut k.options, option),
                ConfigToken::KMenu(menu)     => push_optvec(&mut k.menus,   menu),
                ConfigToken::Source(path)    => push_optvec(&mut k.configs, path),
                ConfigToken::MainMenu(text)  => k.mainmenu = Some(text),
                ConfigToken::Comment(_)       => {} //println!("comment found: '{}'", comment),
                ConfigToken::NewLine          => {}
                ConfigToken::KCommentBlock(_) => {} // TODO something with this block?
            };
        }
        Ok((input, k))
    }
}

#[derive(Debug)]
enum ChoiceToken {
    Comment(String),
    Depend(String),
    Help(String),
    KOption(KOption),
    NewLine,
    OptionDefault(String),
    Optional,
    Prompt(String),
    OptionType((OptionType, Option<String>)),
    KCommentBlock(KCommentBlock),
}

#[derive(Debug, Default)]
pub struct KChoice {
    options:     Vec<KOption>,
    prompt:      String,
    depends:     Option<Vec<String>>,
    help:        Option<String>,
    optional:    bool,
    option_type: OptionType,
    description: Option<String>,
}

impl KChoice {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, _) = space0(input)?;
        let (input, _) = tag("choice")(input)?;
        let (input, _) = space0(input)?;
        let (input, _) = many1(line_ending)(input)?;
        let (input, tokens) = many1(alt((
            map(KCommentBlock::parse, |cb| ChoiceToken::KCommentBlock(cb)),
            map(take_optional, |_| ChoiceToken::Optional),
            map(KOption::parse, |option| ChoiceToken::KOption(option)),
            map(take_comment, |text| ChoiceToken::Comment(text.to_string())),
            map(take_help, |help| ChoiceToken::Help(help.to_string())),
            map(take_depends, |depend| {
                ChoiceToken::Depend(depend.to_string())
            }),
            map(take_default, |default| {
                ChoiceToken::OptionDefault(default.to_string())
            }),
            map(take_prompt, |prompt| {
                ChoiceToken::Prompt(prompt.to_string())
            }),
            map(take_line_ending, |_| ChoiceToken::NewLine),
            map(take_type, |type_desc| ChoiceToken::OptionType(type_desc)),
        )))(input)?;
        let (input, _) = space0(input)?;
        let (input, _) = tag("endchoice")(input)?;

        let mut kchoice = Self::default();
        let mut depends: Vec<String> = vec![];
        for token in tokens {
            match token {
                ChoiceToken::Comment(_) => {} // println!("comment found: '{}'", comment),
                ChoiceToken::Depend(dep) => depends.push(dep),
                ChoiceToken::Help(help) => kchoice.help = Some(help),
                ChoiceToken::Optional => kchoice.optional = true,
                ChoiceToken::Prompt(msg) => kchoice.prompt = msg,
                ChoiceToken::OptionDefault(val) => depends.push(val),
                ChoiceToken::KOption(option) => kchoice.options.push(option),
                ChoiceToken::NewLine => {}
                ChoiceToken::KCommentBlock(_) => {} // TODO something with this block?
                ChoiceToken::OptionType((opttype, description)) => {
                    kchoice.description = description;
                    kchoice.option_type = opttype;
                }
            };
        }
        if !depends.is_empty() {
            kchoice.depends = Some(depends);
        }
        Ok((input, kchoice))
    }
}

#[derive(Debug)]
enum MenuToken {
    //Description(&str),
    Block((String, KConfig)),
    Comment(String),
    KCommentBlock(KCommentBlock),
    KChoice(KChoice),
    KMenu(KMenu),
    KOption(KOption),
    NewLine,
    Source(String),
    Depend(String),
    Visible(String),
}

#[derive(Debug, Default)]
pub struct KMenu {
    options:     Option<Vec<KOption>>,
    menus:       Option<Vec<KMenu>>,
    choices:     Option<Vec<KChoice>>,
    configs:     Option<Vec<String>>,
    blocks:      Option<Vec<(String, KConfig)>>,
    description: String,
    depends:     Option<Vec<String>>,
    visible:     Option<Vec<String>>,
}

impl KMenu {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, _) = tag("menu")(input)?;
        let (input, description) = parse_description(input)?;
        let (input, tokens) = many1(alt((
            map(KMenu::parse, |menu| MenuToken::KMenu(menu)),
            map(KCommentBlock::parse, |cb| MenuToken::KCommentBlock(cb)),
            map(KOption::parse, |option| MenuToken::KOption(option)),
            map(KChoice::parse, |choice| MenuToken::KChoice(choice)),
            map(take_depends, |depend| MenuToken::Depend(depend.to_string())),
            map(take_source_kconfig, |path| {
                MenuToken::Source(path.to_string())
            }),
            map(take_comment, |text| MenuToken::Comment(text.to_string())),
            map(take_visible, |cond| MenuToken::Visible(cond.to_string())),
            map(take_line_ending, |_| MenuToken::NewLine),
            map(take_block, |(cond, config)| {
                MenuToken::Block((cond.to_string(), config))
            }),
        )))(input)?;
        //println!("hello {}", input);
        let (input, _) = tag("endmenu")(input)?;

        let mut kmenu = Self::default();
        let mut menus: Vec<KMenu> = vec![];
        let mut depends: Vec<String> = vec![];
        let mut choices: Vec<KChoice> = vec![];
        let mut options: Vec<KOption> = vec![];
        let mut configs: Vec<String> = vec![];
        let mut visibles: Vec<String> = vec![];
        let mut blocks: Vec<(String, KConfig)> = vec![];
        if let Some(text) = description {
            kmenu.description = text.to_string();
        };
        for token in tokens {
            match token {
                MenuToken::Block(block) => blocks.push(block),
                MenuToken::KChoice(choice) => choices.push(choice),
                MenuToken::KMenu(menu) => menus.push(menu),
                MenuToken::Comment(_) => {}
                MenuToken::KCommentBlock(_) => {} // TODO something with this block?
                MenuToken::KOption(option) => options.push(option),
                MenuToken::Visible(vis) => visibles.push(vis),
                MenuToken::NewLine => {}
                MenuToken::Depend(dep) => depends.push(dep),
                //MenuToken::Description(text) => kmenu.description = text,
                MenuToken::Source(path) => {
                    configs.push(path);
                }
                //MenuToken::Source(path) => {
                //    let content = load_from_file(path);
                //    let config = take_kconfig(&content);
                //    configs.push(config);
                //},
            };
        }
        if !visibles.is_empty() {
            kmenu.visible = Some(visibles);
        }
        if !depends.is_empty() {
            kmenu.depends = Some(depends);
        }
        if !choices.is_empty() {
            kmenu.choices = Some(choices);
        }
        if !menus.is_empty() {
            kmenu.menus = Some(menus);
        }
        if !options.is_empty() {
            kmenu.options = Some(options);
        }
        if !configs.is_empty() {
            kmenu.configs = Some(configs);
        }
        if !blocks.is_empty() {
            kmenu.blocks = Some(blocks);
        }
        Ok((input, kmenu))
    }
}

#[derive(Debug)]
enum CommentBlockToken {
    Depend(String),
    NewLine,
}

#[derive(Debug, Default)]
pub struct KCommentBlock {
    description: String,
    depends:     Option<Vec<String>>,
}

impl KCommentBlock {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, _) = space0(input)?;
        let (input, _) = tag("comment")(input)?;
        let (input, _) = space1(input)?;
        let (input, description) = take_until("\n")(input)?;
        let (input, _) = line_ending(input)?;

        let (input, tokens) = many0(alt((
            map(take_depends, |depend| {
                CommentBlockToken::Depend(depend.to_string())
            }),
            map(take_line_ending, |_| CommentBlockToken::NewLine),
        )))(input)?;

        let mut option = Self::default();

        let mut depends: Vec<String> = vec![];

        for token in tokens {
            match token {
                CommentBlockToken::Depend(dep) => depends.push(dep),
                CommentBlockToken::NewLine => {}
            };
        }

        if !depends.is_empty() {
            option.depends = Some(depends);
        }
        option.description = description.to_string();

        Ok((input, option))
    }
}

#[derive(Debug)]
enum OptionToken {
    Comment(String),
    DefBool(String),
    DefTristate(String),
    Depend(String),
    Help(String),
    Imply(String),
    NewLine,
    OptionDefault(String),
    OptionType((OptionType, Option<String>)),
    Range(String),
    Select(String),
    Prompt(String),
}

#[derive(Debug, PartialEq, Default)]
enum OptionType {
    #[default]
    Uninitialized,
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
            OptionType::Bool => write!(f, "bool"),
            OptionType::Tristate => write!(f, "tristate"),
            OptionType::Str => write!(f, "str"),
            OptionType::Int => write!(f, "int"),
            OptionType::Hex => write!(f, "hex"),
        }
    }
}

#[derive(Debug, Default)]
pub struct KOption {
    name:         String,
    range:        Option<String>,
    option_type:  OptionType,
    description:  Option<String>,
    depends:      Option<Vec<String>>,
    selects:      Option<Vec<String>>,
    help:         Option<String>,
    def_bool:     Option<Vec<String>>,
    def_tristate: Option<Vec<String>>,
    implies:      Option<Vec<String>>,
    defaults:     Option<Vec<String>>,
    prompt:       Option<String>, // NOTE: See SECCOMP option in arch/Kconfig; this might be a bug?
}

impl std::fmt::Display for KOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "KOption {{")?;
        writeln!(f, "  name:         {}", self.name)?;
        writeln!(f, "  option_type:  {}", self.option_type)?;

        if let Some(range) = &self.range {
            writeln!(f, "  range:        {}", range)?;
        }
        if let Some(description) = &self.description {
            writeln!(f, "  description:  {}", description)?;
        }
        if let Some(depends) = &self.depends {
            writeln!(f, "  depends:      {:?}", depends)?;
        }
        if let Some(selects) = &self.selects {
            writeln!(f, "  selects:      {:?}", selects)?;
        }
        if let Some(help) = &self.help {
            writeln!(f, "  help:         {}", help)?;
        }
        if let Some(def_bool) = &self.def_bool {
            writeln!(f, "  def_bool:     {:?}", def_bool)?;
        }
        if let Some(def_tristate) = &self.def_tristate {
            writeln!(f, "  def_tristate: {:?}", def_tristate)?;
        }
        if let Some(implies) = &self.implies {
            writeln!(f, "  implies:      {:?}", implies)?;
        }
        if let Some(defaults) = &self.defaults {
            writeln!(f, "  defaults:     {:?}", defaults)?;
        }
        if let Some(prompt) = &self.prompt {
            writeln!(f, "  prompt:       {}", prompt)?;
        }

        write!(f, "}}")
    }
}

impl KOption {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, _) = space0(input)?;
        let (input, _) = alt((tag("config"), tag("menuconfig")))(input)?;
        let (input, _) = space1(input)?;
        let (input, name) = take_name(input)?;
        let (input, tokens) = many1(alt((
            map(take_prompt, |prompt| {
                OptionToken::Prompt(prompt.to_string())
            }),
            map(take_comment, |text| OptionToken::Comment(text.to_string())),
            map(take_default, |default| {
                OptionToken::OptionDefault(default.to_string())
            }),
            map(take_depends, |depend| {
                OptionToken::Depend(depend.to_string())
            }),
            map(take_help, |help| OptionToken::Help(help.to_string())),
            map(take_imply, |imply| OptionToken::Imply(imply.to_string())),
            map(take_range, |range| OptionToken::Range(range.to_string())),
            map(take_selects, |select| {
                OptionToken::Select(select.to_string())
            }),
            map(take_def_bool, |def_bool| {
                OptionToken::DefBool(def_bool.to_string())
            }),
            map(take_def_tristate, |def_tristate| {
                OptionToken::DefTristate(def_tristate.to_string())
            }),
            map(take_type, |type_desc| OptionToken::OptionType(type_desc)),
            map(take_line_ending, |_| OptionToken::NewLine),
            map(tuple((space1, tag("modules"))), |_| OptionToken::NewLine), // TODO: do something proper with this
        )))(input)?;

        let mut option = Self::default();

        let mut implies: Vec<String> = vec![];
        let mut depends: Vec<String> = vec![];
        let mut selects: Vec<String> = vec![];
        let mut defaults: Vec<String> = vec![];
        let mut def_bools: Vec<String> = vec![];
        let mut def_tristates: Vec<String> = vec![];

        for token in tokens {
            match token {
                OptionToken::Prompt(prompt) => option.prompt = Some(prompt),
                OptionToken::Imply(imply) => implies.push(imply),
                OptionToken::Help(help) => option.help = Some(help),
                OptionToken::Depend(dep) => depends.push(dep),
                OptionToken::Select(sel) => selects.push(sel),
                OptionToken::Range(range) => option.range = Some(range),
                OptionToken::DefBool(val) => def_bools.push(val),
                OptionToken::DefTristate(val) => def_tristates.push(val),
                OptionToken::Comment(_) => {}
                OptionToken::NewLine => {}
                OptionToken::OptionDefault(val) => defaults.push(val),
                OptionToken::OptionType((opttype, description)) => {
                    option.description = description;
                    option.option_type = opttype;
                }
            };
        }
        option.name = name.to_string();

        if !def_bools.is_empty() {
            option.def_bool = Some(def_bools);
        }
        if !def_tristates.is_empty() {
            option.def_tristate = Some(def_tristates);
        }
        if !depends.is_empty() {
            option.depends = Some(depends);
        }
        if !selects.is_empty() {
            option.selects = Some(selects);
        }
        if !defaults.is_empty() {
            option.defaults = Some(defaults);
        }
        if !implies.is_empty() {
            option.implies = Some(implies);
        }

        if option.option_type == OptionType::Uninitialized {
            if let Some(_) = option.def_bool {
                option.option_type = OptionType::Bool;
            } else if let Some(_) = option.def_tristate {
                option.option_type = OptionType::Tristate;
            } else {
                option.option_type = OptionType::Int;
                // TODO make this more correct logic....
                //panic!("option_type was never found for Option '{}'", option.name);
            };
        };

        Ok((input, option))
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
fn is_valid_config_name(c: char) -> bool {
    is_uppercase(c as u8) || is_digit(c as u8) || c == '_' || is_lowercase(c as u8)
}

fn parse_opttype(input: &str) -> IResult<&str, OptionType> {
    alt((
        map(tag("tristate"), |_| OptionType::Tristate),
        map(tag("string"), |_| OptionType::Str),
        map(tag("bool"), |_| OptionType::Bool),
        map(tag("int"), |_| OptionType::Int),
        map(tag("hex"), |_| OptionType::Hex),
    ))(input)
}

fn parse_description(input: &str) -> IResult<&str, Option<String>> {
    let (input, _) = space0(input)?;
    let (input, val) = take_continued_line(input)?;
    let description = if val == "" {
        None
    } else {
        Some(val.to_string())
    };
    Ok((input, description))
}

fn take_line_ending(input: &str) -> IResult<&str, &str> {
    let (input, _) = many1(tuple((space0, line_ending)))(input)?;
    Ok((input, ""))
}

fn take_line_beginning(input: &str) -> IResult<&str, &str> {
    space0(input)
}

fn take_type(input: &str) -> IResult<&str, (OptionType, Option<String>)> {
    let (input, _) = take_line_beginning(input)?;
    let (input, opttype) = parse_opttype(input)?;
    let (input, val) = take_continued_line(input)?;
    let description = if val == "" {
        None
    } else {
        Some(val.to_string())
    };
    Ok((input, (opttype, description)))
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

fn take_name(input: &str) -> IResult<&str, &str> {
    take_while1(is_valid_config_name)(input)
}

fn take_continued_line(input: &str) -> IResult<&str, &str> {
    // This parser will take all bytes until it encounters a newline which is not escaped.
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
    let (input, help) = recognize(many_till(
        take(1usize),
        peek(tuple((
            alt((map(line_ending, |_| ()), map(eof, |_| ()))),
            alt((
                not(satisfy(|c| c == ' ' || c == '\t' || c == '\n' || c == '\r')),
                map(eof, |_| ()),
                map(tuple((multispace1, map(KChoice::parse, |_| ()))), |_| ()),
            )),
        ))),
    ))(input)?;

    Ok((input, help))
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

pub fn take_kconfig(input: &str) -> KConfig {
    //let (remaining, config) = KConfig::parse(input).unwrap();
    match KConfig::parse(input) {
        Ok((remaining, config)) => {
            if remaining != "" {
                eprintln!("SAMMAS ERROR Unprocessed input:\n{}\n", remaining);
                eprintln!("SAMMAS ERROR Unprocessed input");
            }
            return config;
        }
        Err(error) => {
            eprintln!("SAMMAS ERROR Proper error:\n{:?}\n\n", error);
            KConfig::default()
        }
    }
}

fn push_optvec<T>(opt_vec: &mut Option<Vec<T>>, val: T) {
    if let Some(ref mut vec) = opt_vec {
        vec.push(val);
    } else {
        *opt_vec = Some(vec![val]);
    }
}
