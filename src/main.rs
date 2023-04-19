use clap::Parser;


#[derive(Parser)]
struct Cli{
    path: String
}

mod charcounter{
    use std::collections::HashMap;
    pub struct Counter {
        files_text: u32,
        files_nontext: u32,
        directories: u32,
        chars_total: u64,
        charmap: HashMap<char, u64>
    }
    
    enum IncType {
        Text,
        NonText,
        Directory,
        Char(char)
    }
    
    impl Counter {    
        pub fn new() -> Counter {
            Counter {
                files_text: 0,
                files_nontext: 0,
                directories: 0,
                chars_total: 0,
                charmap: HashMap::new()
            }
        }
    
        fn increment(&mut self, file_type: IncType) {
            match file_type {
                IncType::Text => self.files_text += 1,
                IncType::NonText => self.files_nontext += 1,
                IncType::Directory => self.directories += 1,
                IncType::Char(c) => {
                    let count = self.charmap.entry(c).or_insert(0);
                    *count += 1;
                    self.chars_total += 1;
                }
            }
        }
        pub fn chars_total(&self) -> u64 {
            self.chars_total
        }

        pub fn charmap(&self) -> &HashMap<char, u64> {
            &self.charmap
        }

        pub fn files_text(&self) -> u32 {
            self.files_text
        }

        pub fn files_nontext(&self) -> u32 {
            self.files_nontext
        }
        
        pub fn directories(&self) -> u32 {
            self.directories
        }

    }

    // increment the counter for each character
    fn char_count(s: &str, counter: &mut Counter) {
        for c in s.chars() {
            counter.increment(IncType::Char(c));
        }
    }

    // read the file and return the contents as a string
    fn read_file(path: &std::path::Path, counter: &mut Counter) -> String {
        let contents: Result<String, std::io::Error>= std::fs::read_to_string(path);
        match contents {
            Ok(contents) => {
                counter.increment(IncType::Text);
                return contents;
            },
            Err(_) => {
                counter.increment(IncType::NonText);
                return String::new();
            }
        }
        
    }

    // reads the file and pass the contents to char_count
    pub fn do_counting(path: &std::path::Path, counter: &mut Counter){
        let contents: String = read_file(path, counter);
        char_count(&contents, counter);
    }

    // recursively walk through a directory and for each file, do_counting
    pub fn walk_dir(path: &std::path::Path, counter: &mut Counter) {
        if path.is_dir() {
            counter.increment(IncType::Directory);
            for entry in std::fs::read_dir(path)
                .expect("read_dir call failed") {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        walk_dir(&path, counter);
                    } else if path.is_file() {
                        do_counting(&path, counter);
                    }else{
                        continue;
                    }
                }
            }
        }
    }
}

mod printer{
    use super::charcounter::*;
    use cli_table::{format::Justify, print_stdout, Table, WithTitle};

    #[derive(Table)]
    struct CharCounterTable {
        #[table(title = "Character", justify = "Justify::Center")]
        character: char,
        #[table(title = "Unicode", justify = "Justify::Left")]
        code: String,
        #[table(title = "Counts", justify = "Justify::Right")]
        counts: u64
        
    }

    pub fn print_table(counter: &Counter) {
        let mut counts_vec: Vec<CharCounterTable> = vec![];
        let counts = counter.charmap();
        for (key, value) in counts {
            counts_vec.push(CharCounterTable {
                character: *key,
                counts: *value,
                code: format!("U+{:x};", *key as u32),
            });
        }
        // sort the vector by counts
        counts_vec.sort_by(|a, b| b.counts.cmp(&a.counts));
    
        match print_stdout(counts_vec.with_title()) {
            Ok(_) => (),
            Err(_) => println!("Error printing table"),
            
        }
    }
}



fn main() {
    use charcounter::*;
    use printer::*;

    //start the timer
    let start = std::time::Instant::now();

    //parse the command line arguments
    let args = Cli::parse();

    // get the path
    let path = std::path::Path::new(&args.path);

    //define the counter
    let mut counter = Counter::new();

    if path.is_dir() {
        walk_dir(path, &mut counter);
    } else if path.is_file() {
        do_counting(path, &mut counter);
    } else {
        println!("{} is not a valid path", path.display());
    }

    //stop the timer
    let duration = start.elapsed();

    // printing the hashmap
    if counter.chars_total() > 0 {
        print_table(&counter);
    } 
    println!("{} text files, {} non-text files, {} directories, {} chars, {:.6} secs",
                counter.files_text(),
                counter.files_nontext(),
                counter.directories(),
                counter.chars_total(),
                duration.as_secs_f64()
                );
    
}
