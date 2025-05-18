use std::io::{BufRead, Read};

use args::Args;
use clap::Parser;
use pattern::Pattern;

mod args;
mod matcher;
mod pattern;

pub mod utils {
    pub mod match_utils;
    pub mod string_utils;
}

pub mod printers {
    pub mod json_printer;
    pub mod path_printer;

    mod printer_node;
}

pub mod errors {
    pub mod parsing_error;
}

enum PrinterType {
    Path,
    Json,
}

fn process_complete_json(content: &str, printer: &PrinterType, context: usize, pattern: &Pattern) {
    let json = serde_json::from_str::<serde_json::Value>(content).unwrap_or_else(|_| {
        eprintln!("Invalid JSON");
        std::process::exit(3);
    });

    let matches = matcher::match_pattern(&json, pattern);

    match printer {
        PrinterType::Path => {
            printers::path_printer::print(json, matches, context, std::io::stdout())
        }
        PrinterType::Json => {
            printers::json_printer::print(json, matches, context, &mut std::io::stdout())
        }
    }
}

fn process_file(path: &str, printer: PrinterType, context: usize, pattern: &Pattern) {
    let mut content = String::new();
    let file = std::fs::File::open(path).unwrap_or_else(|_| {
        eprintln!("{}: No such file or directory", path);
        std::process::exit(2);
    });
    let mut reader = std::io::BufReader::new(file);
    reader.read_to_string(&mut content).unwrap();

    process_complete_json(&content, &printer, context, pattern);
}

fn stream_process(printer: PrinterType, context: usize, pattern: &Pattern) {
    let stdin = std::io::stdin();
    let mut buffer = String::new();

    let mut start = None;
    let mut depth = 0;

    for line in stdin.lock().lines() {
        if line.is_err() {
            eprintln!("Error reading from stdin");
            std::process::exit(1);
        }
        let line = line.unwrap();
        let mut line = line.trim();

        for (i, c) in line.chars().enumerate() {
            match (start, c) {
                (None, '{') | (None, '[') => {
                    depth = 1;
                    start = Some(c)
                }
                (None, c) => {
                    eprintln!("Invalid JSON, must start with '{{' or '[', starts with '{c}'");
                    std::process::exit(3);
                }
                (Some('{'), '{') | (Some('['), '[') => depth += 1,
                (Some('{'), '}') | (Some('['), ']') => depth -= 1,
                _ => {}
            }

            if depth == 0 {
                buffer.push_str(&line[0..=i]);
                process_complete_json(&buffer, &printer, context, pattern);
                buffer.clear();
                start = None;
                line = &line[i + 1..];
            }
        }

        buffer.push_str(line);
        buffer.push('\n');
    }
}

fn main() {
    let args = Args::parse();

    // Parse pattern
    let pattern = Pattern::parse(&args.pattern);
    let pattern = match pattern {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Invalid JSON");
            std::process::exit(3);
        }
    };

    let context = args.context.unwrap_or(0);

    // Get printer to be used
    let printer = if args.json {
        PrinterType::Json
    } else {
        PrinterType::Path
    };

    if let Some(path) = args.path {
        process_file(&path, printer, context, &pattern);
    } else {
        stream_process(printer, context, &pattern);
    };
}
