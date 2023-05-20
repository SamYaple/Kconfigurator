use super::{
    OptionType,
    util::{
        cleanup_raw_help,
        cleanup_raw_line,
        take_comment,
        take_def_bool,
        take_def_tristate,
        take_default,
        take_depends,
        take_help,
        take_imply,
        take_line_ending,
        take_name,
        take_prompt,
        take_range,
        take_selects,
        take_type,
    },
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{
        space0,
        space1,
    },
    combinator::map,
    multi::many1,
    sequence::{
        preceded,
        tuple,
    },
    IResult,
};

#[derive(Debug)]
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
    pub fn parse(input: &'a str) -> IResult<&'a str, Self> {
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
                    map(take_prompt, |v| {
                        if let Some(_) = prompt {
                            eprintln!("EC_prompt_overridden");
                        }
                        prompt = Some(v);
                    }),
                    map(take_type, |(opttype, desc, cond)| {
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
                range:    if range.is_empty()    { None } else { Some(range)    },
                depends:  if depends.is_empty()  { None } else { Some(depends)  },
                selects:  if selects.is_empty()  { None } else { Some(selects)  },
                implies:  if implies.is_empty()  { None } else { Some(implies)  },
                defaults: if defaults.is_empty() { None } else { Some(defaults) },
                def_bool: if def_bool.is_empty() { None } else { Some(def_bool) },
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