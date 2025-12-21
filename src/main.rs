use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process;

use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use regex::Regex;

use crate::error::{ErrorType, ParseError};
use crate::expression::{ConstraintExpression, check_constraint, parse_expression};

mod error;
mod expression;

#[derive(Debug)]
struct L2C {
    regex: Regex,
    order: Vec<String>,
    constraints: Vec<ConstraintExpression>,
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

fn for_each_line<F>(source: &str, mut f: F)
where
    F: FnMut(&str, usize),
{
    let bytes = source.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let line_start = i;

        // Scan until newline or EOF
        while i < bytes.len() && bytes[i] != b'\n' && bytes[i] != b'\r' {
            i += 1;
        }

        let line_end = i;
        let line = &source[line_start..line_end];

        f(line, line_start);

        // Handle newline
        if i < bytes.len() {
            if bytes[i] == b'\r' {
                i += 1;
                if i < bytes.len() && bytes[i] == b'\n' {
                    i += 1; // Windows CRLF
                }
            } else {
                i += 1; // Unix LF
            }
        }
    }
}

fn l2c_parse(source: &str, file_id: usize) -> Result<L2C, Vec<Diagnostic<usize>>> {
    let mut regex: Option<Regex> = None;
    let mut order: Option<Vec<String>> = None;
    let mut constraints = Vec::new();
    let mut diagnostics = Vec::new();

    for_each_line(source, |line, line_start| {
        let original_line = line;

        let trimmed_line = original_line.trim_start();
        let trim_offset = original_line.len() - trimmed_line.len();

        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            return;
        }

        if let Some((key, value)) = get_kv_pair(trimmed_line) {
            match key.to_lowercase().as_str() {
                "regex" => {
                    if let Ok(r) = Regex::new(value) {
                        regex = Some(r);
                    } else {
                        let start = line_start + trim_offset;
                        let end = start + key.len();

                        diagnostics.push(
                            ParseError {
                                file: file_id,
                                span: start..end,
                                error_type: ErrorType::InvalidSyntax,
                            }
                            .to_diagnostic(),
                        );
                    }
                }

                "order" => {
                    order = Some(get_list(value));
                }

                "constraint" => match parse_expression(value) {
                    Ok(expr) => constraints.push(expr),
                    Err(_) => {
                        let start = line_start + trim_offset;
                        let end = start + key.len();

                        diagnostics.push(
                            ParseError {
                                file: file_id,
                                span: start..end,
                                error_type: ErrorType::InvalidSyntax,
                            }
                            .to_diagnostic(),
                        );
                    }
                },

                _ => {
                    let key_start_in_trimmed = trimmed_line.find(key).unwrap_or(0);
                    let start = line_start + trim_offset + key_start_in_trimmed;
                    let end = start + key.len();

                    diagnostics.push(
                        ParseError {
                            file: file_id,
                            span: start..end,
                            error_type: ErrorType::UnknownKey,
                        }
                        .to_diagnostic(),
                    );
                }
            }
        } else {
            diagnostics.push(
                ParseError {
                    file: file_id,
                    span: line_start..(line_start + original_line.len()),
                    error_type: ErrorType::InvalidSyntax,
                }
                .to_diagnostic(),
            );
        }
    });

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    Ok(L2C {
        regex: regex.unwrap(),
        order: order.unwrap_or_default(),
        constraints,
    })
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

    'outer: for line in input.lines().map_while(Result::ok) {
        let captures = l2c.regex.captures(&line);
        if captures.is_none() {
            continue;
        }
        let captures = captures.unwrap();

        for constraint in l2c.constraints.iter() {
            if !check_constraint(&constraint, &captures) {
                continue 'outer;
            }
        }

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

    let l2c_path = flags.l2c_path.as_path();
    let l2c_file = match fs::read_to_string(l2c_path) {
        Ok(file) => file,
        Err(err) => panic!("Could not open file: {}", err),
    };

    let mut files = SimpleFiles::new();
    let file_id = files.add(l2c_path.file_name().unwrap().to_str().unwrap(), &l2c_file);

    let l2c = match l2c_parse(&l2c_file, file_id) {
        Ok(l2c) => l2c,
        Err(diagnostics) => {
            let writer = StandardStream::stderr(ColorChoice::Always);
            let config = codespan_reporting::term::Config::default();

            for diagnostic in diagnostics {
                term::emit_to_io_write(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
            }

            process::exit(1);
        }
    };

    let output_path = flags
        .output
        .unwrap_or(Path::new("output.csv").to_path_buf());
    let output = File::create(&output_path);
    if output.is_err() {
        panic!(
            "The output file could not be created: {}",
            output.unwrap_err()
        );
    }

    let mut file_writer = BufWriter::new(output.unwrap());
    log_execute(
        BufReader::new(log_file),
        &mut file_writer,
        &l2c,
        flags.separator.unwrap_or(",".to_string()),
    );
    println!("Wrote output to {}.", output_path.as_path().display());
}
