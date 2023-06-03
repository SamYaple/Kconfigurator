use crate::kconfig::{
    KOption,
    KMenu,
    Block,
    KChoice,
    KConfig,
    KCommentBlock,
    OptionType,
    Symbol,
    Expression,
    Prompt,
    Dependency,
    Range,
    RangeType,
    Help,
    Int,
    Hex,
    Delimiter,
    ConstantSymbol,
};

use super::DisplayKConfig;

impl DisplayKConfig for OptionType {
    fn display_kconfig(&self) -> String {
        match self {
            OptionType::Tristate => format!("tristate"),
            OptionType::Bool     => format!("bool"),
            OptionType::Hex      => format!("hex"),
            OptionType::Int      => format!("int"),
            OptionType::Str      => format!("str"),
        }
    }
}

impl DisplayKConfig for Symbol<'_> {
    fn display_kconfig(&self) -> String {
        format!("{}", self.name)
    }
}

impl DisplayKConfig for ConstantSymbol<'_> {
    fn display_kconfig(&self) -> String {
        match self.delimiter {
            Delimiter::DoubleQuote(content)       => format!("\"{}\"", content),
            Delimiter::SingleQuote(content)       => format!("'{}'",   content),
            //Delimiter::Parentheses(content)       => format!("({})",   content),
            //Delimiter::DollarParentheses(content) => format!("$({})",  content),
        }
    }
}

impl DisplayKConfig for Expression<'_> {
    fn display_kconfig(&self) -> String {
        format!("{}", self.val)
    }
}

impl DisplayKConfig for Prompt<'_> {
    fn display_kconfig(&self) -> String {
        let mut ret = format!("{}", self.text);
        if let Some(condition) = &self.condition {
            ret = format!("{} if {}", ret, condition);
        };
        ret
    }
}

impl DisplayKConfig for Dependency<'_> {
    fn display_kconfig(&self) -> String {
        let mut ret = format!("{}", self.expression);
        if let Some(condition) = &self.condition {
            ret = format!("{} if {}", ret, condition);
        }
        ret
    }
}

impl DisplayKConfig for Int {
    fn display_kconfig(&self) -> String {
        format!("{}", self.val)
    }
}

impl DisplayKConfig for Hex {
    fn display_kconfig(&self) -> String {
        format!("{}", self.val)
    }
}

impl DisplayKConfig for RangeType<'_> {
    fn display_kconfig(&self) -> String {
        match self {
            RangeType::Int(v)    => format!("{}", v),
            RangeType::Hex(v)    => format!("{}", v),
            RangeType::Symbol(v) => format!("{}", v),
        }
    }
}

impl DisplayKConfig for Range<'_> {
    fn display_kconfig(&self) -> String {
        let mut ret = format!("{} {}", self.start, self.end);
        if let Some(condition) = &self.condition {
            ret = format!("{} if {}", ret, condition);
        }
        ret
    }
}

impl DisplayKConfig for Help<'_> {
    fn display_kconfig(&self) -> String {
        let mut ret = String::new();
        for line in &self.text {
            ret = format!("{}\n  {}", ret, line);
        }
        ret
    }
}

impl DisplayKConfig for KCommentBlock<'_> {
    fn display_kconfig(&self) -> String {
        let mut ret = format!("comment {}\n", self.prompt);
        if let Some(depends) = &self.depends {
            for dep in depends {
                ret = format!("{}\n\t{}", ret, dep);
            }
        }
        ret
    }
}

