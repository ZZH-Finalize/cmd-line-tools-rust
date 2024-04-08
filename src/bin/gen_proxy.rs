use clap::Parser;
use std::{path::PathBuf, process::ExitCode};

#[derive(Debug, Parser)]
#[command(version = "0.0.1")]
#[command(about = "Generate YAML proxy file for Clash from plant text")]
struct Cli {
    /// Set verbose Level to display debug information
    #[arg(short, long = "verbose", default_value = "0")]
    verbose_level: u8,

    /// Set slice length
    #[arg(short, long, default_value = "200")]
    slice: usize,

    /// Source files or dirs that contains the proxy informations
    files_or_dirs: Vec<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    println!("{:#?}", cli);

    ExitCode::SUCCESS
}
