use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::ffi::OsString;

use std::fs::File;

#[derive(Debug)]
struct L2C {
    regex: String
}

fn get_kv_pair(base: &str) -> Option<(&str, &str)> {
    if let Some(mid) = base.find(':') {
        let (left, right) = base.split_at(mid);

        return Some((left.trim_end(), right.trim_start_matches(':').trim()));
    }

    None
}

fn l2c_parse(l2c_reader: BufReader<File>) -> L2C {
    let mut l2c = L2C {
        regex: String::new()
    };

    let lines = l2c_reader.lines();

    for (i, line) in lines.map_while(Result::ok).enumerate() {
        let line = line.trim_start();
        if line.starts_with('#') { // Skip comments
            println!("Found comment in line {}", i);

            continue;
        }

        if line.is_empty() { // Skip blank lines
            continue;
        }

        if let Some((key, value)) = get_kv_pair(line) {
            match key.to_lowercase().as_str() {
                "regex" => l2c.regex = value.to_string(),
                _ => {
                    panic!("The key {key} is currently not known!"); // TODO: Error!
                }
            };
        } else {
            panic!("No key value pair found!"); // TODO: Error!
        }
    }

    return l2c;
}

pub fn main() {
    let flags = xflags::parse_or_exit! {
        /// The path of the log file
        required log_path: PathBuf

        /// The path of the l2c file
        required l2c_path: PathBuf

        // - Flags -

        /// The output file path (defaults to 'output.csv')
        optional -o, --output output_path: PathBuf

        /// The separator sequence (defaults to ',')
        optional --separator separator: OsString
    };

    let l2c = {
        let l2c_file = match File::open(flags.l2c_path) {
            Ok(file) => file,
            Err(err) => panic!("Could not open file: {}", err),
        };

        l2c_parse(BufReader::new(l2c_file))
    };

    println!("{:#?}", l2c);
    // TODO: Rest of the program
}
