use std::{env, fs};
use std::sync::Arc;

use lookup_engine::{DirectoryEntry, LookupConfig};
use writers::{ConsoleWriter, FileWriter, Writer};

mod postprocess;
mod writers;
mod lookup_engine;
mod thread_pool;

fn main() {
    let settings = get_directory_from_cli_args();
    let path_metadata = fs::metadata(&settings.start_path);

    match path_metadata {
        Ok(metadata) => {
            if !metadata.is_dir() {
                eprintln!("It is not a directory: {}", settings.start_path);
                return;
            }
            let result = lookup_engine::process_dir(&settings.start_path,
                                                    LookupConfig::new(empty_string(), settings.target_substring.clone(), settings.sort_files),
                                                    Arc::new(DirectoryEntry::create_empty()));
            let parsed_and_sorted_result = postprocess::parse_result_vector(result.body, &settings);
            if result.is_successful {
                settings.writer.write(&parsed_and_sorted_result)
            } else { println!("Nothing to show"); }
        }
        Err(_) => {
            eprintln!("No such directory: {}", settings.start_path);
        }
    }
}

struct Settings {
    start_path: String,
    target_substring: String,
    sort_files: bool,
    writer: Box<dyn Writer>,
    look_for_key_entry_in_files: bool,
    key_in_file: String,
}

fn get_directory_from_cli_args() -> Settings {
    let args: Vec<String> = env::args().collect();

    let mut settings: Settings = Settings {
        start_path: empty_string(),
        target_substring: empty_string(),
        sort_files: false,
        writer: Box::new(ConsoleWriter),
        look_for_key_entry_in_files: false,
        key_in_file: empty_string(),
    };

    if args.len() < 2 {
        settings.start_path = String::from("./");
    } else {
        settings.start_path = args[1].clone();
    }

    for arg_index in 2..args.len() {
        if args[arg_index] == "--find" { settings.target_substring = args[arg_index + 1].to_string() }
        if args[arg_index] == "--to_file" {
            let file_name = args[arg_index + 1].to_string();
            settings.writer = Box::new(FileWriter { file_name })
        }
        if args[arg_index] == "--in_file" {
            settings.look_for_key_entry_in_files = true;
            settings.key_in_file = args[arg_index + 1].to_string();
        }
        if args[arg_index] == "--sort" { settings.sort_files = true }
    }

    settings
}

fn empty_string() -> String {
    String::from("")
}
