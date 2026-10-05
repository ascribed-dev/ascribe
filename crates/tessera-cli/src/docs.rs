//! The command reference, `docs/content/reference/cli.md`, includes fragments
//! generated from the `clap` definitions: the synopsis, the options every
//! command accepts, and each command's own, in
//! `docs/content/_generated/cli-*.md`. This test renders them and fails when
//! one is out of date; run it with `ASCRIBE_BLESS=1` to rewrite them. To
//! change what a fragment says, change the help text.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fmt::Write as _;
use std::path::PathBuf;

use clap::{Arg, ArgAction, Command, CommandFactory};

use crate::cli::Cli;

const BLESS: &str = "ASCRIBE_BLESS=1 cargo test -p tessera-cli docs";

/// What each fragment starts with: what generates it, and how.
const HEADER: &str = "<!-- Generated from the help text in crates/tessera-cli/src/ by \
     crates/tessera-cli/src/docs.rs. Edit the help text, then run \
     `ASCRIBE_BLESS=1 cargo test -p tessera-cli docs`. -->\n";

/// Options that no release has yet, as `(command, option)`: the reference
/// marks them `@available: next`. Remove an entry when the option's command
/// section is itself marked, or when `next` stops meaning "unreleased" and
/// the release that shipped the option is named instead.
const UNRELEASED: &[(&str, &str)] = &[("build", "anchors")];

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The address help links to is the docs site's, `[consumer] site` in
/// `docs/ascribe.toml`, and each page it links to is a page of the docs.
#[test]
fn help_links_to_the_docs_site() {
    let model = tessera_model::load(repo().join("docs/ascribe.toml")).unwrap();
    assert_eq!(
        model
            .consumer
            .site
            .as_deref()
            .map(|s| s.trim_end_matches('/')),
        Some(crate::cli::DOCS_SITE),
        "the docs site's address in crates/tessera-cli/src/cli.rs isn't [consumer] site in docs/ascribe.toml"
    );

    let page = std::fs::read_to_string(repo().join("docs/content/reference/cli.md")).unwrap();
    let mut cli = Cli::command();
    let mut helps = vec![cli.render_long_help().to_string()];
    // Rendering builds the command, which adds clap's own `help` subcommand.
    for command in cli.get_subcommands_mut().filter(|c| c.get_name() != "help") {
        helps.push(command.render_long_help().to_string());
        for sub in command
            .get_subcommands_mut()
            .filter(|c| c.get_name() != "help")
        {
            helps.push(sub.render_long_help().to_string());
        }
    }
    for help in helps {
        let link = help
            .lines()
            .find_map(|line| line.strip_prefix("Documentation: "))
            .expect("each command's help ends with a link to its documentation");
        let path = link.strip_prefix(crate::cli::DOCS_SITE).unwrap();
        let (route, anchor) = path.split_once('#').unwrap_or((path, ""));
        assert_eq!(route, "/reference/cli/", "{link}");
        if !anchor.is_empty() {
            // `ascribe-sources-fetch` is the heading `ascribe sources fetch`,
            // a level below `ascribe sources`.
            let command = anchor.strip_prefix("ascribe-").unwrap().replace('-', " ");
            assert!(
                page.contains(&format!("\n## `ascribe {command}`\n"))
                    || page.contains(&format!("\n### `ascribe {command}`\n")),
                "{link}: reference/cli.md has no `ascribe {command}` heading"
            );
        }
    }
}

#[test]
fn the_command_reference_is_current() {
    let fragments = render();
    check_fragments("cli-", &fragments);
    let page = std::fs::read_to_string(repo().join("docs/content/reference/cli.md")).unwrap();
    for (name, _) in &fragments {
        assert!(
            page.contains(&format!("@include: ../_generated/{name}\n")),
            "docs/content/reference/cli.md doesn't include _generated/{name}"
        );
    }
}

