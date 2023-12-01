use std::fs::File;
use std::io::Write;

pub trait Writer {
    fn write(&self, text: &String);
}

pub struct ConsoleWriter;

pub struct FileWriter {
    pub file_name: String,
}

impl Writer for ConsoleWriter {
    fn write(&self, text: &String) {
        println!("{}", text)
    }
}

impl Writer for FileWriter {
    fn write(&self, text: &String) {
        let mut file = File::create(&self.file_name).expect("Unable to create file");
        match file.write_all(text.as_bytes()) {
            Ok(_) => println!("Result was written to file {}", &self.file_name),
            Err(e) => eprintln!("Error accursed during writing to file: {}", e)
        }
    }
}
