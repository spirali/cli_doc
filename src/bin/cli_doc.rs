use clap::Parser;
use cli_doc::create_html_doc;
use colored::Colorize;
use std::path::{Path, PathBuf};

/// Generator of documentation for CLI
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    program: PathBuf,

    #[clap(long, default_value = "doc.html")]
    output_filename: PathBuf,
}

pub fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let svg = create_html_doc(&args.program)?;
    std::fs::write(&args.output_filename, &svg)?;
    println!("Output written into: {}", args.output_filename.display().to_string().green());
    Ok(())
}