/// Compares each fragment with its file in `docs/content/_generated/`, or,
/// with `ASCRIBE_BLESS`, writes it. A file there whose name starts with
/// `prefix` and that isn't one of `fragments` is left over from something
/// that no longer exists: it fails the test, and blessing removes it.
fn check_fragments(prefix: &str, fragments: &[(String, String)]) {
    let dir = repo().join("docs/content/_generated");
    let bless = std::env::var_os("ASCRIBE_BLESS").is_some();
    if bless {
        std::fs::create_dir_all(&dir).unwrap();
    }
    let mut stale = Vec::new();
    for entry in std::fs::read_dir(&dir).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        if name.starts_with(prefix) && !fragments.iter().any(|(n, _)| *n == name) {
            if bless {
                std::fs::remove_file(dir.join(&name)).unwrap();
            } else {
                stale.push(name);
            }
        }
    }
    for (name, text) in fragments {
        let path = dir.join(name);
        if bless {
            std::fs::write(&path, text).unwrap();
            continue;
        }
        let current = std::fs::read_to_string(&path).unwrap_or_default();
        if current != *text {
            stale.push(name.clone());
        }
    }
    assert!(
        stale.is_empty(),
        "out of date in docs/content/_generated/: {}. Run `{BLESS}`",
        stale.join(", ")
    );
}

/// The fragments, by file name: the synopsis, the options every command
/// accepts, and one for each command that has options of its own. A command
/// with subcommands (`sources`) is listed as each of them (`sources fetch`).
fn render() -> Vec<(String, String)> {
    let cli = Cli::command();
    let commands = leaves(&cli);
    let mut fragments = vec![("cli-synopsis.md".to_owned(), synopsis(&commands))];

    let global: Vec<&Arg> = cli.get_arguments().filter(|a| documented(a)).collect();
    fragments.push(("cli-global-options.md".to_owned(), options("", &global)));

    for (name, command) in commands {
        let args: Vec<&Arg> = command
            .get_arguments()
            .filter(|a| documented(a) && !a.is_global_set())
            .collect();
        if !args.is_empty() {
            let file = name.replace(' ', "-");
            fragments.push((format!("cli-{file}-options.md"), options(&name, &args)));
        }
    }
    fragments
}

/// Each command that runs, by its name after `ascribe`: the top-level
/// commands, and the subcommands of one that has them in its place.
fn leaves(cli: &Command) -> Vec<(String, &Command)> {
    let mut out = Vec::new();
    for command in cli.get_subcommands() {
        let subcommands: Vec<&Command> = command
            .get_subcommands()
            .filter(|c| c.get_name() != "help")
            .collect();
        if subcommands.is_empty() {
            out.push((command.get_name().to_owned(), command));
        }
        for sub in subcommands {
            out.push((format!("{} {}", command.get_name(), sub.get_name()), sub));
        }
    }
    out
}

/// Whether an argument is documented: every one but `--help` and `--version`,
/// which `clap` adds.
fn documented(arg: &Arg) -> bool {
    !arg.is_hide_set() && !matches!(arg.get_id().as_str(), "help" | "version")
}

/// The synopsis: a line for each command, with its options in order.
fn synopsis(commands: &[(String, &Command)]) -> String {
    let width = commands
        .iter()
        .map(|(name, _)| name.len())
        .max()
        .unwrap_or(0);
    let mut out = format!("{HEADER}\n```text\n");
    for (name, command) in commands {
        let usage: Vec<String> = command
            .get_arguments()
            .filter(|a| documented(a) && !a.is_global_set())
            .map(usage)
            .collect();
        let line = format!("ascribe {name:width$} {}", usage.join(" "));
        let _ = writeln!(out, "{}", line.trim_end());
    }
    out.push_str("ascribe --version\n```\n");
    out
}

