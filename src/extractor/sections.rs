use crate::text::RichText;
use anyhow::bail;
use std::borrow::Cow;
use std::ops::Deref;
// #[derive(Debug)]
// pub(crate) struct ParsedSubsection<'a> {
//     pub subsection_title: &'a str,
//     pub items: Vec<&'a str>,
// }
//
// #[derive(Debug)]
// pub(crate) struct ParsedSection<'a> {
//     pub section_title: &'a str,
//     pub subsections: Vec<ParsedSubsection<'a>>,
// }
//
// #[derive(Debug)]
// pub(crate) struct ParsedOutput<'a> {
//     pub main_title: &'a str,
//     pub sections: Vec<ParsedSection<'a>>,
// }

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq, Clone))]
pub(crate) struct Section<'a> {
    paragraph: Vec<&'a str>,
    subsections: Vec<Section<'a>>,
    indent: usize,
}

impl<'a> Section<'a> {
    pub fn first_line(&self) -> &'a str {
        self.paragraph.first().unwrap_or(&"")
    }

    pub fn paragraph(&self) -> String {
        self.paragraph.join("\n")
    }

    pub fn lines(&self) -> &[&'a str] {
        &self.paragraph
    }

    pub fn subsections(&self) -> &[Section<'a>] {
        &self.subsections
    }

    pub fn flatten_child_lines(&self) -> impl Iterator<Item = &'a str> {
        self.subsections
            .iter()
            .flat_map(|s| s.lines().iter().copied())
    }

    pub fn extract_sections_upto_ident<'b, 'c>(
        &'b mut self,
        indent: usize,
        mut out: &'c mut Vec<Section<'a>>,
    ) {
        self.subsections
            .extract_if(.., |s| s.indent < indent)
            .for_each(|mut s| {
                let i = s.indent;
                let idx = out.len();
                s.extract_sections_upto_ident(indent - i, out);
                out.insert(idx, s);
            });
    }

    pub fn common_subsection_indent(&self) -> usize {
        self.subsections.iter().map(|s| s.indent).min().unwrap_or(0)
    }

    pub fn as_rich_text_into(&self, out: &mut RichText) {
        out.add_lines(&self.paragraph);
        self.subsections_as_rich_text_into(out);
    }

    pub fn subsections_as_brief_and_full_description(&self) -> (RichText, Option<RichText>) {
        let mut brief = RichText::new();
        if self.subsections.is_empty() {
            return (brief, None);
        }
        self.subsections[0].as_rich_text_into(&mut brief);
        let mut description = RichText::new();
        for section in &self.subsections[1..] {
            description.add_lines(&section.paragraph);
            section.subsections_as_rich_text_into(&mut description);
        }
        (
            brief,
            if description.is_empty() {
                None
            } else {
                Some(description)
            },
        )
    }

    pub fn subsections_as_rich_text(&self) -> RichText {
        let mut out = RichText::new();
        self.subsections_as_rich_text_into(&mut out);
        out
    }

    fn subsections_as_rich_text_into(&self, out: &mut RichText) {
        for section in &self.subsections {
            out.add_lines(&section.paragraph);
            section.subsections_as_rich_text_into(out);
        }
    }

    // pub fn all(&self) -> String {
    // #    self.paragraph.join()
    // }
}

struct LineReader<'a> {
    lines: Vec<&'a str>,
    idx: usize,
}

impl<'a> LineReader<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lines: input.split("\n").collect(),
            idx: 0,
        }
    }

    pub fn current(&self) -> Option<&'a str> {
        self.lines.get(self.idx).map(|s| *s)
    }

    pub fn next(&mut self) {
        self.idx += 1;
    }

    pub fn skip_empty_lines(&mut self) {
        while let Some(line) = self.current() {
            if !line.trim().is_empty() {
                break;
            }
            self.next();
        }
    }

    pub fn read_paragraph(&mut self, indent: usize) -> Vec<&'a str> {
        let mut paragraph = Vec::new();
        while let Some(line) = self.current() {
            if line.trim().is_empty() {
                break;
            }
            if compute_indentation(line) != indent {
                break;
            }
            paragraph.push(&line[indent..]);
            self.next();
        }
        self.skip_empty_lines();
        paragraph
    }
}

fn parse_section(reader: &mut LineReader, section: &mut Section) {}

