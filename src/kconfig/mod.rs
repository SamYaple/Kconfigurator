mod kchoice;
mod kcommentblock;
mod kconfig;
mod kmenu;
mod koption;
mod expr;
mod util;

use kchoice::KChoice;
use kcommentblock::KCommentBlock;
use kconfig::KConfig;
use kmenu::KMenu;
use koption::KOption;

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
