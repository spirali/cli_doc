use crate::commands::{
    ArgumentDesc, CategoryDesc, CommandDesc, CommandDoc, CommandId, CommandOuterDoc, OptionDesc,
    Usage, UsagePart,
};
use crate::extractor::sections::Section;
use crate::text::RichText;
use anyhow::bail;
use itertools::Either;

pub(crate) struct ClapParser {}

impl ClapParser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(
        &self,
        sections: &mut [Section],
    ) -> anyhow::Result<(CommandDoc, Vec<CommandOuterDoc>)> {
        if sections.is_empty() {
            bail!("Input is empty");
        };
        let mut brief = RichText::new();
        sections[0].as_rich_text_into(&mut brief);

        let mut desc = RichText::new();
        let mut intro_section = 1;
        let usage = loop {
            if let Some(s) = sections.get(intro_section) {
                if s.first_line().starts_with("Usage: ") {
                    let usage_content = s.first_line().strip_prefix("Usage: ").unwrap();
                    let mut usage = vec![usage_content];
                    for s in s.flatten_child_lines() {
                        usage.push(s);
                    }
                    break usage;
                }
                s.as_rich_text_into(&mut desc);
                intro_section += 1;
            } else {
                bail!("Usage section not found");
            }
        };
        let sections = &mut sections[intro_section..];
        let commands = self.extract_commands(sections);
        let arguments = self.extract_arguments(sections);
        let option_categories = self.extract_options(sections);

        Ok((
            CommandDoc {
                brief,
                description: if desc.is_empty() { None } else { Some(desc) },
                usage: usage.iter().map(|s| parse_usage(s)).collect(),
                arguments,
                option_categories,
            },
            commands,
        ))
    }

    fn extract_options(&self, sections: &mut [Section]) -> Vec<CategoryDesc> {
        let mut option_sections = Vec::new();
        let mut categories = Vec::new();
        for section in sections {
            option_sections.clear();
            if section.first_line().starts_with("Commands:") {
                continue;
            }
            if section.first_line().starts_with("Arguments:") {
                continue;
            }
            let Some(title) = section.first_line().strip_suffix(':') else {
                continue;
            };
            section.extract_sections_upto_ident(8, &mut option_sections);
            let options: Vec<_> = option_sections
                .iter()
                .flat_map(|s| {
                    if s.lines().len() == 1 {
                        let (left, right) = split_once2(s.first_line(), "  ");
                        let (short, long) = split_short_long(left);
                        let (brief, description) = if right.is_empty() {
                            s.subsections_as_brief_and_full_description()
                        } else {
                            (RichText::from_single_line(right.trim()), None)
                        };
                        Either::Left(
                            Some(OptionDesc {
                                short,
                                long,
                                brief,
                                description,
                            })
                            .into_iter(),
                        )
                    } else {
                        Either::Right(s.lines().iter().map(|s| {
                            let (left, right) = split_once2(s, "  ");
                            let (short, long) = split_short_long(left);
                            let brief = RichText::from_single_line(right.trim());
                            OptionDesc {
                                short,
                                long,
                                brief,
                                description: None,
                            }
                        }))
                    }
                })
                .collect();

            categories.push(CategoryDesc {
                title: title.to_string(),
                options,
            });
        }
        categories
    }

    fn extract_arguments(&self, sections: &[Section]) -> Vec<ArgumentDesc> {
        if let Some(section) = sections
            .iter()
            .find(|s| s.first_line().starts_with("Arguments:"))
        {
            section
                .subsections()
                .iter()
                .map(|s| {
                    if let Some((left, right)) = s.first_line().split_once("  ") {
                        ArgumentDesc {
                            name: left.trim().to_string(),
                            brief: RichText::from_single_line(right.trim()),
                            description: None,
                        }
                    } else {
                        let (brief, description) = s.subsections_as_brief_and_full_description();
                        ArgumentDesc {
                            name: s.paragraph(),
                            brief,
                            description,
                        }
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    fn extract_commands(&self, sections: &[Section]) -> Vec<CommandOuterDoc> {
        if let Some(section) = sections
            .iter()
            .find(|s| s.first_line().starts_with("Commands:"))
        {
            section
                .flatten_child_lines()
                .filter_map(|line| {
                    let Some((raw_name, desc)) = line.split_once("  ") else {
                        return None;
                    };
                    let (short, name) = if let Some((first, second)) = raw_name.split_once(",") {
                        let first = first.trim().to_string();
                        let second = second.trim().to_string();
                        if first.len() > second.len() {
                            (Some(second), first)
                        } else {
                            (Some(first), second)
                        }
                    } else {
                        (None, raw_name.trim().to_string())
                    };
                    if !name
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
                    {
                        return None;
                    }
                    Some(CommandOuterDoc {
                        name,
                        short,
                        description: desc.trim().to_string(),
                    })
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}

fn parse_usage(input: &str) -> Usage {
    Usage {
        parts: input
            .split_whitespace()
            .map(|s| {
                if s.starts_with('<') {
                    UsagePart::Argument(s.to_string())
                } else if s.starts_with('[') {
                    UsagePart::Option(s.to_string())
                } else {
                    UsagePart::Command(s.to_string())
                }
            })
            .collect(),
    }
}

fn split_once2<'a, 'b>(s: &'a str, sep: &'b str) -> (&'a str, &'a str) {
    if let Some((left, right)) = s.split_once(sep) {
        (left, right)
    } else {
        (s, "")
    }
}

fn split_short_long(s: &str) -> (Option<String>, String) {
    if let Some((first, second)) = s.split_once(',') {
        let first = first.trim().to_string();
        let second = second.trim().to_string();
        (Some(first), second)
    } else {
        (None, s.trim().to_string())
    }
}
