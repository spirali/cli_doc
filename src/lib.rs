mod commands;
mod extractor;
mod html;
mod settings;

use crate::extractor::runner::collect_program_info;
use crate::html::render_html;
use std::path::Path;

pub fn create_cheatsheet(path: &Path) -> anyhow::Result<String> {
    use colored::Colorize;
    let cmd = collect_program_info(path)?;
    println!("Creating templates ...");
    render_html(&cmd)
}

#[cfg(test)]
mod tests;
mod text;
//
// #[cfg(test)]
// mod tests {
//     use std::fmt::Debug;
//     use clap::{CommandFactory, Parser, Subcommand};
//     use crate::commands::{CommandDesc, OptionDesc};
//     use crate::renderer::render::RenderContext;
//     use super::*;
//
//     #[test]
//     fn it_works() {
//
//         let cmd = CommandDesc { name: "X".to_string(), help: String::new(), options: vec![], commands: vec![
//             CommandDesc { name: "hq server".to_string(), commands: vec![], options: vec![
//                 OptionDesc {
//                     long: "--do-something".to_string(),
//                     help: "Starts server".to_string(),
//                 }
//             ],
//                 help: "Server commands".to_string(),
//             },
//             CommandDesc { name: "worker".to_string(), help: "".to_string(), options: vec![], commands: vec![] }
//         ] };
//
//         let mut render_ctx = RenderContext::new();
//         render_ctx.render_command(&cmd);
//         let result = render_ctx.build();
//         std::fs::write("out.svg", result).unwrap();
//         // #[derive(Parser)]
//         // #[command(version, about, long_about = None)]
//         // struct Args {
//         //     filename: String,
//         //
//         //     #[arg(long, default_value = "4050")]
//         //     port: u16,
//         //
//         //     #[arg(long)]
//         //     key: Option<String>,
//         //
//         //     #[clap(subcommand)]
//         //     cmd: MainCommand,
//         // }
//         //
//         // #[derive(Parser)]
//         // #[command(version, about, long_about = None)]
//         // enum MainCommand {
//         //     DoSomething,
//         //     FooBar,
//         // }
//
//         // let mut cmd = Args::command();
//         // cmd.build();
//         // for a in cmd.get_arguments() {
//         //     println!("{:?}", a);
//         //     println!("{}", a.get_id());
//         // }
//         // cmd.print_help()
//         // for c in cmd.get_subcommands() {
//         //     println!("SUB {:?}", c)
//         // }
//     }
// }
