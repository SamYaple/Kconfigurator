use super::{
    OptionType,
    Dependency,
    ReverseDependency,
    Range,
    Help,
    Prompt,
    util::{
        take_comment,
        take_line_ending,
        take_name,
    },
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{
        space0,
        space1,
    },
    combinator::{
        map,
        opt,
    },
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
    pub prompts:      Option<Vec<Prompt<'a>>>, // prompt exists as its own key
    pub help:         Option<Help<'a>>, // Raw help text, with leading whitespace on each line
    pub depends:      Option<Vec<Dependency<'a>>>, // These are strong dependencies
    pub selects:      Option<Vec<ReverseDependency<'a>>>, // These select options directly, avoiding the dependency graph
    pub implies:      Option<Vec<ReverseDependency<'a>>>, // This signifies a feature can provided to the implied option
    pub defaults:     Option<Vec<Dependency<'a>>>, // This gives a list of defaults to use, with optional condition

    // This is shorthand for `bool` type, then parses a `defaults`
    // as of v6.3.1, there are no options that declare def_bool more than once
    pub def_bool:     Option<Vec<Dependency<'a>>>,

    // This is shorthand for `tristate` type, then parses a `defaults`
    // as of v6.3.1, the only option that declares def_tristate more than once is `INET_DCCP_DIAG`
    pub def_tristate: Option<Vec<Dependency<'a>>>,

    // Only valid for `hex` and `int` types
    pub ranges:       Option<Vec<Range<'a>>>,
}

impl<'a> KOption<'a> {
    pub fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let mut opt_option_type = None;
        let mut opt_prompt_from_type = None;
        let mut help = None;

        let mut prompts      = vec![];
        let mut ranges       = vec![];
        let mut depends      = vec![];
        let mut selects      = vec![];
        let mut implies      = vec![];
        let mut defaults     = vec![];
        let mut def_bool     = vec![];
        let mut def_tristate = vec![];

        let type_line_parser = tuple((
            OptionType::parse,
            opt(Prompt::parse("")),
        ));

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
                    map(type_line_parser,  |(opttype, opt_prompt)| {
                        opt_option_type = Some(opttype);
                        opt_prompt_from_type = opt_prompt;
                    }),
                    map(Dependency::parse("def_bool"),      |v| def_bool.push(v)),
                    map(Dependency::parse("def_tristate"),  |v| def_tristate.push(v)),
                    map(Dependency::parse("default"),       |v| defaults.push(v)),
                    map(Dependency::parse("depends on"),    |v| depends.push(v)),
                    map(ReverseDependency::parse("select"), |v| selects.push(v)),
                    map(ReverseDependency::parse("imply"),  |v| implies.push(v)),
                    map(Range::parse("range"),              |v| ranges.push(v)),
                    map(Help::parse("help"),                |v| help = Some(v)),
                    map(Prompt::parse("prompt"),            |v| prompts.push(v)),
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
                    OptionType::Int
                }
            }
        };

        if let Some(prompt) = opt_prompt_from_type {
            prompts.push(prompt);
        }

        Ok((input, Self{
                name,
                option_type,
                help,
                ranges:       if ranges.is_empty()       { None } else { Some(ranges)       },
                depends:      if depends.is_empty()      { None } else { Some(depends)      },
                implies:      if implies.is_empty()      { None } else { Some(implies)      },
                prompts:      if prompts.is_empty()      { None } else { Some(prompts)      },
                selects:      if selects.is_empty()      { None } else { Some(selects)      },
                defaults:     if defaults.is_empty()     { None } else { Some(defaults)     },
                def_bool:     if def_bool.is_empty()     { None } else { Some(def_bool)     },
                def_tristate: if def_tristate.is_empty() { None } else { Some(def_tristate) },
        }))
    }
}

impl std::fmt::Display for KOption<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "config {}", self.name)?;
        writeln!(f, "\t{}", self.option_type)?;

        if let Some(prompts) = &self.prompts {
            for prompt in prompts {
                writeln!(f, "\tprompt {}", prompt)?;
            }
        }
        if let Some(defaults) = &self.defaults {
            for def in defaults {
                writeln!(f, "\tdefaults {}", def)?;
            }
        }
        if let Some(def_bool) = &self.def_bool {
            for def in def_bool {
                writeln!(f, "\tdef_bool {}", def)?;
            }
        }
        if let Some(def_tristate) = &self.def_tristate {
            for def in def_tristate {
                writeln!(f, "\tdef_tristate {}", def)?;
            }
        }
        if let Some(depends) = &self.depends {
            for dep in depends {
                writeln!(f, "\tdepends on {}", dep)?;
            }
        }
        if let Some(selects) = &self.selects {
            for sel in selects {
                writeln!(f, "\tselect {}", sel)?;
            }
        }
        if let Some(implies) = &self.implies {
            for imply in implies {
                writeln!(f, "\timply {}", imply)?;
            }
        }
        if let Some(ranges) = &self.ranges {
            for range in ranges {
                writeln!(f, "\trange {}", range)?;
            }
        }

        if let Some(help) = &self.help {
            writeln!(f, "\thelp")?;
            for line in &help.text {
                if line.is_empty() {
                    writeln!(f)?;
                } else {
                    write!(f, "\t  {}", line)?;
                }
            }
        }
        Ok(())
    }
}
