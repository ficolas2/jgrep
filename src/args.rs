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

    /// Shows N levels of parent context around the match.
    #[clap(short = 'C', long)]
    pub context: Option<usize>,

    /// Output format: 'path' (default), 'json' (pruned tree), or 'only' (just the match and
    /// children).
    /// Equivalent to using -p, j, or -o.
    #[clap(short = 'p', long, value_enum, default_value="path")]
    pub printer: PrinterType,

    /// Shortcut for --printer=only.
    /// Show only the matching value, or json chunk.
    #[clap(short = 'o', long)]
    pub only: bool,

    /// Shortcut for --printer=path.
    /// Show the path and value of the match before the value or json chunk (default output).
    #[clap(short = 'P', long = "path")]
    pub path_printer: bool,

    /// Shortcut for --printer=json.
    /// Show a pruned JSON tree leading to the match
    #[clap(short, long)]
    pub json: bool,

}

