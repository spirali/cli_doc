use crate::text::RichText;
use askama::filters::{Escaper, Html};
use std::collections::HashSet;

pub type CommandId = u32;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct OptionDesc {
    pub short: Option<String>,
    pub long: String,
    pub brief: RichText,
    pub description: Option<RichText>,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct ArgumentDesc {
    pub name: String,
    pub brief: RichText,
    pub description: Option<RichText>,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct CategoryDesc {
    pub title: String,
    pub options: Vec<OptionDesc>,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum UsagePart {
    Command(String),
    Argument(String),
    Option(String),
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Usage {
    pub parts: Vec<UsagePart>,
}

impl Usage {
    pub fn to_html(&self) -> String {
        let mut out = String::new();
        let html = Html;
        for part in &self.parts {
            match part {
                UsagePart::Command(command) => {
                    out.push_str("<span class=\"usage-command\">");
                    html.write_escaped_str(&mut out, command).unwrap();
                    out.push_str("</span> ");
                }
                UsagePart::Argument(argument) => {
                    out.push_str("<span class=\"usage-argument\">");
                    html.write_escaped_str(&mut out, argument).unwrap();
                    out.push_str("</span> ");
                }
                UsagePart::Option(option) => {
                    out.push_str("<span class=\"usage-option\">");
                    html.write_escaped_str(&mut out, option).unwrap();
                    out.push_str("</span> ");
                }
            }
        }
        if !self.parts.is_empty() {
            out.pop();
        }
        out
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct CommandDoc {
    pub brief: RichText,
    pub description: Option<RichText>,
    pub usage: Vec<Usage>,
    pub arguments: Vec<ArgumentDesc>,
    pub option_categories: Vec<CategoryDesc>,
}

impl CommandDoc {
    pub fn is_args_effectively_empty(&self) -> bool {
        self.arguments.iter().all(|arg| arg.brief.is_empty())
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct CommandOuterDoc {
    pub name: String,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct CommandDesc {
    pub id: CommandId,
    pub name: String,
    pub doc: CommandDoc,
    pub commands: Vec<CommandDesc>,
}

impl CommandDesc {
    fn prune_repeated_options_helper(&mut self, parent_options: &HashSet<&OptionDesc>) {
        self.doc.option_categories.retain_mut(|category| {
            category
                .options
                .retain(|option| !parent_options.contains(option));
            !category.options.is_empty()
        });
        let mut my_options = parent_options.clone();
        for category in &self.doc.option_categories {
            for option in &category.options {
                my_options.insert(option);
            }
        }

        for command in &mut self.commands {
            command.prune_repeated_options_helper(&my_options);
        }
    }
    pub fn prune_repeated_options(&mut self) {
        self.prune_repeated_options_helper(&HashSet::new());
    }
}

#[derive(Debug)]
pub struct ProgramDesc {
    pub command: CommandDesc,
    pub version: String,
}
