use std::fs::DirEntry;
use std::rc::Rc;
use std::fs;
use crate::empty_string;

pub struct LookupConfig {
    prefix_print: String,
    target_substring: String,
    write_full_path: bool,
}

impl LookupConfig {
    pub fn new(prefix_print: String, target_substring: String, write_full_path: bool) -> LookupConfig {
        LookupConfig {
            prefix_print,
            target_substring,
            write_full_path,
        }
    }
}

pub struct LookupResult {
    pub is_successful: bool,
    pub body: Vec<LookupResultEntry>,
}

pub enum LookupResultEntry {
    Directory { directory: Rc<DirectoryEntry> },
    File { file: Rc<FileEntry> },
    TextOrRustFile { file: Rc<FileEntry> },
}

impl LookupResultEntry {
    pub fn get_full_path(&self) -> String {
        match self {
            LookupResultEntry::Directory { directory } => {
                format!("Directory : {}", directory.path)
            }
            LookupResultEntry::File { file } => {
                format!("SomeFile  : {}/{}", file.parent.path, file.entry.file_name().to_str().unwrap())
            }
            LookupResultEntry::TextOrRustFile { file } => {
                format!("RustFile  : {}/{}", file.parent.path, file.entry.file_name().to_str().unwrap())
            }
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            LookupResultEntry::Directory { directory } => {
                directory.entry.clone().unwrap().file_name().to_str().unwrap().to_string()
            }
            LookupResultEntry::File { file } => {
                file.entry.file_name().to_str().unwrap().to_string()
            }
            LookupResultEntry::TextOrRustFile { file } => {
                file.entry.file_name().to_str().unwrap().to_string()
            }
        }
    }
}

pub struct FileEntry {
    parent: Rc<DirectoryEntry>,
    entry: Rc<DirEntry>,
}

impl FileEntry {
    fn new(parent: Rc<DirectoryEntry>, entry: DirEntry) -> Self { Self { parent: parent.clone(), entry: Rc::new(entry) } }

    pub fn get_path(&self) -> String {
        self.entry.path().to_str().unwrap().to_string()
    }
}

pub struct DirectoryEntry {
    entry: Option<Rc<DirEntry>>,
    path: String,
}

impl DirectoryEntry {
    fn new(entry: DirEntry, path: String) -> Self { DirectoryEntry { entry: Option::Some(Rc::new(entry)), path } }

    pub fn create_empty() -> Self { DirectoryEntry { entry: None, path: empty_string() } }
}

trait FileSystemEntry {
    fn process_entry(self, lookup_config: &LookupConfig) -> LookupResult;
}

impl FileSystemEntry for FileEntry {
    fn process_entry(self, lookup_config: &LookupConfig) -> LookupResult {
        if let Some(file_name) = self.entry.file_name().to_str() {
            if file_name.contains(&lookup_config.target_substring) {
                let file_type = self.entry.file_name().to_str().unwrap().to_string();
                let lookup_result_entry;
                if file_type.ends_with(".rs") || file_type.ends_with(".txt") {
                    lookup_result_entry = LookupResultEntry::TextOrRustFile { file: Rc::new(self) };
                } else {
                    lookup_result_entry = LookupResultEntry::File { file: Rc::new(self) };
                }
                return LookupResult { is_successful: true, body: vec![lookup_result_entry] };
            }
        }
        return LookupResult { is_successful: false, body: vec![] };
    }
}

impl FileSystemEntry for DirectoryEntry {
    fn process_entry(self, lookup_config: &LookupConfig) -> LookupResult {
        if let Some(dir_name) = self.entry.clone().unwrap().file_name().to_str() {
            let new_prefix: String;
            if lookup_config.write_full_path { new_prefix = format!("{}{dir_name}/", lookup_config.prefix_print); } else { new_prefix = format!("{}  | ", lookup_config.prefix_print); }
            let mut new_subst: String = lookup_config.target_substring.to_string();
            if dir_name.contains(&lookup_config.target_substring) { new_subst = crate::empty_string() }
            let self_rc = Rc::new(self);
            let mut result = process_dir(self_rc.entry.clone().unwrap().path().to_str().unwrap(),
                                         LookupConfig::new(new_prefix, new_subst, lookup_config.write_full_path), self_rc.clone());
            if result.is_successful {
                // let dir_string = LookupResultEntry { some_prefix: String::from("Dir     : "), path: format!("{}{dir_name}", lookup_config.prefix_print), filename: empty_string(), is_file: false };
                let dir_entry = LookupResultEntry::Directory { directory: self_rc };
                result.body.insert(0, dir_entry);
                return LookupResult { is_successful: result.is_successful, body: result.body };
            }
        }
        return LookupResult { is_successful: false, body: vec![] };
    }
}

pub fn process_dir(path: &str, lookup_config: LookupConfig, dir: Rc<DirectoryEntry>) -> LookupResult {
    let mut is_not_empty = false;
    let mut body: Vec<LookupResultEntry> = vec![];
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            let result = process_entry(entry, &lookup_config, dir.clone());
            is_not_empty |= result.is_successful;
            if result.is_successful {
                body.extend(result.body)
            }
        }
    }
    return LookupResult { is_successful: is_not_empty, body };
}

fn process_entry(entry: Result<DirEntry, std::io::Error>, lookup_config: &LookupConfig, parent: Rc<DirectoryEntry>) -> LookupResult {
    let entry = entry.expect("Failed to read directory entry\n");
    let name = entry.file_name().to_str().clone().unwrap().to_string();
    let entry_path = entry.path();
    if entry_path.is_file() {
        let current_file = FileEntry::new(parent, entry);
        return current_file.process_entry(lookup_config);
    } else if entry_path.is_dir() {
        let current_dir = DirectoryEntry::new(entry, format!("{}/{}", &parent.path, name));
        return current_dir.process_entry(lookup_config);
    }
    return LookupResult { is_successful: false, body: vec![] };
}
