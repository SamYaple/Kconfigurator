mod kconfig;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} /path/to/linux_kernel_source", args[0]);
        return;
    }

    let dir = &args[1];
    let matches = search(&Path::new(dir));

    let mut opts: usize = 0;
    for path in matches {
        let content = load_from_file(path.display().to_string());
        let config = take_kconfig(&content);
        println!("{}", config);

        opts = opts + config.collect_options().len();
    }
    eprintln!("Total options found across all KConfigs in '{}': {}", dir, opts);
}

fn search(path: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    result.extend(search(&path));
                } else if path.file_name().map_or(false, |f| f.to_string_lossy().starts_with("Kconfig")) { 
					result.push(path.to_path_buf());
                }
            }
        }
    }

    result
}

fn load_from_file(path_string: String) -> String {
    let pathed = std::path::Path::new(&path_string).to_path_buf();
    match std::fs::read_to_string(pathed) {
        Ok(content) => return content,
        Err(e) => {
            panic!("Failed to open '{}' with error '{}'", path_string, e);
        }
    }
}


fn take_kconfig(input: &str) -> kconfig::KConfig {
    match kconfig::KConfig::parse(input) {
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
