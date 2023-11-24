use std::{env, fs};
use std::fs::{DirEntry, File};
use std::io::Write;


#[derive(Clone)]
struct LookupConfig {
    prefix_print: String,
    target_substring: String,
    write_full_path: bool,
}

impl LookupConfig {
    fn new(prefix_print: String, target_substring: String, write_full_path: bool) -> LookupConfig {
        LookupConfig {
            prefix_print,
            target_substring,
            write_full_path
        }
    }
}

struct LookupResult {
    is_successful: bool,
    body: Vec<String>,
}

struct FileEntry {
    entry: DirEntry,
}

struct DirectoryEntry {
    entry: DirEntry,
}

impl FileEntry {
    fn new(entry: DirEntry) -> Self { Self { entry } }
}

impl FileSystemEntry for FileEntry {
    fn process_entry(&self, lookup_config: &LookupConfig) -> LookupResult {
        if let Some(file_name) = self.entry.file_name().to_str() {
            if file_name.contains(&lookup_config.target_substring) {
                return LookupResult { is_successful: true, body: vec![format!("File    : {}{file_name}", lookup_config.prefix_print)] };
            }
        }
        return LookupResult { is_successful: false, body: vec![] };
    }
}

impl DirectoryEntry {
    fn new(entry: DirEntry) -> Self { DirectoryEntry { entry } }
}

impl FileSystemEntry for DirectoryEntry {
    fn process_entry(&self, lookup_config: &LookupConfig) -> LookupResult {
        if let Some(dir_name) = self.entry.file_name().to_str() {
            let new_prefix: String;
            if lookup_config.write_full_path { new_prefix = format!("{}{dir_name}/", lookup_config.prefix_print); }
            else { new_prefix = format!("{}  | ", lookup_config.prefix_print); }
            let mut new_subst: String = lookup_config.target_substring.to_string();
            if dir_name.contains(&lookup_config.target_substring) { new_subst = empty_string() }
            let mut result = process_dir(self.entry.path().to_str().unwrap(),
                                         LookupConfig::new(new_prefix, new_subst, lookup_config.write_full_path));
            if result.is_successful {
                let dir_string = format!("Dir     : {}{dir_name}", lookup_config.prefix_print);
                result.body.insert(0, dir_string);
                return LookupResult { is_successful: result.is_successful, body: result.body};
            }
        }
        return LookupResult { is_successful: false, body: vec![] };
    }
}

trait FileSystemEntry {
    fn process_entry(&self, lookup_config: &LookupConfig) -> LookupResult;
}


fn main() {
    let settings = get_directory_from_cli_args();
    let path_metadata = fs::metadata(&settings.start_path);

    match path_metadata {
        Ok(metadata) => {
            if !metadata.is_dir() {
                eprintln!("It is not a directory: {}", settings.start_path);
                return;
            }
            let result = process_dir(&settings.start_path, LookupConfig::new(empty_string(), settings.target_substring, settings.sort_files));
            let parsed_and_sorted_result = parse_result_vector(result.body);
            if result.is_successful {
                if settings.write_to_file {
                    let mut file = File::create(&settings.out_file_name).expect("Unable to create file");
                    match file.write_all(parsed_and_sorted_result.as_bytes()) {
                        Ok(_) => println!("Result was written to file {}", &settings.out_file_name),
                        Err(e) => eprintln!("Error accursed during writing to file: {}", e),
                    }
                } else { println!("{}", parsed_and_sorted_result) }
            } else { println!("Nothing to show"); }
        }
        Err(_) => {
            eprintln!("No such directory: {}", settings.start_path);
        }
    }
}

fn parse_result_vector(mut result: Vec<String>) -> String {
    let mut parsed: Vec<Vec<String>> = vec![];
    for entry in &result {
        let parsed_entry: Vec<String> = entry.split('/').map(|s| s.to_owned()).collect();
        parsed.push(parsed_entry);
    }

    fn sort_by_last_predicate<T: PartialOrd>(a: &Vec<T>, b: &Vec<T>) -> bool {
        return a.last() < b.last();
    }

    parsed = insertion_sort(parsed, sort_by_last_predicate);
    for (index, entry) in parsed.iter().enumerate() {
        result[index] = entry.join("/")
    }
    return result.join("\n");
}

fn insertion_sort<T: PartialOrd, Fun>(mut vec: Vec<T>, predicate: Fun) -> Vec<T>
    where Fun: Fn(&T, &T) -> bool {
    let len = vec.len();

    for i in 1..len {
        let mut j = i;

        while j > 0 && predicate(&vec[j], &vec[j - 1]) {
            vec.swap(j, j - 1);
            j -= 1;
        }
    }
    vec
}

fn process_dir(path: &str, lookup_config: LookupConfig) -> LookupResult {
    let mut is_not_empty = false;
    let mut body: Vec<String> = vec![];
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            let result = process_entry(entry, &lookup_config);
            is_not_empty |= result.is_successful;
            if result.is_successful {
                body.extend(result.body)
            }
        }
    } else {
        body.push(format!("{}Problem occurred during reading the directory\n", lookup_config.prefix_print))
    }
    return LookupResult { is_successful: is_not_empty, body };
}

struct Settings {
    start_path: String,
    target_substring: String,
    sort_files: bool,
    write_to_file: bool,
    out_file_name: String,
}

fn get_directory_from_cli_args() -> Settings {
    let args: Vec<String> = env::args().collect();

    let mut settings: Settings = Settings {
        start_path: empty_string(),
        target_substring: empty_string(),
        sort_files: false,
        write_to_file: false,
        out_file_name: empty_string(),
    };

    if args.len() < 2 {
        settings.start_path = String::from("./");
    } else {
        settings.start_path = args[1].clone();
    }

    for arg_index in 2..args.len() {
        if args[arg_index] == "--find" { settings.target_substring = args[arg_index + 1].to_string() }
        if args[arg_index] == "--to_file" { settings.out_file_name = args[arg_index + 1].to_string(); settings.write_to_file = true }
        if args[arg_index] == "--sort" { settings.sort_files = true }

    }

    settings
}

fn process_entry(entry: Result<DirEntry, std::io::Error>, lookup_config: &LookupConfig) -> LookupResult {
    let entry = entry.expect("Failed to read directory entry\n");
    let entry_path = entry.path();
    if entry_path.is_file() {
        let current_file = FileEntry::new(entry);
        return current_file.process_entry(lookup_config);
    } else if entry_path.is_dir() {
        let current_dir = DirectoryEntry::new(entry);
        return current_dir.process_entry(lookup_config);
    }
    return LookupResult { is_successful: false, body: vec![] };
}

fn empty_string() -> String {
    String::from("")
}
