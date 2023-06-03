mod kconfig;
//mod json;

use std::fmt::{
    Display,
    Formatter,
    Result,
};

use crate::kconfig::{
    Block,
    ConstantSymbol,
    Dependency,
    Expression,
    Help,
    Hex,
    Int,
    KChoice,
    KCommentBlock,
    KConfig,
    KMenu,
    KOption,
    OptionType,
    Prompt,
    Range,
    RangeType,
    Symbol,
};

trait DisplayKConfig {
    fn display_kconfig(&self) -> String;
}

//trait DisplayJSON {
//    fn display_json(&self) -> String;
//}

impl Display for KConfig<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.display_kconfig())
    }
}

impl Display for OptionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.display_kconfig())
    }
}

impl Display for Symbol<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.display_kconfig())
    }
}

impl Display for ConstantSymbol<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.display_kconfig())
    }
}

impl Display for Expression<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.display_kconfig())
    }
}

impl Display for Prompt<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.display_kconfig())
    }
}

impl Display for Dependency<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.display_kconfig())
    }
}

impl Display for Int {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.display_kconfig())
    }
}

impl Display for Hex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.display_kconfig())
    }
}

impl Display for RangeType<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.display_kconfig())
    }
}

impl Display for Range<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.display_kconfig())
    }
}

impl Display for Help<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.display_kconfig())
    }
}

impl Display for KCommentBlock<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.display_kconfig())
    }
}

impl Display for KOption<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.display_kconfig())
    }
}

impl Display for KMenu<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.display_kconfig())
    }
}

impl Display for KChoice<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.display_kconfig())
    }
}

impl Display for Block<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.display_kconfig())
    }
}
