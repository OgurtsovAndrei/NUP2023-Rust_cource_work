use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;

use crate::Settings;
use crate::lookup_engine::LookupResultEntry;

pub fn parse_result_vector(mut result: Vec<LookupResultEntry>, settings: &Settings) -> String {
    fn sort_by_last_predicate(a: &LookupResultEntry, b: &LookupResultEntry) -> bool {
        return a.get_name() < b.get_name();
    }

    if settings.sort_files {
        let mut files = vec![];
        for entry in result.into_iter() {
            match &entry {
                LookupResultEntry::File { .. } => { files.push(entry) }
                LookupResultEntry::TextOrRustFile { .. } => { files.push(entry) }
                LookupResultEntry::Directory { .. } => {}
            }
        }
        result = insertion_sort(files, sort_by_last_predicate);
    }
    return result.iter().map(|entry| process_one_file(entry, settings)).collect::<Vec<String>>().join("");
}

fn process_one_file(file: &LookupResultEntry, settings: &Settings) -> String {
    let mut result = file.get_full_path() + "\n";
    if !settings.look_for_key_entry_in_files { return result; }
    return match file {
        LookupResultEntry::Directory { .. } => { crate::empty_string() }
        LookupResultEntry::File { .. } => { crate::empty_string() }
        LookupResultEntry::TextOrRustFile { file } => {
            let prefix = " ".repeat(8) + "--> ";
            let key_entries: Vec<String>;
            let key_entries_res = look_for_key_in_file(&file.get_path(), &settings.key_in_file);
            match key_entries_res {
                Ok(data) => { key_entries = data }
                Err(_) => { return crate::empty_string(); }
            }
            for key_entry in key_entries.iter() {
                result.push_str(&format!("{}{}\n", prefix, key_entry))
            }
            result
        }
    }
}

fn look_for_key_in_file(filename: &str, key: &str) -> io::Result<Vec<String>> {
    let path = Path::new(filename);
    let file = File::open(&path)?;
    let reader = io::BufReader::new(file);

    let mut results = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.contains(key) {
            results.push(line);
        }
    }
    Ok(results)
}

fn insertion_sort<T, Fun>(mut vec: Vec<T>, predicate: Fun) -> Vec<T>
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
