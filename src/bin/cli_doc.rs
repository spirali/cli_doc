use clap::Parser;
use clap_cheatsheet::create_cheatsheet;
use colored::Colorize;
use std::path::{Path, PathBuf};

/// Generator of documentation for CLI
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    program: PathBuf,
}

pub fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let svg = create_cheatsheet(&args.program)?;
    let output = "out.html";
    std::fs::write(Path::new(&output), &svg)?;
    println!("Output written to: {}", output.green());
    Ok(())
}
