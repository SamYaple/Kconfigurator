mod expr;

use nom::{
    branch::alt,
    bytes::complete::{
        tag,
        take,
        take_until,
        take_while1,
    },
    character::complete::{
        anychar,
        line_ending,
        multispace1,
        satisfy,
        space0,
        space1,
    },
    combinator::{
        map,
        not,
        peek,
        recognize,
        opt,
    },
    multi::{
        many0,
        many1,
        many_till,
    },
    sequence::{
        preceded,
        tuple,
        delimited,
    },
    IResult,
};

#[derive(Debug, PartialEq, Copy, Clone, Default)]
enum OptionType {
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
    options:     Vec<KOption<'a>>,
    prompt:      &'a str,
    depends:     Option<Vec<(&'a str, Option<&'a str>)>>,
    defaults:    Option<Vec<(&'a str, Option<&'a str>)>>,
    help:        Option<&'a str>,
    optional:    bool,
    option_type: OptionType,
    description: Option<&'a str>,
    conditional: Option<&'a str>,
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
            map(take_type,            |(opttype, desc, cond)| {
                k.option_type = opttype;
                k.description = desc;
                k.conditional = cond;
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
    depends:     Option<Vec<(&'a str, Option<&'a str>)>>,
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
    depends:     Option<Vec<(&'a str, Option<&'a str>)>>,
}

impl<'a> KCommentBlock<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (input, description) = preceded(
            tuple((
                space0,
                tag("comment"),
                space1,
            )),
            take_until("\n")  // NOTE: AFAIK these comment blocks cannot be multiline
        )(input)?;

        // TODO: This pattern is better than before, but still has a smell
        let mut depends: Option<Vec<(&str, Option<&str>)>> = None;
        let (input, _) = many0(alt((
            map(take_depends,     |v| push_optvec(&mut depends, v)),
            map(take_comment,     |_| {}),
            map(take_line_ending, |_| {}),
        )))(input)?;

        Ok((input, Self {
                description,
                depends,
        }))
    }
}

#[derive(Debug, Default)]
pub struct KOption<'a> {
    pub name:         &'a str,         // This field must always exist
    option_type:      OptionType,      // While this field must exist, it may be inferred from `def_bool` or `def_tristate`
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
        let (input, name) = preceded(
            tuple((
                space0,
                alt((tag("config"), tag("menuconfig"))),
                space1,
            )),
            take_name,
        )(input)?;

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


        let (input, _) = many1(alt((
            map(take_default,      |v| defaults.push(v)),
            map(take_depends,      |v| depends.push(v)),
            map(take_selects,      |v| selects.push(v)),
            map(take_imply,        |v| implies.push(v)),
            map(take_def_bool,     |v| def_bool.push(v)),
            map(take_def_tristate, |v| def_tristate.push(v)),
            map(take_range,        |v| range.push(v)),
            map(take_help,         |v| {
                if let Some(_) = help {
                    eprintln!("EC_help_overridden {}", name);
                }
                help = Some(v);
            }),
            map(take_prompt,       |v| {
                if let Some(_) = prompt {
                    eprintln!("EC_prompt_overridden {}", name);
                }
                prompt = Some(v);
            }),
            map(take_comment,      |_| {}),
            map(take_line_ending,  |_| {}),
            map(take_type,         |(opttype, desc, cond)| {
                if let Some(option_type) = opt_option_type {
                    if option_type == opttype {
                        // This branch indicates the option has the same type declared twice
                        eprintln!("EC_type_duplicate {}", name);
                    } else {
                        // This means the type of this option CHANGED, not good at all.
                        eprintln!("EC_type_overridden {}", name);
                    }
                }
                opt_option_type = Some(opttype);

                if desc.is_some() {
                    if description.is_some() {
                        eprintln!("EC_description_overridden {}", name);
                    }
                    description = desc;
                }

                if cond.is_some() {
                    if conditional.is_some() {
                        eprintln!("EC_conditional_overridden {}", name);
                    }
                    conditional = cond;
                }
            }),
            map(tuple((space1, tag("modules"))), |_| {}), // NOTE: only shows up once in MODULES option
        )))(input)?;

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
                    eprintln!("EC_missing_type {}", name);
                    OptionType::Int
                }
            }
        };

        if def_bool.len() > 0 && option_type != OptionType::Bool {
            eprintln!("EC_type_mismatch {}", name);
        }
        if def_tristate.len() > 0 && option_type != OptionType::Tristate {
            eprintln!("EC_type_mismatch {}", name);
        }
        if range.len() > 0 && (option_type != OptionType::Int && option_type != OptionType::Hex){
            eprintln!("EC_range_in_wrong_type {}", name);
        }
        //println!("SAMMAS {}", name);
        Ok((input, Self{
                name,
                option_type,
                description,
                prompt,
                conditional,
                help,
                depends: if depends.len() > 0 { Some(depends) } else { None },
                selects: if selects.len() > 0 { Some(selects) } else { None },
                implies: if implies.len() > 0 { Some(implies) } else { None },
                defaults: if defaults.len() > 0 { Some(defaults) } else { None },
                def_bool: if def_bool.len() > 0 { Some(def_bool) } else { None },
                def_tristate: if def_tristate.len() > 0 { Some(def_tristate) } else { None },
                range: if range.len() > 0 { Some(range) } else { None },
        }))
    }
}

