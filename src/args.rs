use clap::Parser;


#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    pub pattern: String,
    pub path: Option<String>,

    #[clap(short, long)]
    pub json: bool,

    #[clap(short = 'C', long)]
    pub context: Option<usize>,
}