impl DisplayKConfig for KOption<'_> {
    fn display_kconfig(&self) -> String {
        let mut ret = format!("config {}\n", self.name);

        ret = format!("{}\t{}", ret, self.option_type);
        if let Some(prompt) = &self.prompt {
            ret = format!("{} {}", ret, prompt);
        }
        ret = format!("{}\n", ret);

        if let Some(defaults) = &self.defaults {
            for def in defaults {
                ret = format!("{}\tdefaults {}\n", ret, def);
            }
        }
        if let Some(def_bool) = &self.def_bool {
            for def in def_bool {
                ret = format!("{}\tdef_bool {}\n", ret, def);
            }
        }
        if let Some(def_tristate) = &self.def_tristate {
            for def in def_tristate {
                ret = format!("{}\tdef_tristate {}\n", ret, def);
            }
        }
        if let Some(depends) = &self.depends {
            for dep in depends {
                ret = format!("{}\tdepends on {}\n", ret, dep);
            }
        }
        if let Some(selects) = &self.selects {
            for sel in selects {
                ret = format!("{}\tselect {}\n", ret, sel);
            }
        }
        if let Some(implies) = &self.implies {
            for imply in implies {
                ret = format!("{}\timply {}\n", ret, imply);
            }
        }
        if let Some(ranges) = &self.ranges {
            for range in ranges {
                ret = format!("{}\trange {}\n", ret, range);
            }
        }

        if let Some(help) = &self.help {
            ret = format!("{}\thelp\n", ret);
            for line in &help.text {
                if !line.is_empty() {
                    ret = format!("{}\t  {}", ret, line);
                }
                ret = format!("{}", ret);
            }
        }
        ret
    }
}

impl DisplayKConfig for KMenu<'_> {
    fn display_kconfig(&self) -> String {
        let mut ret = String::new();
        macro_rules! writemacro {
            ($field:ident) => {
                if let Some($field) = &self.$field {
                    for i in $field {
                        ret = format!("{}{}\n", ret, i);
                    }
                }
            };
        }

        ret = format!("{}menu {}\n", ret, self.description);
        if let Some(conditions) = &self.visible {
            for condition in conditions {
                ret = format!("{}\tvisible if {}\n", ret, condition);
            }
        }
        if let Some(depends) = &self.depends {
            for dep in depends {
                ret = format!("{}\tdepends on {}\n", ret, dep);
            }
        }

        if let Some(configs) = &self.configs {
            for config in configs {
                ret = format!("{}source {}\n", ret, config);
            }
        }
        writemacro!(blocks);
        writemacro!(choices);
        writemacro!(menus);
        writemacro!(options);
        ret
    }
}

impl DisplayKConfig for KChoice<'_> {
    fn display_kconfig(&self) -> String {
        let mut ret = String::new();
        ret = format!("{}choice\n", ret);

        if self.optional {
            ret = format!("{}\toptional\n", ret);
        }

        ret = format!("{}\t{}", ret, self.option_type);
        if let Some(prompt) = &self.prompt {
            ret = format!("{} {}", ret, prompt);
        }
        ret = format!("{}\n", ret);

        if let Some(defaults) = &self.defaults {
            for def in defaults {
                ret = format!("{}\tdefaults {}\n", ret, def);
            }
        }

        if let Some(depends) = &self.depends {
            for dep in depends {
                ret = format!("{}\tdepends on {}\n", ret, dep);
            }
        }

        if let Some(help) = &self.help {
            ret = format!("{}\thelp\n", ret);
            for line in &help.text {
                if !line.is_empty() {
                    format!("{}\t  {}", ret, line);
                }
                ret = format!("{}", ret);
            }
        }

        for opt in &self.options {
            ret = format!("{}{}\n", ret, opt);
        }

        ret = format!("{}endchoice\n", ret);
        ret
    }
}

impl DisplayKConfig for KConfig<'_> {
    fn display_kconfig(&self) -> String {
        let mut ret = String::new();
        macro_rules! writemacro {
            ($field:ident) => {
                if let Some($field) = &self.$field {
                    for i in $field {
                        ret = format!("{}{}\n", ret, i);
                    }
                }
            };
        }

        if let Some(mainmenu) = &self.mainmenu {
            ret = format!("{}mainmenu {}\n", ret, mainmenu);
        }

        if let Some(configs) = &self.configs {
            for config in configs {
                ret = format!("{}source {}\n", ret, config);
            }
        }


        writemacro!(blocks);
        writemacro!(choices);
        writemacro!(menus);
        writemacro!(options);
        ret
    }
}

impl DisplayKConfig for Block<'_> {
    fn display_kconfig(&self) -> String {
        format!("if {}\n{}\nendif\n", self.condition, self.config)
    }
}
