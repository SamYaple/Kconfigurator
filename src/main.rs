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

    // janky yaml output for loading into python...
    println!("kconfigs:");
    for path in matches {
        print!("  \"{}\":", path.display());
        let content = kconfig::load_from_file(path.display().to_string());
        let config = kconfig::take_kconfig(&content);

        let opts = config.collect_options();
        if opts.is_empty() {
            println!(" []");
        } else {
            println!();
            for opt in opts {
                println!("{}", opt);
            }
        }
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
