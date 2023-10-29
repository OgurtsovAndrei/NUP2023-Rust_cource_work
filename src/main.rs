use std::{env, fs};
use std::fs::DirEntry;

fn main() {
    let (path, find_substr) = get_directory_from_cli_args();
    let path_metadata = fs::metadata(&path);

    match path_metadata {
        Ok(metadata) => {
            if !metadata.is_dir() {
                eprintln!("It is not a directory: {}", path);
                return;
            }
            let (res, body) = process_dir(&path, "", &*find_substr);
            if res { println!("{}", body) }
        }
        Err(_) => {
            eprintln!("No such directory: {}", path);
        }
    }
}

fn process_dir(path: &str, prefix_print: &str, find_substr: &str) -> (bool, String) {
    let mut is_not_empty = false;
    let mut body: String = String::from("");
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            let (res, res_body) = process_entry(entry, prefix_print, find_substr);
            is_not_empty |= res;
            if res { body.push_str(&*res_body) }
        }
    } else {
        body.push_str(&*(String::from(prefix_print) + "Problem occurred during reading the directory\n"))
    }
    return (is_not_empty, body);
}

fn get_directory_from_cli_args() -> (String, String) {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <directory_path>", args[0]);
        std::process::exit(1);
    }
    let path = args[1].clone();

    let mut find_substr = String::from("");
    if args.len() >= 4 {
        if args[2] == "--find" { find_substr = args[3].to_string() }
    }

    (path, find_substr)
}

fn process_entry(entry: Result<DirEntry, std::io::Error>, prefix_print: &str, find_substr: &str) -> (bool, String) {
    let mut is_not_empty = false;
    let mut body: String = String::from("");
    let entry = entry.expect("Failed to read directory entry\n");
    let entry_path = entry.path();
    if entry_path.is_file() {
        if let Some(file_name) = entry.file_name().to_str() {
            if file_name.contains(find_substr) {
                body.push_str(&*format!("File\t: {prefix_print}{file_name}\n"));
                is_not_empty = true
            }
        }
    } else if entry_path.is_dir() {
        if let Some(dir_name) = entry.file_name().to_str() {
            let new_prefix: String = prefix_print.to_string() + "  | ";
            let mut new_subst: String = find_substr.to_string();
            if dir_name.contains(find_substr) { new_subst = String::from("") }
            let (res, res_body) = process_dir(entry_path.to_str().unwrap(), &*new_prefix, &*new_subst);
            is_not_empty |= res;
            if res {
                body.push_str(&*format!("Dir\t: {prefix_print}{dir_name}\n"));
                body.push_str(&*res_body)
            }
        }
    }
    return (is_not_empty, body);
}
