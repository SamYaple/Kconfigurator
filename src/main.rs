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

    for path in matches {
        //eprintln!("LOADED FILE: {}", path.display());
        let content = kconfig::load_from_file(path.display().to_string());
        let config = kconfig::take_kconfig(&content);

        // Rip into the top level config, this is not recursize but the `config` object does
        // contain all of the parsed information without any trailing, unprocessed data.
        // TODO: The individual string options have not been trimmed for whitespace
        // TODO: Kconfig.include includes lots of macros to use for string replacement
        //       these macros need to be used to properly generate some of the descriptions.
        // TODO: Some variables are implicitly expected either set from the Makefile or the env
        if let Some(options) = config.options {
            for opt in options {
                println!("{}", opt);
            }
        }
        // TODO: Stage2 where we parse and evaluate the boolean statements against a .config
    }
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
                    //if let Some(parent) = path.parent() {
                    //    result.push(parent.to_path_buf());
                    //}
                }
            }
        }
    }

    result
}
