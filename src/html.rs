use crate::commands::{CommandDesc, CommandId, ProgramDesc};
use askama::Template;
use askama::filters::{Escaper, Html};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Template)]
#[template(path = "page.html")]
struct PageTemplate<'a> {
    project_name: &'a str,
    version: &'a str,
    command: CommandTemplate<'a>,
    command_json: String,
    generator: String,
}

#[derive(Template)]
#[template(path = "command.html", escape = "none")]
struct CommandTemplate<'a> {
    name: &'a str,
    id: CommandId,
    depth: u32,
    subcommands: Vec<CommandTemplate<'a>>,
}

#[derive(Serialize)]
struct OptionJson<'a> {
    id: String,
    short: Option<&'a str>,
    long: String,
    brief: String,
    description: Option<String>,
}

#[derive(Serialize)]
struct ArgumentJson {
    name: String,
    brief: String,
    description: Option<String>,
}

#[derive(Serialize)]
struct CategoryJson<'a> {
    title: &'a str,
    options: Vec<OptionJson<'a>>,
}

#[derive(Serialize)]
struct CommandJson<'a> {
    name: String,
    parent: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    children: Vec<String>,
    brief: String,
    description: Option<String>,
    usages: Vec<String>,
    arguments: Vec<ArgumentJson>,
    categories: Vec<CategoryJson<'a>>,
}

impl<'a> CommandJson<'a> {
    pub fn new(desc: &'a CommandDesc, parent: Option<&str>) -> Self {
        let mut option_id = 0;
        CommandJson {
            name: desc.name.to_string(),
            parent: parent.map(|x| x.to_string()),
            children: desc.commands.iter().map(|c| format!("c{}", c.id)).collect(),
            brief: desc.doc.brief.to_html(),
            description: desc.doc.description.as_ref().map(|t| t.to_html()),
            usages: desc.doc.usage.iter().map(|u| u.to_html()).collect(),
            arguments: if desc.doc.is_args_effectively_empty() {
                Vec::new()
            } else {
                desc.doc
                    .arguments
                    .iter()
                    .map(|a| ArgumentJson {
                        name: escape_html(&a.name),
                        brief: a.brief.to_html(),
                        description: a.description.as_ref().map(|t| t.to_html()),
                    })
                    .collect()
            },
            categories: desc
                .doc
                .option_categories
                .iter()
                .map(|c| CategoryJson {
                    title: &c.title,
                    options: c
                        .options
                        .iter()
                        .map(|o| OptionJson {
                            id: {
                                option_id += 1;
                                format!("o{}-{}", desc.id, option_id)
                            },
                            short: o.short.as_deref(),
                            long: escape_html(&o.long),
                            brief: o.brief.to_html(),
                            description: o.description.as_ref().map(|t| t.to_html()),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

fn build_command_json<'a>(
    command: &'a CommandDesc,
    parent: Option<&str>,
    out: &mut HashMap<String, CommandJson<'a>>,
) {
    let id = format!("c{}", command.id);
    for c in &command.commands {
        build_command_json(c, Some(id.as_str()), out);
    }
    out.insert(id, CommandJson::new(command, parent));
}

fn escape_html(s: &str) -> String {
    let html = Html;
    let mut out = String::new();
    html.write_escaped_str(&mut out, s).unwrap();
    out
}

fn build_command_tree(command: &CommandDesc, depth: u32) -> CommandTemplate {
    let subcommands = command
        .commands
        .iter()
        .map(|c| build_command_tree(c, depth + 1))
        .collect();
    CommandTemplate {
        name: &command.name,
        id: command.id,
        subcommands,
        depth,
    }
}

pub fn render_html(program: &ProgramDesc) -> anyhow::Result<String> {
    let command_template = build_command_tree(&program.command, 0);
    let mut command_jsons: HashMap<String, CommandJson> = Default::default();
    build_command_json(&program.command, None, &mut command_jsons);

    Ok(PageTemplate {
        project_name: &program.command.name,
        version: &program.version,
        command: command_template,
        command_json: serde_json::to_string(&command_jsons)?,
        generator: format!("cli-doc {}", env!("CARGO_PKG_VERSION")),
    }
    .render()?)
}