fn escape_quoted(input: &str) -> String {
    let mut result = String::with_capacity(input.len() + 2);
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
        macro_rules! print_if_some_list {
            ($field:ident) => {
                if let Some(values) = &self.$field {
                    writeln!(f, "      {}:", stringify!($field))?;
                    for val in values {
                        let quoted = escape_quoted(&cleanup_raw_line(val));
                        writeln!(f, "        - {}", quoted)?;
                    }
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

fn count_whitespace(s: &str) -> usize {
    let mut count = 0;
    for c in s.chars() {
        if c == '\t' {
            let spaces = 8 - (count % 8);
            count += spaces;
        } else {
            count += 1;
        }
    }
    count
}

fn prefix_spaces(n: usize) -> String {
    let mut result = String::with_capacity(n);
    for _ in 0..n {
        result.push(' ');
    }
    result
}

fn cleanup_raw_help(text: &str) -> String {
    // Preserve the whitespace structure while trimming the text in the help functions
    let init_ws = count_whitespace(text);
    let mut help = String::new();
    for l in text.split('\n') {
        let line_ws = count_whitespace(l);
        if line_ws < init_ws {
            help += l.trim_start();
            help += "\n";
        } else {
            let padding = line_ws - init_ws;
            help += &prefix_spaces(padding);
            help += l.trim_start();
            help += "\n";
        }
    }
    help.trim_end().to_string()
}

fn take_help(input: &str) -> IResult<&str, &str> {
    let (input, _) = space0(input)?;
    let (input, _) = tag("help")(input)?;
    let (input, _) = many1(tuple((space0, line_ending)))(input)?;
    let (_, raw_ws) = space1(input)?;
    let ws = count_whitespace(raw_ws);
    recognize(many1(alt((
        tag("\n"),
        take_while_help(ws),
    ))))(input)
}

fn take_while_help(min_ws: usize) -> impl Fn(&str) -> IResult<&str, &str> {
    move |input: &str| -> IResult<&str, &str> {
        // First we need to record the amount of whitespace (tab==8, space=1) on the initial line
        let (_, raw_ws) = space1(input)?;
        let ws = count_whitespace(raw_ws);

        // Now we will take all characters from that line until and including the '\n' or EOF
        let (input, line) = preceded(
            space0,
            alt((
                take_until("\n"),          // Take until newline
                recognize(many1(anychar)), // There is no '\n', take any char until eof
            )),
        )(input)?;
        if ws < min_ws && !line.is_empty() { // if current line `ws` is less that `min_ws` the block has ended
            Err(nom::Err::Error(
                nom::error::Error{
                    input: input,
                    code: nom::error::ErrorKind::Tag,
                }
            ))
        } else {
            Ok((input, line))
        }
    }
}

fn cleanup_raw_line(text: &str) -> String {
    let mut result = String::new();
    for l in text.split('\n') {
        let mut cleaned_line = l.trim_start().to_string();
        if cleaned_line.chars().last() == Some('\\') {
            cleaned_line.pop();
        }
        if !result.is_empty() {
            result.push_str(&" ");
        }
        result.push_str(&cleaned_line.trim_end());
    }
    result
}

fn push_optvec<T>(opt_vec: &mut Option<Vec<T>>, val: T) {
    // this pattern seems wrong to break into a function... maybe its fine
    if let Some(ref mut vec) = opt_vec {
        vec.push(val);
    } else {
        *opt_vec = Some(vec![val]);
    }
}

fn is_hex(chr: u8) -> bool {
    // matches ASCII digits A-Fa-f0-9
    (chr >= 0x41 && chr <= 0x46) || (chr >= 0x61 && chr <= 0x66) || (chr >= 0x30 && chr <= 0x39)
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

fn parse_kstring(input: &str) -> IResult<&str, &str> {
    // NOTE: this will take newlines and other chars which are not valid
    preceded(
        space0,
        alt((
            // we check for double qoute strings, careful to preserve escaped \"
            delimited(
                tag("\""),
                recognize(many0(alt((
                    take_while1(|c| c != '\\' && c != '"'),
                    tag("\\\""), // We have encoutered an escaped quote -- \"
                    tag("\\"),   // We didn't find an end to our string, take -- \\
                )))),
                tag("\""),
            ),
            // we check for single qoute strings, careful to preserve escaped \'
            delimited(
                tag("'"),
                recognize(many0(alt((
                    take_while1(|c| c != '\\' && c != '\''),
                    tag("\\'"), // We have encoutered an escaped quote -- \'
                    tag("\\"),  // We didn't find an end to our string, take -- \\
                )))),
                tag("'"),
            ),
        ))
    )(input)
}

fn take_type(input: &str) -> IResult<&str, (OptionType, Option<&str>, Option<&str>)> {
    let (input, _) = space0(input)?;
    let (input, opttype) = parse_opttype(input)?;
    let (input, description) = opt(parse_kstring)(input)?;
    let (input, conditional) = opt(take_cond)(input)?;
    Ok((input, (opttype, description, conditional)))
}

fn take_line_ending(input: &str) -> IResult<&str, &str> {
    recognize(many1(tuple((space0, line_ending))))(input)
}

fn take_tagged_line<'a>(input: &'a str, str_match: &str) -> IResult<&'a str, &'a str> {
    let (input, _) = tuple((space0, tag(str_match), space1))(input)?;
    take_continued_line(input)
}

fn take_named_line<'a>(input: &'a str, str_match: &str) -> IResult<&'a str, (&'a str, Option<&'a str>)> {
    let (input, expr) = preceded(
        tuple((
            space0,
            tag(str_match),
            space1,
        )),
        recognize(take_expr),
    )(input)?;
    let (input, cond) = opt(take_cond)(input)?;
    Ok((input, (expr, cond)))
}

fn take_mainmenu(input: &str) -> IResult<&str, &str> {
    take_tagged_line(input, "mainmenu")
}

fn take_source_kconfig(input: &str) -> IResult<&str, &str> {
    take_tagged_line(input, "source")
}

fn take_visible(input: &str) -> IResult<&str, &str> {
    let (input, _) = tuple((
        space0,
        tag("visible if"),
        space1,
    ))(input)?;
    let (input, cond) = recognize(take_expr)(input)?;
    Ok((input, cond))
}

fn take_imply(input: &str) -> IResult<&str, (&str, Option<&str>)> {
    take_named_line(input, "imply")
}

fn take_default(input: &str) -> IResult<&str, (&str, Option<&str>)> {
    take_named_line(input, "default")
}

fn take_def_tristate(input: &str) -> IResult<&str, (&str, Option<&str>)> {
    take_named_line(input, "def_tristate")
}

fn take_def_bool(input: &str) -> IResult<&str, (&str, Option<&str>)> {
    take_named_line(input, "def_bool")
}

fn take_depends(input: &str) -> IResult<&str, (&str, Option<&str>)> {
    take_named_line(input, "depends on")
}

fn take_range(input: &str) -> IResult<&str, ((&str, &str), Option<&str>)> {
    let (input, _) = tuple((
        space0,
        tag("range"),
        space1,
    ))(input)?;
    let (input, (start, _, end)) = alt((
        tuple((expr::take_signed_int, space1, expr::take_signed_int)),
        tuple((expr::take_hex, space1,  expr::take_hex)),
        tuple((take_name, space1, take_name)),
    ))(input)?;
    let (input, cond) = opt(take_cond)(input)?;
    Ok((input, ((start, end), cond)))
}

fn take_prompt(input: &str) -> IResult<&str, &str> {
    take_tagged_line(input, "prompt")
}

fn take_expr(input: &str) -> IResult<&str, expr::Expr> {
    expr::expr(input)
}

fn take_cond(input: &str) -> IResult<&str, &str> {
    preceded(
        tuple((
            expr::special_space,
            tag("if"),
            expr::special_space,
        )),
        recognize(take_expr)
    )(input)
}

fn take_selects(input: &str) -> IResult<&str, (&str, Option<&str>)> {
    take_named_line(input, "select")
}

fn take_optional(input: &str) -> IResult<&str, bool> {
    map(tuple((space0, tag("optional"))), |_| true)(input)
}

fn take_comment(input: &str) -> IResult<&str, &str> {
    let (input, _) = space0(input)?;
    recognize(tuple((tag("#"), take_until("\n"))))(input)
}

fn take_continued_line(input: &str) -> IResult<&str, &str> {
    // This parser will take all bytes until it encounters a newline which is not escaped. or a
    // comment
    let (input, _) = space0(input)?;
    recognize(alt((
        map(tag("\n"), |_| ()), // Simplest case of the first char being a newline
        map(
            many_till(
                take(1usize),
                alt((
                    peek(tag("#")), // This is now a comment block and we can exit
                    recognize(tuple((
                        not(satisfy(|c| c == '\\')), // Make sure the next char isn't a \
                        take(1usize),                // Take whatever it was to move pos
                        peek(line_ending),           // Exit many_till if the next char is a newline
                    ))),
                )),
            ),
            |_| (),
        ),
    )))(input)
}

fn take_block(input: &str) -> IResult<&str, (&str, KConfig)> {
    let (input, condition) = take_cond(input)?;
    let (input, _) = multispace1(input)?;
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
                panic!("SAMMAS ERROR Unprocessed input:```\n{}'\n```", remaining);
            }
            return config;
        }
        Err(error) => {
            panic!("SAMMAS ERROR Proper error:\n{:?}\n\n", error);
        }
    }
}
