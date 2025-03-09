use std::io::Read;

use args::Args;
use clap::Parser;
use pattern::Pattern;

mod args;
mod matcher;
mod pattern;

pub mod utils {
    pub mod string_utils;
    pub mod match_utils;
}

pub mod printers {
    pub mod path_printer;
    pub mod json_printer;

    mod printer_node;
}

pub mod errors {
    pub mod parsing_error;
}

fn main() {
    let args = Args::parse();

    let content = if let Some(path) = args.path {
        let mut content = String::new();
        let file = std::fs::File::open(&path).unwrap_or_else(|_| {
            eprintln!("{}: No such file or directory", path);
            std::process::exit(2);
        });
        let mut reader = std::io::BufReader::new(file);
        reader.read_to_string(&mut content).unwrap();

        content
    } else {
        let mut content = String::new();
        std::io::stdin().read_to_string(&mut content).unwrap();
        content
    };

    let json = serde_json::from_str::<serde_json::Value>(&content).unwrap_or_else(|_| {
        eprintln!("Invalid JSON");
        std::process::exit(3);
    });

    let pattern = Pattern::parse(&args.pattern);
    let pattern = match pattern {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Invalid JSON");
            std::process::exit(3);
        },
    };

    let context = args.context.unwrap_or(0);

    let matches = matcher::match_pattern(&json, &pattern);

    if args.json {
        printers::json_printer::print(json, matches, context, &mut std::io::stdout());
    } else {
        printers::path_printer::print(json, matches, std::io::stdout());
    }


}
