use std::{env, fs};
use std::fs::DirEntry;

fn main() {
    let path = get_directory_from_cli_args();
    let path_metadata = fs::metadata(path);

    match path_metadata {
        Ok(metadata) => {
            if metadata.is_dir() {
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries {
                        print_entry(entry);
                    }
                } else {
                    eprintln!("Problem occurred during reading the directory");
                }
            } else {
                eprintln!("It is not a directory: {}", path);
            }
        }
        Err(_) => {
            eprintln!("No such directory: {}", path);
        }
    }
}

fn get_directory_from_cli_args() -> &String {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <directory_path>", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];
    path
}

fn print_entry(entry: Result<DirEntry, std::io::Error>) {
    let entry = entry.expect("Failed to read directory entry");
    let entry_path = entry.path();
    if entry_path.is_file() {
        if let Some(file_name) = entry.file_name().to_str() {
            println!("{:<10}: {}", "File", file_name);
        }
    } else if entry_path.is_dir() {
        if let Some(dir_name) = entry.file_name().to_str() {
            println!("{:<10}: {}", "Directory", dir_name);
        }
    }
}
