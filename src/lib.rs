mod commands;
mod extractor;
mod html;
mod text;

use crate::extractor::runner::collect_program_info;
use crate::html::render_html;
use std::path::Path;

pub fn create_html_doc(path: &Path) -> anyhow::Result<String> {
    let mut program = collect_program_info(path)?;
    program.command.prune_repeated_options();
    println!("Rendering HTML ...");
    render_html(&program)
}
