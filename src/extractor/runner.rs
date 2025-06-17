use crate::commands::{CommandDesc, CommandId, ProgramDesc};
use crate::extractor::clap_parser::ClapParser;
use crate::extractor::sections::parse_sections;
use anyhow::{anyhow, bail};
use colored::Colorize;
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

fn get_program_output(program: &Path, args: &[String], flag: &str) -> anyhow::Result<String> {
    print!("Running {}", program.display().to_string().cyan());
    for arg in args {
        print!(" {}", arg.yellow());
    }
    println!(" {}", flag.magenta());
    let output = Command::new(program).args(args).arg(flag).output()?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)
            .map_err(|_| anyhow!("Invalid output of command with {} flag", flag))?)
    } else {
        println!(
            "{:?}",
            String::from_utf8(output.stderr).unwrap_or("<INVALID UTF8>".to_string())
        );
        bail!("Invalid invocation of command with {} flag", flag)
    }
}

fn concat_lines(s: String, lines: &[&str]) -> String {
    let mut result = s;
    if !result.is_empty() && !lines.is_empty() {
        result.push('\n');
    }
    for (i, line) in lines.iter().enumerate() {
        result.push_str(line.trim());
        if i + 1 < lines.len() {
            result.push('\n');
        }
    }
    result
}

fn first_split_or_all<'a, 'b>(input: &'a str, delimiter: &'b str) -> &'a str {
    input.split(delimiter).next().unwrap_or(input)
}

fn gather_command_helper(
    program: &Path,
    args: &mut Vec<String>,
    id_counter: &mut CommandId,
) -> anyhow::Result<CommandDesc> {
    let id = *id_counter;
    *id_counter += 1;

    let output = get_program_output(program, args.as_slice(), "--help")?;
    let mut sections = parse_sections(&output);
    let (command_doc, subcommands) = ClapParser::new().parse(&mut sections)?;

    let commands: Vec<_> = subcommands
        .into_iter()
        .map(|s| {
            args.push(s.name.to_string());
            let r = gather_command_helper(program, args, id_counter);
            args.pop();
            r
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let name = args
        .last()
        .map(|x| x.to_string())
        .unwrap_or_else(|| program.file_name().unwrap().to_string_lossy().to_string());

    Ok(CommandDesc {
        id,
        name,
        doc: command_doc,
        commands,
    })
}

pub fn collect_program_info(program: &Path) -> anyhow::Result<ProgramDesc> {
    let mut args: Vec<String> = Vec::new();
    let mut id_counter = 0;
    let mut already_defined = HashSet::new();
    already_defined.insert("-h, --help");
    let command = gather_command_helper(program, &mut args, &mut id_counter)?;
    let version = get_program_output(program, &args, "--version")?
        .trim()
        .to_string();
    Ok(ProgramDesc { command, version })
}