/// How an argument is written in the synopsis: `[--build <NAME>]...`,
/// `[--format text|json]`, `[--check]`, `[PATHS]...`.
fn usage(arg: &Arg) -> String {
    let values: Vec<String> = arg
        .get_possible_values()
        .iter()
        .filter(|v| !v.is_hide_set())
        .map(|v| v.get_name().to_owned())
        .collect();
    let delimited = arg.get_value_delimiter().is_some();
    let value = if !values.is_empty() {
        values.join(if delimited { "," } else { "|" })
    } else {
        format!("<{}>", value_name(arg))
    };
    let repeated = matches!(arg.get_action(), ArgAction::Append) && !delimited;
    let written = match arg.get_long() {
        Some(long) if takes_value(arg) => format!("[--{long} {value}]"),
        Some(long) => format!("[--{long}]"),
        None => format!("[{}]", value_name(arg)),
    };
    if repeated {
        format!("{written}...")
    } else {
        written
    }
}

/// A list item for each argument: how it's written, then its help, its
/// values, and its default.
fn options(command: &str, args: &[&Arg]) -> String {
    let mut out = format!("{HEADER}\n");
    for arg in args {
        out.push_str("- ");
        let id = arg.get_id().as_str();
        if UNRELEASED.contains(&(command, id)) {
            out.push_str("@available: next\n  ");
        }
        let written = match arg.get_long() {
            Some(long) if takes_value(arg) => format!("--{long} <{}>", value_name(arg)),
            Some(long) => format!("--{long}"),
            None => usage(arg),
        };
        let help = arg
            .get_long_help()
            .or_else(|| arg.get_help())
            .map(ToString::to_string)
            .unwrap_or_default();
        // The help's first paragraph is what `-h` shows; the reference reads
        // them all as one.
        let help: Vec<String> = help.split("\n\n").map(sentence).collect();
        let _ = write!(out, "`{written}`: {}", escape(&help.join(" ")));

        let defaults: Vec<&str> = arg
            .get_default_values()
            .iter()
            .filter_map(|v| v.to_str())
            .collect();
        let values: Vec<_> = if takes_value(arg) {
            arg.get_possible_values()
                .into_iter()
                .filter(|v| !v.is_hide_set())
                .collect()
        } else {
            Vec::new()
        };
        // Every value is the default: say so once, rather than on each.
        let all = values.len() > 1 && values.iter().all(|v| defaults.contains(&v.get_name()));
        if all {
            out.push_str(" All of them by default.");
        } else if values.is_empty() && takes_value(arg) && !defaults.is_empty() {
            let _ = write!(out, " By default, `{}`.", defaults.join(","));
        }
        out.push('\n');
        for value in values {
            let name = value.get_name();
            let default = if defaults.contains(&name) && !all {
                " (the default)"
            } else {
                ""
            };
            let help = value
                .get_help()
                .map(ToString::to_string)
                .unwrap_or_default();
            let _ = writeln!(out, "  - `{name}`{default}: {}", escape(&sentence(&help)));
        }
    }
    out
}

fn takes_value(arg: &Arg) -> bool {
    matches!(arg.get_action(), ArgAction::Set | ArgAction::Append)
}

fn value_name(arg: &Arg) -> String {
    arg.get_value_names()
        .and_then(|names| names.first())
        .map_or_else(|| arg.get_id().as_str().to_uppercase(), ToString::to_string)
}

/// A paragraph of help text as a sentence: on one line, ending with a period,
/// which `clap` leaves off a summary.
fn sentence(text: &str) -> String {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.is_empty() || text.ends_with(['.', '!', '?', ':']) {
        text
    } else {
        format!("{text}.")
    }
}

/// Text with a backslash before each `{` and `<` outside code spans, so a
/// placeholder such as `{value}` or `<version>` reads as itself, and is never
/// taken for one of the docs' phrases or for HTML.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut code = false;
    for c in text.chars() {
        if c == '`' {
            code = !code;
        } else if matches!(c, '{' | '<') && !code {
            out.push('\\');
        }
        out.push(c);
    }
    out
}
