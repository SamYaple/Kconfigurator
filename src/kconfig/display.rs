use std::fmt::{
    Display,
    Formatter,
    Result,
};

use super::{
    KOption,
    KCommentBlock,
    OptionType,
    Symbol,
    Expression,
    Condition,
    Prompt,
    Dependency,
    Range,
    RangeType,
    Help,
    Int,
    Hex,
};

impl Display for OptionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            OptionType::Tristate => write!(f, "tristate"),
            OptionType::Bool     => write!(f, "bool"),
            OptionType::Hex      => write!(f, "hex"),
            OptionType::Int      => write!(f, "int"),
            OptionType::Str      => write!(f, "str"),
        }
    }
}

impl Display for Symbol<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.name)?;
        Ok(())
    }
}

impl Display for Expression<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.val)?;
        Ok(())
    }
}

impl Display for Condition<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "if {}", self.expression)?;
        Ok(())
    }
}

impl Display for Prompt<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.text)?;
        if let Some(condition) = &self.condition {
            write!(f, " {}", condition)?;
        }
        Ok(())
    }
}

impl Display for Dependency<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.expression)?;
        if let Some(condition) = &self.condition {
            write!(f, " {}", condition)?;
        }
        Ok(())
    }
}

impl Display for Int {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.val)?;
        Ok(())
    }
}

impl Display for Hex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.val)?;
        Ok(())
    }
}

impl Display for RangeType<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            RangeType::Int(v)    => write!(f, "{}", v)?,
            RangeType::Hex(v)    => write!(f, "{}", v)?,
            RangeType::Symbol(v) => write!(f, "{}", v)?,
        }
        Ok(())
    }
}

impl Display for Range<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{} {}", self.start, self.end)?;
        if let Some(condition) = &self.condition {
            write!(f, " {}", condition)?;
        }
        Ok(())
    }
}

impl Display for Help<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for line in &self.text {
            write!(f, "  {}", line)?;
        }
        Ok(())
    }
}

impl Display for KCommentBlock<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        // TODO: make sure prompt gets wrapped in quotes without hardcode
        writeln!(f, "comment {}", self.prompt)?;
        if let Some(depends) = &self.depends {
            for dep in depends {
                writeln!(f, "\t{}", dep)?;
            }
        }
        Ok(())
    }
}

impl Display for KOption<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
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
