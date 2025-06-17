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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::sections::parse_sections;
    use crate::text::RichText;
    use insta::{assert_debug_snapshot, assert_snapshot};

    fn parse_clap(text: &str) -> (CommandDoc, Vec<CommandOuterDoc>) {
        let mut sections = parse_sections(text);
        ClapParser::new().parse(&mut sections).unwrap()
    }

    #[test]
    fn test_parse_clap_minimal() {
        let text = "Simple program to greet a person

Usage: test-minimal --name <NAME>

Options:
  -n, --name <NAME>  The name of the person to greet
  -h, --help         Print help
  -V, --version      Print version
";
        let (doc, commands) = parse_clap(text);
        assert_debug_snapshot!(doc, @r#"
        CommandDoc {
            intro: RichText {
                parts: [
                    Text(
                        "Simple program to greet a person",
                    ),
                ],
            },
            usage: [
                "test-minimal --name <NAME>",
            ],
            arguments: [],
            option_categories: [
                CategoryDesc {
                    title: "Options",
                    options: [
                        OptionDesc {
                            short: Some(
                                "-n",
                            ),
                            long: "--name <NAME>",
                            description: RichText {
                                parts: [
                                    Text(
                                        "The name of the person to greet",
                                    ),
                                ],
                            },
                        },
                        OptionDesc {
                            short: Some(
                                "-h",
                            ),
                            long: "--help",
                            description: RichText {
                                parts: [
                                    Text(
                                        "Print help",
                                    ),
                                ],
                            },
                        },
                        OptionDesc {
                            short: Some(
                                "-V",
                            ),
                            long: "--version",
                            description: RichText {
                                parts: [
                                    Text(
                                        "Print version",
                                    ),
                                ],
                            },
                        },
                    ],
                },
            ],
        }
        "#);
        assert!(commands.is_empty());
    }

    #[test]
    fn test_parse_args() {
        let text = "Submit a new job

Usage: hq submit [OPTIONS] <COMMANDS>...

Arguments:
  <COMMANDS>...
          Command that should be executed by each task

Options:
      --name <NAME>
          The name of the job
";
        let (doc, commands) = parse_clap(text);
        assert_debug_snapshot!(doc, @r#"
        CommandDoc {
            intro: RichText {
                parts: [
                    Text(
                        "Submit a new job",
                    ),
                ],
            },
            usage: [
                "hq submit [OPTIONS] <COMMANDS>...",
            ],
            arguments: [],
            option_categories: [
                CategoryDesc {
                    title: "Options",
                    options: [
                        OptionDesc {
                            short: None,
                            long: "--name <NAME>",
                            description: RichText {
                                parts: [
                                    Text(
                                        "The name of the job",
                                    ),
                                ],
                            },
                        },
                    ],
                },
            ],
        }
        "#);
        assert!(commands.is_empty());
    }

    #[test]
    fn test_parse_clap_cargo() {
        let text = "Rust's package manager

Usage: cargo [+toolchain] [OPTIONS] [COMMAND]
       cargo [+toolchain] [OPTIONS] -Zscript <MANIFEST_RS> [ARGS]...

Options:
  -V, --version
          Print version info and exit
      --list
          List installed commands
      --explain <CODE>
          Provide a detailed explanation of a rustc error message
  -v, --verbose...
          Use verbose output (-vv very verbose/build.rs output)
  -q, --quiet
          Do not print cargo log messages
      --color <WHEN>
          Coloring [possible values: auto, always, never]
  -C <DIRECTORY>
          Change to DIRECTORY before doing anything (nightly-only)
      --locked
          Assert that `Cargo.lock` will remain unchanged
      --offline
          Run without accessing the network
      --frozen
          Equivalent to specifying both --locked and --offline
      --config <KEY=VALUE|PATH>
          Override a configuration value
  -Z <FLAG>
          Unstable (nightly-only) flags to Cargo, see 'cargo -Z help' for
          details
  -h, --help
          Print help

Commands:
    build, b    Compile the current package
    check, c    Analyze the current package and report errors, but don't build object files
    clean       Remove the target directory
    doc, d      Build this package's and its dependencies' documentation
    new         Create a new cargo package
    init        Create a new cargo package in an existing directory
    add         Add dependencies to a manifest file
    remove      Remove dependencies from a manifest file
    run, r      Run a binary or example of the local package
    test, t     Run the tests
    bench       Run the benchmarks
    update      Update dependencies listed in Cargo.lock
    search      Search registry for crates
    publish     Package and upload this package to the registry
    install     Install a Rust binary
    uninstall   Uninstall a Rust binary
    ...         See all commands with --list

See 'cargo help <command>' for more information on a specific command.
";
        let (doc, commands) = parse_clap(text);
        assert_debug_snapshot!(doc, @r#"
        CommandDoc {
            intro: RichText {
                parts: [
                    Text(
                        "Rust's package manager",
                    ),
                ],
            },
            usage: [
                "cargo [+toolchain] [OPTIONS] [COMMAND]",
                "cargo [+toolchain] [OPTIONS] -Zscript <MANIFEST_RS> [ARGS]...",
            ],
            arguments: [],
            option_categories: [
                CategoryDesc {
                    title: "Options",
                    options: [
                        OptionDesc {
                            short: Some(
                                "-V",
                            ),
                            long: "--version",
                            description: RichText {
                                parts: [
                                    Text(
                                        "Print version info and exit",
                                    ),
                                ],
                            },
                        },
                        OptionDesc {
                            short: None,
                            long: "--list",
                            description: RichText {
                                parts: [
                                    Text(
                                        "List installed commands",
                                    ),
                                ],
                            },
                        },
                        OptionDesc {
                            short: None,
                            long: "--explain <CODE>",
                            description: RichText {
                                parts: [
                                    Text(
                                        "Provide a detailed explanation of a rustc error message",
                                    ),
                                ],
                            },
                        },
                        OptionDesc {
                            short: Some(
                                "-v",
                            ),
                            long: "--verbose...",
                            description: RichText {
                                parts: [
                                    Text(
                                        "Use verbose output (-vv very verbose/build.rs output)",
                                    ),
                                ],
                            },
                        },
                        OptionDesc {
                            short: Some(
                                "-q",
                            ),
                            long: "--quiet",
                            description: RichText {
                                parts: [
                                    Text(
                                        "Do not print cargo log messages",
                                    ),
                                ],
                            },
                        },
                        OptionDesc {
                            short: None,
                            long: "--color <WHEN>",
                            description: RichText {
                                parts: [
                                    Text(
                                        "Coloring [possible values: auto, always, never]",
                                    ),
                                ],
                            },
                        },
                        OptionDesc {
                            short: None,
                            long: "-C <DIRECTORY>",
                            description: RichText {
                                parts: [
                                    Text(
                                        "Change to DIRECTORY before doing anything (nightly-only)",
                                    ),
                                ],
                            },
                        },
                        OptionDesc {
                            short: None,
                            long: "--locked",
                            description: RichText {
                                parts: [
                                    Text(
                                        "Assert that `Cargo.lock` will remain unchanged",
                                    ),
                                ],
                            },
                        },
                        OptionDesc {
                            short: None,
                            long: "--offline",
                            description: RichText {
                                parts: [
                                    Text(
                                        "Run without accessing the network",
                                    ),
                                ],
                            },
                        },
                        OptionDesc {
                            short: None,
                            long: "--frozen",
                            description: RichText {
                                parts: [
                                    Text(
                                        "Equivalent to specifying both --locked and --offline",
                                    ),
                                ],
                            },
                        },
                        OptionDesc {
                            short: None,
                            long: "--config <KEY=VALUE|PATH>",
                            description: RichText {
                                parts: [
                                    Text(
                                        "Override a configuration value",
                                    ),
                                ],
                            },
                        },
                        OptionDesc {
                            short: None,
                            long: "-Z <FLAG>",
                            description: RichText {
                                parts: [
                                    Text(
                                        "Unstable (nightly-only) flags to Cargo, see 'cargo -Z help' for details",
                                    ),
                                ],
                            },
                        },
                        OptionDesc {
                            short: Some(
                                "-h",
                            ),
                            long: "--help",
                            description: RichText {
                                parts: [
                                    Text(
                                        "Print help",
                                    ),
                                ],
                            },
                        },
                    ],
                },
            ],
        }
        "#);
        assert_debug_snapshot!(commands, @r#"
        [
            CommandOuterDoc {
                name: "build",
                short: Some(
                    "b",
                ),
                description: "Compile the current package",
            },
            CommandOuterDoc {
                name: "check",
                short: Some(
                    "c",
                ),
                description: "Analyze the current package and report errors, but don't build object files",
            },
            CommandOuterDoc {
                name: "clean",
                short: None,
                description: "Remove the target directory",
            },
            CommandOuterDoc {
                name: "doc",
                short: Some(
                    "d",
                ),
                description: "Build this package's and its dependencies' documentation",
            },
            CommandOuterDoc {
                name: "new",
                short: None,
                description: "Create a new cargo package",
            },
            CommandOuterDoc {
                name: "init",
                short: None,
                description: "Create a new cargo package in an existing directory",
            },
            CommandOuterDoc {
                name: "add",
                short: None,
                description: "Add dependencies to a manifest file",
            },
            CommandOuterDoc {
                name: "remove",
                short: None,
                description: "Remove dependencies from a manifest file",
            },
            CommandOuterDoc {
                name: "run",
                short: Some(
                    "r",
                ),
                description: "Run a binary or example of the local package",
            },
            CommandOuterDoc {
                name: "test",
                short: Some(
                    "t",
                ),
                description: "Run the tests",
            },
            CommandOuterDoc {
                name: "bench",
                short: None,
                description: "Run the benchmarks",
            },
            CommandOuterDoc {
                name: "update",
                short: None,
                description: "Update dependencies listed in Cargo.lock",
            },
            CommandOuterDoc {
                name: "search",
                short: None,
                description: "Search registry for crates",
            },
            CommandOuterDoc {
                name: "publish",
                short: None,
                description: "Package and upload this package to the registry",
            },
            CommandOuterDoc {
                name: "install",
                short: None,
                description: "Install a Rust binary",
            },
            CommandOuterDoc {
                name: "uninstall",
                short: None,
                description: "Uninstall a Rust binary",
            },
        ]
        "#);
    }

    #[test]
    fn test_parse_clap_hq() {
        let text = "Commands for the server

a multi line title

Usage: hq server [OPTIONS] <COMMAND>

Commands:
  start            Start the server
  stop             Stop the server
  info             Show info of a running server
  generate-access  Generate an access file without starting the server

Options:
  -h, --help
          Print help (see a summary with '-h')

GLOBAL OPTIONS:
  --server-dir <SERVER_DIR>
          The path where access files are stored

          [env: HQ_SERVER_DIR=]

  --colors <COLORS>
          Sets console color policy

          [default: auto]

          Possible values:
          - auto:   Use colors if the stdout is detected to be a terminal
          - always: Always use colors
          - never:  Never use colors

  --output-mode <OUTPUT_MODE>
          Sets output formatting

          [env: HQ_OUTPUT_MODE=]
          [default: cli]
          [possible values: cli, json, quiet]

  --debug
          Enables more detailed log output

          [env: HQ_DEBUG=]
";
        let (doc, commands) = parse_clap(text);
        assert_debug_snapshot!(doc, @r#"
        CommandDoc {
            brief: RichText {
                parts: [
                    Text(
                        "Commands for the server",
                    ),
                ],
            },
            description: Some(
                RichText {
                    parts: [
                        Text(
                            "a multi line title",
                        ),
                    ],
                },
            ),
            usage: [
                Usage {
                    parts: [
                        Command(
                            "hq",
                        ),
                        Command(
                            "server",
                        ),
                        Option(
                            "OPTIONS",
                        ),
                        Argument(
                            "COMMAND",
                        ),
                    ],
                },
            ],
            arguments: [],
            option_categories: [
                CategoryDesc {
                    title: "Options",
                    options: [
                        OptionDesc {
                            short: Some(
                                "-h",
                            ),
                            long: "--help",
                            brief: RichText {
                                parts: [
                                    Text(
                                        "Print help (see a summary with '-h')",
                                    ),
                                ],
                            },
                            description: None,
                        },
                    ],
                },
                CategoryDesc {
                    title: "GLOBAL OPTIONS",
                    options: [
                        OptionDesc {
                            short: None,
                            long: "--server-dir <SERVER_DIR>",
                            brief: RichText {
                                parts: [
                                    Text(
                                        "The path where access files are stored",
                                    ),
                                ],
                            },
                            description: Some(
                                RichText {
                                    parts: [
                                        Config {
                                            key: "env",
                                            value: "HQ_SERVER_DIR=]",
                                        },
                                    ],
                                },
                            ),
                        },
                        OptionDesc {
                            short: None,
                            long: "--colors <COLORS>",
                            brief: RichText {
                                parts: [
                                    Text(
                                        "Sets console color policy",
                                    ),
                                ],
                            },
                            description: Some(
                                RichText {
                                    parts: [
                                        Config {
                                            key: "default",
                                            value: "auto]",
                                        },
                                        Text(
                                            "Possible values:",
                                        ),
                                        Ul(
                                            [
                                                "auto:   Use colors if the stdout is detected to be a terminal",
                                                "always: Always use colors",
                                                "never:  Never use colors",
                                            ],
                                        ),
                                    ],
                                },
                            ),
                        },
                        OptionDesc {
                            short: None,
                            long: "--output-mode <OUTPUT_MODE>",
                            brief: RichText {
                                parts: [
                                    Text(
                                        "Sets output formatting",
                                    ),
                                ],
                            },
                            description: Some(
                                RichText {
                                    parts: [
                                        Config {
                                            key: "env",
                                            value: "HQ_OUTPUT_MODE=]",
                                        },
                                        Config {
                                            key: "default",
                                            value: "cli]",
                                        },
                                        Config {
                                            key: "possible values",
                                            value: "cli, json, quiet]",
                                        },
                                    ],
                                },
                            ),
                        },
                        OptionDesc {
                            short: None,
                            long: "--debug",
                            brief: RichText {
                                parts: [
                                    Text(
                                        "Enables more detailed log output",
                                    ),
                                ],
                            },
                            description: Some(
                                RichText {
                                    parts: [
                                        Config {
                                            key: "env",
                                            value: "HQ_DEBUG=]",
                                        },
                                    ],
                                },
                            ),
                        },
                    ],
                },
            ],
        }
        "#);
        assert_debug_snapshot!(commands, @r#""#);
    }

    #[test]
    fn test_parse_clap_cargo_plain_args() {
        let text = "Create a new cargo package at <path>

Usage: cargo new [OPTIONS] <PATH>

Arguments:
  <PATH>
";
        let (doc, commands) = parse_clap(text);
        assert_debug_snapshot!(doc, @r#"
        CommandDoc {
            brief: RichText {
                parts: [
                    Text(
                        "Create a new cargo package at <path>",
                    ),
                ],
            },
            description: None,
            usage: [
                Usage {
                    parts: [
                        Command(
                            "cargo",
                        ),
                        Command(
                            "new",
                        ),
                        Option(
                            "OPTIONS",
                        ),
                        Argument(
                            "PATH",
                        ),
                    ],
                },
            ],
            arguments: [
                ArgumentDesc {
                    name: "<PATH>",
                    brief: RichText {
                        parts: [],
                    },
                    description: None,
                },
            ],
            option_categories: [],
        }
        "#);
        assert!(commands.is_empty());
    }

    #[test]
    fn test_parse_clap_cargo_add_usage() {
        let text = "Add dependencies to a Cargo.toml manifest file

Usage: cargo add [OPTIONS] <DEP>[@<VERSION>] ...
       cargo add [OPTIONS] --path <PATH> ...
       cargo add [OPTIONS] --git <URL> ...
        ";

        let (doc, commands) = parse_clap(text);
        assert_debug_snapshot!(doc, @r#"
        CommandDoc {
            brief: RichText {
                parts: [
                    Text(
                        "Add dependencies to a Cargo.toml manifest file",
                    ),
                ],
            },
            description: None,
            usage: [
                Usage {
                    parts: [
                        Command(
                            "cargo",
                        ),
                        Command(
                            "add",
                        ),
                        Option(
                            "[OPTIONS]",
                        ),
                        Argument(
                            "<DEP>[@<VERSION>]",
                        ),
                        Command(
                            "...",
                        ),
                    ],
                },
                Usage {
                    parts: [
                        Command(
                            "cargo",
                        ),
                        Command(
                            "add",
                        ),
                        Option(
                            "[OPTIONS]",
                        ),
                        Command(
                            "--path",
                        ),
                        Argument(
                            "<PATH>",
                        ),
                        Command(
                            "...",
                        ),
                    ],
                },
                Usage {
                    parts: [
                        Command(
                            "cargo",
                        ),
                        Command(
                            "add",
                        ),
                        Option(
                            "[OPTIONS]",
                        ),
                        Command(
                            "--git",
                        ),
                        Argument(
                            "<URL>",
                        ),
                        Command(
                            "...",
                        ),
                    ],
                },
            ],
            arguments: [],
            option_categories: [],
        }
        "#);
        assert!(commands.is_empty());
    }
}
