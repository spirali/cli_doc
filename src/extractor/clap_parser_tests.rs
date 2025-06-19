use crate::commands::{CommandDoc, CommandOuterDoc};
use crate::extractor::clap_parser::ClapParser;
use crate::extractor::sections::parse_sections;
use insta::assert_debug_snapshot;

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
        brief: RichText {
            parts: [
                Text(
                    "Simple program to greet a person",
                ),
            ],
        },
        description: None,
        usage: [
            Usage {
                parts: [
                    Command(
                        "test-minimal",
                    ),
                    Command(
                        "--name",
                    ),
                    Argument(
                        "<NAME>",
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
                            "-n",
                        ),
                        long: "--name <NAME>",
                        brief: RichText {
                            parts: [
                                Text(
                                    "The name of the person to greet",
                                ),
                            ],
                        },
                        description: None,
                    },
                    OptionDesc {
                        short: Some(
                            "-h",
                        ),
                        long: "--help",
                        brief: RichText {
                            parts: [
                                Text(
                                    "Print help",
                                ),
                            ],
                        },
                        description: None,
                    },
                    OptionDesc {
                        short: Some(
                            "-V",
                        ),
                        long: "--version",
                        brief: RichText {
                            parts: [
                                Text(
                                    "Print version",
                                ),
                            ],
                        },
                        description: None,
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
        brief: RichText {
            parts: [
                Text(
                    "Submit a new job",
                ),
            ],
        },
        description: None,
        usage: [
            Usage {
                parts: [
                    Command(
                        "hq",
                    ),
                    Command(
                        "submit",
                    ),
                    Option(
                        "[OPTIONS]",
                    ),
                    Argument(
                        "<COMMANDS>...",
                    ),
                ],
            },
        ],
        arguments: [
            ArgumentDesc {
                name: "<COMMANDS>...",
                brief: RichText {
                    parts: [
                        Text(
                            "Command that should be executed by each task",
                        ),
                    ],
                },
                description: None,
            },
        ],
        option_categories: [
            CategoryDesc {
                title: "Options",
                options: [
                    OptionDesc {
                        short: None,
                        long: "--name <NAME>",
                        brief: RichText {
                            parts: [
                                Text(
                                    "The name of the job",
                                ),
                            ],
                        },
                        description: None,
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
        brief: RichText {
            parts: [
                Text(
                    "Rust's package manager",
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
                    Option(
                        "[+toolchain]",
                    ),
                    Option(
                        "[OPTIONS]",
                    ),
                    Option(
                        "[COMMAND]",
                    ),
                ],
            },
            Usage {
                parts: [
                    Command(
                        "cargo",
                    ),
                    Option(
                        "[+toolchain]",
                    ),
                    Option(
                        "[OPTIONS]",
                    ),
                    Command(
                        "-Zscript",
                    ),
                    Argument(
                        "<MANIFEST_RS>",
                    ),
                    Option(
                        "[ARGS]...",
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
                            "-V",
                        ),
                        long: "--version",
                        brief: RichText {
                            parts: [
                                Text(
                                    "Print version info and exit",
                                ),
                            ],
                        },
                        description: None,
                    },
                    OptionDesc {
                        short: None,
                        long: "--list",
                        brief: RichText {
                            parts: [
                                Text(
                                    "List installed commands",
                                ),
                            ],
                        },
                        description: None,
                    },
                    OptionDesc {
                        short: None,
                        long: "--explain <CODE>",
                        brief: RichText {
                            parts: [
                                Text(
                                    "Provide a detailed explanation of a rustc error message",
                                ),
                            ],
                        },
                        description: None,
                    },
                    OptionDesc {
                        short: Some(
                            "-v",
                        ),
                        long: "--verbose...",
                        brief: RichText {
                            parts: [
                                Text(
                                    "Use verbose output (-vv very verbose/build.rs output)",
                                ),
                            ],
                        },
                        description: None,
                    },
                    OptionDesc {
                        short: Some(
                            "-q",
                        ),
                        long: "--quiet",
                        brief: RichText {
                            parts: [
                                Text(
                                    "Do not print cargo log messages",
                                ),
                            ],
                        },
                        description: None,
                    },
                    OptionDesc {
                        short: None,
                        long: "--color <WHEN>",
                        brief: RichText {
                            parts: [
                                Text(
                                    "Coloring [possible values: auto, always, never]",
                                ),
                            ],
                        },
                        description: None,
                    },
                    OptionDesc {
                        short: None,
                        long: "-C <DIRECTORY>",
                        brief: RichText {
                            parts: [
                                Text(
                                    "Change to DIRECTORY before doing anything (nightly-only)",
                                ),
                            ],
                        },
                        description: None,
                    },
                    OptionDesc {
                        short: None,
                        long: "--locked",
                        brief: RichText {
                            parts: [
                                Text(
                                    "Assert that `Cargo.lock` will remain unchanged",
                                ),
                            ],
                        },
                        description: None,
                    },
                    OptionDesc {
                        short: None,
                        long: "--offline",
                        brief: RichText {
                            parts: [
                                Text(
                                    "Run without accessing the network",
                                ),
                            ],
                        },
                        description: None,
                    },
                    OptionDesc {
                        short: None,
                        long: "--frozen",
                        brief: RichText {
                            parts: [
                                Text(
                                    "Equivalent to specifying both --locked and --offline",
                                ),
                            ],
                        },
                        description: None,
                    },
                    OptionDesc {
                        short: None,
                        long: "--config <KEY=VALUE|PATH>",
                        brief: RichText {
                            parts: [
                                Text(
                                    "Override a configuration value",
                                ),
                            ],
                        },
                        description: None,
                    },
                    OptionDesc {
                        short: None,
                        long: "-Z <FLAG>",
                        brief: RichText {
                            parts: [
                                Text(
                                    "Unstable (nightly-only) flags to Cargo, see 'cargo -Z help' for details",
                                ),
                            ],
                        },
                        description: None,
                    },
                    OptionDesc {
                        short: Some(
                            "-h",
                        ),
                        long: "--help",
                        brief: RichText {
                            parts: [
                                Text(
                                    "Print help",
                                ),
                            ],
                        },
                        description: None,
                    },
                ],
            },
        ],
    }
    "#);
    assert_debug_snapshot!(commands, @r###"
    [
        CommandOuterDoc {
            name: "build",
        },
        CommandOuterDoc {
            name: "check",
        },
        CommandOuterDoc {
            name: "clean",
        },
        CommandOuterDoc {
            name: "doc",
        },
        CommandOuterDoc {
            name: "new",
        },
        CommandOuterDoc {
            name: "init",
        },
        CommandOuterDoc {
            name: "add",
        },
        CommandOuterDoc {
            name: "remove",
        },
        CommandOuterDoc {
            name: "run",
        },
        CommandOuterDoc {
            name: "test",
        },
        CommandOuterDoc {
            name: "bench",
        },
        CommandOuterDoc {
            name: "update",
        },
        CommandOuterDoc {
            name: "search",
        },
        CommandOuterDoc {
            name: "publish",
        },
        CommandOuterDoc {
            name: "install",
        },
        CommandOuterDoc {
            name: "uninstall",
        },
    ]
    "###);
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
                        "[OPTIONS]",
                    ),
                    Argument(
                        "<COMMAND>",
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
                                        value: "HQ_SERVER_DIR=",
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
                                        value: "auto",
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
                                        value: "HQ_OUTPUT_MODE=",
                                    },
                                    Config {
                                        key: "default",
                                        value: "cli",
                                    },
                                    Config {
                                        key: "possible values",
                                        value: "cli, json, quiet",
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
                                        value: "HQ_DEBUG=",
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
    assert_debug_snapshot!(commands, @r###"
    [
        CommandOuterDoc {
            name: "start",
        },
        CommandOuterDoc {
            name: "stop",
        },
        CommandOuterDoc {
            name: "info",
        },
        CommandOuterDoc {
            name: "generate-access",
        },
    ]
    "###);
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
                        "[OPTIONS]",
                    ),
                    Argument(
                        "<PATH>",
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
