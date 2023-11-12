use std::{env, fs};
use std::fs::DirEntry;

#[derive(Clone)]
struct LookupConfig {
    prefix_print: String,
    target_substring: String
}

impl LookupConfig {
    fn new(prefix_print: String, target_substring: String) -> LookupConfig{
        LookupConfig {
            prefix_print,
            target_substring
        }
    }
}


struct File {
    entry: DirEntry
}

struct Directory {
    entry: DirEntry
}

impl FileSystemEntry for File {
    fn new(entry: DirEntry) -> Self { Self { entry } }

    fn process_entry(&self, lookup_config: LookupConfig) -> (bool, String) {
        if let Some(file_name) = self.entry.file_name().to_str() {
            if file_name.contains(&lookup_config.target_substring) {
                return (true, format!("File\t: {}{file_name}\n", lookup_config.prefix_print))
            }
        }
        return (false, empty_string())
    }
}


impl FileSystemEntry for Directory {
    fn new(entry: DirEntry) -> Self { Directory { entry } }

    fn process_entry(&self, lookup_config: LookupConfig) -> (bool, String) {
        if let Some(dir_name) = self.entry.file_name().to_str() {
            let new_prefix: String = format!("{}  | ", lookup_config.prefix_print);
            let mut new_subst: String = lookup_config.target_substring.to_string();
            if dir_name.contains(&lookup_config.target_substring) { new_subst = empty_string() }
            let (res, res_body) = process_dir(self.entry.path().to_str().unwrap(),
                                              LookupConfig::new(new_prefix, new_subst));
            if res {
                return (res, format!("Dir\t: {}{dir_name}\n{res_body}", lookup_config.prefix_print));
            }
        }
        return (false, empty_string());
    }
}

trait FileSystemEntry {
    fn new(entry: DirEntry) -> Self;
    fn process_entry(&self, lookup_config: LookupConfig) -> (bool, String);
}



fn main() {
    let (path, find_substr) = get_directory_from_cli_args();
    let path_metadata = fs::metadata(&path);

    match path_metadata {
        Ok(metadata) => {
            if !metadata.is_dir() {
                eprintln!("It is not a directory: {}", path);
                return;
            }
            let (res, body) = process_dir(&path, LookupConfig::new(empty_string(), find_substr));
            if res { println!("{}", body) }
        }
        Err(_) => {
            eprintln!("No such directory: {}", path);
        }
    }
}

fn process_dir(path: &str, lookup_config: LookupConfig) -> (bool, String) {
    let mut is_not_empty = false;
    let mut body: String = empty_string();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            let (res, res_body) = process_entry(entry, lookup_config.clone());
            is_not_empty |= res;
            if res { body.push_str(&res_body) }
        }
    } else {
        body.push_str(&(format!("{}Problem occurred during reading the directory\n", lookup_config.prefix_print)))
    }
    return (is_not_empty, body);
}

fn get_directory_from_cli_args() -> (String, String) {
    let args: Vec<String> = env::args().collect();
    let path;

    if args.len() < 2 {
        path = String::from("./");
    } else {
        path = args[1].clone();
    }

    let mut find_substr = empty_string();
    if args.len() >= 4 {
        if args[2] == "--find" { find_substr = args[3].to_string() }
    }

    (path, find_substr)
}

fn process_entry(entry: Result<DirEntry, std::io::Error>, lookup_config: LookupConfig) -> (bool, String) {
    let entry = entry.expect("Failed to read directory entry\n");
    let entry_path = entry.path();
    if entry_path.is_file() {
        let current_file = File::new(entry);
        return current_file.process_entry(lookup_config);
    } else if entry_path.is_dir() {
        let current_dir = Directory::new(entry);
        return current_dir.process_entry(lookup_config);
    }
    return (false, empty_string());
}

fn empty_string() -> String {
    String::from("")
}
