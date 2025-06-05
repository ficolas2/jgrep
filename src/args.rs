use clap::Parser;

use crate::PrinterType;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(group(
    clap::ArgGroup::new("printer_group")
        .args(&["printer", "only", "json", "path_printer"])
        .multiple(false)
))]
pub struct Args {
    pub pattern: String,
    pub path: Option<String>,

    /// Shows N levels of parent context around the match
    #[clap(short = 'C', long)]
    pub context: Option<usize>,

    /// Output format: 'path' (default), 'json' (pruned tree), or 'only' (just the match).
    /// Equivalent to using -p, j, or -o.
    #[clap(short = 'p', long, value_enum, default_value="path")]
    pub printer: PrinterType,

    /// Show only the matching key, value, or key-value pair.
    #[clap(short = 'o', long)]
    pub only: bool,

    /// Show only the path and value of the match (default output).
    #[clap(short = 'P', long = "path")]
    pub path_printer: bool,

    /// Show a pruned JSON tree leading to the match (includes context if -C is used).
    /// Shortcut for --printer=json
    #[clap(short, long)]
    pub json: bool,

}

