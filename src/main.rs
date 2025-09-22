use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::fs::File;

use regex::Regex;

#[derive(Debug)]
struct L2C {
    regex: Regex,
    order: Vec<String>
}

fn get_kv_pair(base: &str) -> Option<(&str, &str)> {
    if let Some(mid) = base.find(':') {
        let (left, right) = base.split_at(mid);

        return Some((left.trim_end(), right.trim_start_matches(':').trim()));
    }

    None
}

fn get_list(value: &str) -> Vec<String> {
    let mut result = Vec::new();

    for v in value.split_whitespace() {
        result.push(v.to_string());
    }

    result
}

fn l2c_parse<R: BufRead>(l2c_reader: R) -> L2C {
    let mut regex: Option<Regex> = None; 
    let mut order: Option<Vec<String>> = None;

    let lines = l2c_reader.lines();

    for (i, line) in lines.map_while(Result::ok).enumerate() {
        let line = line.trim_start();
        if line.starts_with('#') { // Skip comments
            continue;
        }

        if line.is_empty() { // Skip blank lines
            continue;
        }

        if let Some((key, value)) = get_kv_pair(line) {
            match key.to_lowercase().as_str() {
                "regex" => regex = Some(Regex::new(value).unwrap()), // TODO: Do not assume the regex works
                "order" => order = Some(get_list(value)),
                _ => {
                    panic!("The key {key} is currently not known!"); // TODO: Error!
                }
            };
        } else {
            panic!("No key value pair found!"); // TODO: Error!
        }
    }

    // TODO: Check if the values have been set
    let regex = regex.unwrap();
    let order = order.unwrap_or(Vec::new()); // TODO: Check if the values are correct

    return L2C {
        regex,
        order,
    };
}

fn write_string<W: Write>(writer: &mut W, capture: &str) {
    writer.write(b"\"").unwrap(); // Starting "

    for (i, s) in capture.split('"').enumerate() {
        if i > 0 {
            //    Based on the RFC 4180 specification
            //
            //    7.  If double-quotes are used to enclose fields, then a double-quote
            //        appearing inside a field must be escaped by preceding it with
            //        another double quote.  For example:
            //
            //        "aaa","b""bb","ccc"
            writer.write(b"\"\"").unwrap();
        }
        writer.write(s.as_bytes()).unwrap();
    }

    writer.write(b"\"").unwrap(); // Ending "
}

fn log_execute<R: BufRead, W: Write>(input: R, writer: &mut W, l2c: &L2C, sep: String) {
    for (i, element) in l2c.order.iter().enumerate() {
        if i != 0 {
            writer.write(sep.as_bytes()).unwrap();
        }
        writer.write(element.as_bytes()).unwrap();
    }

    writer.write(b"\n").unwrap();

    for line in input.lines().map_while(Result::ok) {
        let captures = l2c.regex.captures(&line);
        if captures.is_none() {
            continue;
        }
        let captures = captures.unwrap();

        for (i, element) in l2c.order.iter().enumerate() {
            if i != 0 {
                writer.write(sep.as_bytes()).unwrap();
            }

            let capture = captures.name(&element).unwrap().as_str();

            if !capture.contains(&sep) {
                writer.write(capture.as_bytes()).unwrap();
            } else {
                write_string(writer, capture);
            }
        }

        writer.write(b"\n").unwrap();
    }
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
        optional --separator separator: String
    };

    let log_file = match File::open(flags.log_path) {
        Ok(file) => file,
        Err(err) => panic!("Could not open file: {}", err),
    };

    let l2c = {
        let l2c_file = match File::open(flags.l2c_path) {
            Ok(file) => file,
            Err(err) => panic!("Could not open file: {}", err),
        };

        l2c_parse(BufReader::new(l2c_file))
    };
    
    let output_path = flags.output.unwrap_or(Path::new("output.csv").to_path_buf());
    let output = File::create(&output_path);
    if output.is_err() {
        panic!("The output file could not be created: {}", output.unwrap_err());
    }

    let mut file_writer = BufWriter::new(output.unwrap());
    log_execute(
        BufReader::new(log_file), 
        &mut file_writer, 
        &l2c,
        flags.separator.unwrap_or(",".to_string())
    );
    println!("Wrote output to {}.", output_path.as_path().display());
}
