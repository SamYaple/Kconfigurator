mod kchoice;
mod kcommentblock;
mod kconfig;
mod kmenu;
mod koption;
mod expr;
mod util;

pub use kchoice::KChoice;
pub use kcommentblock::KCommentBlock;
pub use kconfig::KConfig;
pub use kmenu::KMenu;
pub use koption::KOption;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum OptionType {
    Tristate,
    Bool,
    Hex,
    Int,
    Str,
}

impl std::fmt::Display for OptionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionType::Tristate => write!(f, "\"tristate\""),
            OptionType::Bool     => write!(f, "\"bool\""),
            OptionType::Hex      => write!(f, "\"hex\""),
            OptionType::Int      => write!(f, "\"int\""),
            OptionType::Str      => write!(f, "\"str\""),
        }
    }
}