fn read_sections<'a, 'b>(
    reader: &'a mut LineReader<'b>,
    base_indent: usize,
    level: usize,
) -> Vec<Section<'b>> {
    let mut output = Vec::new();
    while let Some(line) = reader.current() {
        let indent = compute_indentation(line);
        if indent <= base_indent && level > 0 {
            break;
        }
        let paragraph = reader.read_paragraph(indent);
        let subsections = if reader
            .current()
            .map(|line| compute_indentation(line) > indent)
            .unwrap_or(false)
        {
            read_sections(reader, indent, level + 1)
        } else {
            Vec::new()
        };
        output.push(Section {
            paragraph,
            subsections,
            indent: indent - base_indent,
        });
    }
    output
}

pub(crate) fn parse_sections(input: &str) -> Vec<Section<'_>> {
    let mut reader = LineReader::new(input);
    reader.skip_empty_lines();
    read_sections(&mut reader, 0, 0)
}

fn strip_indent(text: &str) -> (usize, &str) {
    let indent = compute_indentation(text);
    (indent, &text[indent..])
}

fn compute_indentation(text: &str) -> usize {
    text.chars().take_while(|&c| c == ' ').count() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    pub fn test_parse_big() {
        let s = "Add dependencies to a Cargo.toml manifest file

Usage: cargo add [OPTIONS] <DEP>[@<VERSION>] ...
       cargo add [OPTIONS] --path <PATH> ...
       cargo add [OPTIONS] --git <URL> ...

Arguments:
  [DEP_ID]...
          Reference to a package to add as a dependency

          You can reference a package by:
          - `<name>`, like `cargo add serde` (latest version will be used)
          - `<name>@<version-req>`, like `cargo add serde@1` or `cargo add serde@=1.0.38`

Options:
      --no-default-features
          Disable the default features

      --default-features
          Re-enable the default features

  -F, --features <FEATURES>
          Space or comma separated list of features to activate

      --optional
          Mark the dependency as optional

          The package name will be exposed as feature of your crate.

      --no-optional
          Mark the dependency as required

          The package will be removed from your features.

      --public
          Mark the dependency as public (unstable)

          The dependency can be referenced in your library's public API.

      --no-public
          Mark the dependency as private (unstable)

          While you can use the crate in your implementation, it cannot be referenced in your public API.

      --rename <NAME>
          Rename the dependency

          Example uses:
          - Depending on multiple versions of a crate
          - Depend on crates with the same name from different registries

  -n, --dry-run
          Don't actually write the manifest

  -v, --verbose...
          Use verbose output (-vv very verbose/build.rs output)

  -q, --quiet
          Do not print cargo log messages

      --color <WHEN>
          Coloring

          [possible values: auto, always, never]

      --config <KEY=VALUE|PATH>
          Override a configuration value

  -Z <FLAG>
          Unstable (nightly-only) flags to Cargo, see 'cargo -Z help' for details

  -h, --help
          Print help (see a summary with '-h')

Manifest Options:
      --manifest-path <PATH>
          Path to Cargo.toml

      --lockfile-path <PATH>
          Path to Cargo.lock (unstable)

      --ignore-rust-version
          Ignore `rust-version` specification in packages

      --locked
          Assert that `Cargo.lock` will remain unchanged

      --offline
          Run without accessing the network

      --frozen
          Equivalent to specifying both --locked and --offline

Package Selection:
  -p, --package [<SPEC>]
          Package to modify

Source:
      --path <PATH>
          Filesystem path to local crate to add

      --base <BASE>
          The path base to use when adding from a local crate (unstable).

      --git <URI>
          Git repository location

          Without any other information, cargo will use latest commit on the main branch.

      --branch <BRANCH>
          Git branch to download the crate from

      --tag <TAG>
          Git tag to download the crate from

      --rev <REV>
          Git reference to download the crate from

          This is the catch all, handling hashes to named references in remote repositories.

      --registry <NAME>
          Package registry for this dependency

Section:
      --dev
          Add as development dependency

          Dev-dependencies are not used when compiling a package for building, but are used for compiling tests, examples, and benchmarks.

          These dependencies are not propagated to other packages which depend on this package.

      --build
          Add as build dependency

          Build-dependencies are the only dependencies available for use by build scripts (`build.rs` files).

      --target <TARGET>
          Add as dependency to the given target platform

Run `cargo help add` for more detailed information.
";
        assert_debug_snapshot!(parse_sections(s), @r#"
        [
            Section {
                paragraph: [
                    "Add dependencies to a Cargo.toml manifest file",
                ],
                subsections: [],
                indent: 0,
            },
            Section {
                paragraph: [
                    "Usage: cargo add [OPTIONS] <DEP>[@<VERSION>] ...",
                ],
                subsections: [
                    Section {
                        paragraph: [
                            "cargo add [OPTIONS] --path <PATH> ...",
                            "cargo add [OPTIONS] --git <URL> ...",
                        ],
                        subsections: [],
                        indent: 7,
                    },
                ],
                indent: 0,
            },
            Section {
                paragraph: [
                    "Arguments:",
                ],
                subsections: [
                    Section {
                        paragraph: [
                            "[DEP_ID]...",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Reference to a package to add as a dependency",
                                ],
                                subsections: [],
                                indent: 8,
                            },
                            Section {
                                paragraph: [
                                    "You can reference a package by:",
                                    "- `<name>`, like `cargo add serde` (latest version will be used)",
                                    "- `<name>@<version-req>`, like `cargo add serde@1` or `cargo add serde@=1.0.38`",
                                ],
                                subsections: [],
                                indent: 8,
                            },
                        ],
                        indent: 2,
                    },
                ],
                indent: 0,
            },
            Section {
                paragraph: [
                    "Options:",
                ],
                subsections: [
                    Section {
                        paragraph: [
                            "--no-default-features",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Disable the default features",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                        ],
                        indent: 6,
                    },
                    Section {
                        paragraph: [
                            "--default-features",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Re-enable the default features",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                        ],
                        indent: 6,
                    },
                    Section {
                        paragraph: [
                            "-F, --features <FEATURES>",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Space or comma separated list of features to activate",
                                ],
                                subsections: [],
                                indent: 8,
                            },
                            Section {
                                paragraph: [
                                    "--optional",
                                ],
                                subsections: [
                                    Section {
                                        paragraph: [
                                            "Mark the dependency as optional",
                                        ],
                                        subsections: [],
                                        indent: 4,
                                    },
                                    Section {
                                        paragraph: [
                                            "The package name will be exposed as feature of your crate.",
                                        ],
                                        subsections: [],
                                        indent: 4,
                                    },
                                ],
                                indent: 4,
                            },
                            Section {
                                paragraph: [
                                    "--no-optional",
                                ],
                                subsections: [
                                    Section {
                                        paragraph: [
                                            "Mark the dependency as required",
                                        ],
                                        subsections: [],
                                        indent: 4,
                                    },
                                    Section {
                                        paragraph: [
                                            "The package will be removed from your features.",
                                        ],
                                        subsections: [],
                                        indent: 4,
                                    },
                                ],
                                indent: 4,
                            },
                            Section {
                                paragraph: [
                                    "--public",
                                ],
                                subsections: [
                                    Section {
                                        paragraph: [
                                            "Mark the dependency as public (unstable)",
                                        ],
                                        subsections: [],
                                        indent: 4,
                                    },
                                    Section {
                                        paragraph: [
                                            "The dependency can be referenced in your library's public API.",
                                        ],
                                        subsections: [],
                                        indent: 4,
                                    },
                                ],
                                indent: 4,
                            },
                            Section {
                                paragraph: [
                                    "--no-public",
                                ],
                                subsections: [
                                    Section {
                                        paragraph: [
                                            "Mark the dependency as private (unstable)",
                                        ],
                                        subsections: [],
                                        indent: 4,
                                    },
                                    Section {
                                        paragraph: [
                                            "While you can use the crate in your implementation, it cannot be referenced in your public API.",
                                        ],
                                        subsections: [],
                                        indent: 4,
                                    },
                                ],
                                indent: 4,
                            },
                            Section {
                                paragraph: [
                                    "--rename <NAME>",
                                ],
                                subsections: [
                                    Section {
                                        paragraph: [
                                            "Rename the dependency",
                                        ],
                                        subsections: [],
                                        indent: 4,
                                    },
                                    Section {
                                        paragraph: [
                                            "Example uses:",
                                            "- Depending on multiple versions of a crate",
                                            "- Depend on crates with the same name from different registries",
                                        ],
                                        subsections: [],
                                        indent: 4,
                                    },
                                ],
                                indent: 4,
                            },
                        ],
                        indent: 2,
                    },
                    Section {
                        paragraph: [
                            "-n, --dry-run",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Don't actually write the manifest",
                                ],
                                subsections: [],
                                indent: 8,
                            },
                        ],
                        indent: 2,
                    },
                    Section {
                        paragraph: [
                            "-v, --verbose...",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Use verbose output (-vv very verbose/build.rs output)",
                                ],
                                subsections: [],
                                indent: 8,
                            },
                        ],
                        indent: 2,
                    },
                    Section {
                        paragraph: [
                            "-q, --quiet",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Do not print cargo log messages",
                                ],
                                subsections: [],
                                indent: 8,
                            },
                            Section {
                                paragraph: [
                                    "--color <WHEN>",
                                ],
                                subsections: [
                                    Section {
                                        paragraph: [
                                            "Coloring",
                                        ],
                                        subsections: [],
                                        indent: 4,
                                    },
                                    Section {
                                        paragraph: [
                                            "[possible values: auto, always, never]",
                                        ],
                                        subsections: [],
                                        indent: 4,
                                    },
                                ],
                                indent: 4,
                            },
                            Section {
                                paragraph: [
                                    "--config <KEY=VALUE|PATH>",
                                ],
                                subsections: [
                                    Section {
                                        paragraph: [
                                            "Override a configuration value",
                                        ],
                                        subsections: [],
                                        indent: 4,
                                    },
                                ],
                                indent: 4,
                            },
                        ],
                        indent: 2,
                    },
                    Section {
                        paragraph: [
                            "-Z <FLAG>",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Unstable (nightly-only) flags to Cargo, see 'cargo -Z help' for details",
                                ],
                                subsections: [],
                                indent: 8,
                            },
                        ],
                        indent: 2,
                    },
                    Section {
                        paragraph: [
                            "-h, --help",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Print help (see a summary with '-h')",
                                ],
                                subsections: [],
                                indent: 8,
                            },
                        ],
                        indent: 2,
                    },
                ],
                indent: 0,
            },
            Section {
                paragraph: [
                    "Manifest Options:",
                ],
                subsections: [
                    Section {
                        paragraph: [
                            "--manifest-path <PATH>",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Path to Cargo.toml",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                        ],
                        indent: 6,
                    },
                    Section {
                        paragraph: [
                            "--lockfile-path <PATH>",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Path to Cargo.lock (unstable)",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                        ],
                        indent: 6,
                    },
                    Section {
                        paragraph: [
                            "--ignore-rust-version",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Ignore `rust-version` specification in packages",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                        ],
                        indent: 6,
                    },
                    Section {
                        paragraph: [
                            "--locked",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Assert that `Cargo.lock` will remain unchanged",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                        ],
                        indent: 6,
                    },
                    Section {
                        paragraph: [
                            "--offline",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Run without accessing the network",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                        ],
                        indent: 6,
                    },
                    Section {
                        paragraph: [
                            "--frozen",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Equivalent to specifying both --locked and --offline",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                        ],
                        indent: 6,
                    },
                ],
                indent: 0,
            },
            Section {
                paragraph: [
                    "Package Selection:",
                ],
                subsections: [
                    Section {
                        paragraph: [
                            "-p, --package [<SPEC>]",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Package to modify",
                                ],
                                subsections: [],
                                indent: 8,
                            },
                        ],
                        indent: 2,
                    },
                ],
                indent: 0,
            },
            Section {
                paragraph: [
                    "Source:",
                ],
                subsections: [
                    Section {
                        paragraph: [
                            "--path <PATH>",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Filesystem path to local crate to add",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                        ],
                        indent: 6,
                    },
                    Section {
                        paragraph: [
                            "--base <BASE>",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "The path base to use when adding from a local crate (unstable).",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                        ],
                        indent: 6,
                    },
                    Section {
                        paragraph: [
                            "--git <URI>",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Git repository location",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                            Section {
                                paragraph: [
                                    "Without any other information, cargo will use latest commit on the main branch.",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                        ],
                        indent: 6,
                    },
                    Section {
                        paragraph: [
                            "--branch <BRANCH>",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Git branch to download the crate from",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                        ],
                        indent: 6,
                    },
                    Section {
                        paragraph: [
                            "--tag <TAG>",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Git tag to download the crate from",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                        ],
                        indent: 6,
                    },
                    Section {
                        paragraph: [
                            "--rev <REV>",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Git reference to download the crate from",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                            Section {
                                paragraph: [
                                    "This is the catch all, handling hashes to named references in remote repositories.",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                        ],
                        indent: 6,
                    },
                    Section {
                        paragraph: [
                            "--registry <NAME>",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Package registry for this dependency",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                        ],
                        indent: 6,
                    },
                ],
                indent: 0,
            },
            Section {
                paragraph: [
                    "Section:",
                ],
                subsections: [
                    Section {
                        paragraph: [
                            "--dev",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Add as development dependency",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                            Section {
                                paragraph: [
                                    "Dev-dependencies are not used when compiling a package for building, but are used for compiling tests, examples, and benchmarks.",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                            Section {
                                paragraph: [
                                    "These dependencies are not propagated to other packages which depend on this package.",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                        ],
                        indent: 6,
                    },
                    Section {
                        paragraph: [
                            "--build",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Add as build dependency",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                            Section {
                                paragraph: [
                                    "Build-dependencies are the only dependencies available for use by build scripts (`build.rs` files).",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                        ],
                        indent: 6,
                    },
                    Section {
                        paragraph: [
                            "--target <TARGET>",
                        ],
                        subsections: [
                            Section {
                                paragraph: [
                                    "Add as dependency to the given target platform",
                                ],
                                subsections: [],
                                indent: 4,
                            },
                        ],
                        indent: 6,
                    },
                ],
                indent: 0,
            },
            Section {
                paragraph: [
                    "Run `cargo help add` for more detailed information.",
                ],
                subsections: [],
                indent: 0,
            },
        ]
        "#);
    }

    #[test]
    pub fn test_parse_output() {
        assert!(parse_sections("").is_empty());
        assert_eq!(
            parse_sections("abc"),
            vec![Section {
                paragraph: vec!["abc"],
                subsections: vec![],
                indent: 0,
            }]
        );
        assert_eq!(
            parse_sections("  abc"),
            vec![Section {
                paragraph: vec!["abc"],
                subsections: vec![],
                indent: 2,
            }]
        );
        let s = Section {
            paragraph: vec!["abc", "xyz"],
            subsections: vec![],
            indent: 0,
        };
        assert_eq!(parse_sections("abc\nxyz"), vec![s.clone()]);
        assert_eq!(parse_sections("\n\nabc\nxyz\n\n"), vec![s]);

        assert_eq!(
            parse_sections("abc\n\nxyz\n123"),
            vec![
                Section {
                    paragraph: vec!["abc"],
                    subsections: vec![],
                    indent: 0,
                },
                Section {
                    paragraph: vec!["xyz", "123"],
                    subsections: vec![],
                    indent: 0,
                }
            ]
        );

        assert_eq!(
            parse_sections("abc\n  xyz\n   123\n   321\n\n  e\nf"),
            vec![
                Section {
                    paragraph: vec!["abc"],
                    subsections: vec![
                        Section {
                            paragraph: vec!["xyz"],
                            subsections: vec![Section {
                                paragraph: vec!["123", "321"],
                                subsections: vec![],
                                indent: 1,
                            }],
                            indent: 2,
                        },
                        Section {
                            paragraph: vec!["e"],
                            subsections: vec![],
                            indent: 2,
                        }
                    ],
                    indent: 0,
                },
                Section {
                    paragraph: vec!["f"],
                    subsections: vec![],
                    indent: 0,
                }
            ]
        );

        // assert_eq!(
        //     parse_sections("xxx\n  -C <DIRECTORY>\n          Change\n      --locked"),
        //     vec![
        //         Section {
        //             paragraph: vec!["first"],
        //             subsections: vec![Section {
        //                 paragraph: vec!["second"],
        //                 subsections: vec![Section {
        //                     paragraph: vec!["third"],
        //                     subsections: vec![],
        //                     indent: 1,
        //                 }],
        //                 indent: 1,
        //             }, ],
        //             indent: 0,
        //         },
        //     ])
    }
}
