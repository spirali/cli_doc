use std::collections::HashMap;
use std::fmt::Debug;
use crate::commands::{CommandDesc, CommandId, ProgramDesc};
use askama::{filters, Template};
use askama::filters::{Escaper, Html};
use serde::Serialize;
use crate::text::RichText;


#[derive(Template)]
#[template(path = "page.html")]
struct PageTemplate<'a> {
    project_name: &'a str,
    command: CommandTemplate<'a>,
    command_json: String,
}

#[derive(Template)]
#[template(path = "command.html", escape = "none")]
struct CommandTemplate<'a> {
    name: &'a str,
    id: CommandId,
    depth: u32,
    subcommands: Vec<CommandTemplate<'a>>,
}

#[derive(Template)]
#[template(path = "richtext.html", escape = "none")]
struct RichTextTemplate<'a> {
    text: &'a RichText
}

#[derive(Serialize)]
struct OptionJson<'a> {
    short: Option<&'a str>,
    long: String,
    brief: String,
    description: Option<String>,
}

#[derive(Serialize)]
struct CategoryJson<'a> {
    title: &'a str,
    options: Vec<OptionJson<'a>>
}

#[derive(Serialize)]
struct CommandJson<'a> {
    name: &'a str,
    brief: String,
    description: Option<String>,
    usages: Vec<String>,
    categories: Vec<CategoryJson<'a>>,
}

impl<'a> CommandJson<'a> {
    pub fn new(desc: &'a CommandDesc) -> Self {
        let html = Html::default();
        CommandJson {
            name: &desc.name,
            brief: desc.doc.brief.to_html(),
            description: desc.doc.description.as_ref().map(|t| t.to_html()),
            usages: desc.doc.usage.iter().map(|u| u.to_html()).collect(),
            categories: desc.doc.option_categories.iter().map(|c| CategoryJson {
                title: &c.title,
                options: c.options.iter().map(|o| OptionJson {
                    short: o.short.as_deref(),
                    long: { let mut s = String::new(); html.write_escaped_str(&mut s, &o.long).unwrap(); s },
                    brief: o.brief.to_html(),
                    description: o.description.as_ref().map(|t| t.to_html()),
                }).collect(),
            }).collect(),
        }
    }
}

fn build_command_json<'a>(command: &'a CommandDesc, out: &mut HashMap<String, CommandJson<'a>>) {
    let id = format!("c{}", command.id);
    out.insert(id, CommandJson::new(command));
    for c in &command.commands {
        build_command_json(c, out);
    }
}

fn build_command_tree<'a, 'b>(
    command: &'a CommandDesc,
    depth: u32,
) -> CommandTemplate<'a> {
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
    let mut command_jsons = Default::default();
    build_command_json(&program.command, &mut command_jsons);
    Ok(PageTemplate {
        project_name: &program.command.name,
        command: command_template,
        command_json: serde_json::to_string(&command_jsons)?,
    }
    .render()?)
}
