//! The command line's conventions, which CONTRIBUTING.md's "Command-line
//! conventions" writes down, held over the `clap` definitions, as
//! `crates/ascribe-core/tests/names.rs` holds the element names (#965).
//!
//! Every command is in [`WRITES`] or [`READS`], so a new one says which it is.
//! A command that breaks a convention today is in [`EXCEPTIONS`], and the
//! test fails when an entry is no longer needed, so the list only shrinks: a
//! pull request that brings a command into line removes its entries.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fmt;
use std::path::PathBuf;

use clap::{Arg, Command, CommandFactory};

use crate::cli::Cli;

/// The commands that change a file you commit: one the "Commit it" column of
/// `docs/content/command/project-files.md` says yes to, or a page, or
/// `ascribe.toml`. Writing a build's outputs, a cache, or a history under
/// `.ascribe/`, or standard output, isn't writing.
const WRITES: &[&str] = &[
    "init",
    "new",
    "check",
    "fmt",
    "rename",
    "baseline write",
    "baseline prune",
    "versions release",
    "versions retire",
    "permalinks sync",
    "permalinks retire",
    "addresses init",
    "addresses move",
    "addresses retire",
    "sources fetch",
    "sources update",
    "observe",
    "templates eject",
    "vale eject",
    "agents sync",
    "agents index",
];

/// The commands that write none of the files [`WRITES`] means.
const READS: &[&str] = &[
    "build",
    "preview",
    "diff",
    "drift",
    "report",
    "explain",
    "model",
    "outline",
    "link",
    "refs",
    "search",
    "resolve",
    "versions list",
    "versions changes",
    "versions export",
    "permalinks list",
    "permalinks verify",
    "addresses list",
    "sources status",
    "templates list",
    "templates status",
    "agents rules",
    "agents skill",
    "agents prompt",
    "agents hook",
    "mcp",
    "lsp",
];

/// The commands that take no `--format json`, for good: they only print a
/// file (`agents rules`, `agents skill`), or what they print is another
/// program's protocol (the servers, and the hook's answer to an agent).
const NO_JSON: &[&str] = &[
    "preview",
    "agents rules",
    "agents skill",
    "agents hook",
    "mcp",
    "lsp",
];

/// The commands that compare the project with a git revision, which they
/// take as `--base <REV>`.
const COMPARES: &[&str] = &["check", "diff", "drift", "addresses move"];

/// Options that take a revision and compare nothing, so aren't `--base`:
/// `sources update --to` is the commit to move a pin to.
const OTHER_REVISIONS: &[(&str, &str)] = &[("sources update", "to")];

/// The values every `--format` draws from (`project-docs/checklists.md`, "A
/// new command or option").
const FORMATS: &[&str] = &["text", "concise", "json", "prompt", "html", "markdown"];

/// The conventions, each one a rule [`check`] holds a command to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Rule {
    /// A command that writes takes `--dry-run`.
    DryRun,
    /// A command that writes takes `--force`, past the git guard.
    Force,
    /// `--check` is a dry run, so only a command that writes takes it.
    CheckWrites,
    /// `--deny-warnings` is an alias of `--exit-code=warning`, so a command
    /// takes it only beside `--exit-code`.
    ExitCode,
    /// Every command takes `--format json`, apart from [`NO_JSON`].
    Json,
    /// A `--format` value is one of [`FORMATS`], and a command that takes
    /// `text` takes `concise` too, as its own form or an alias of `text`.
    FormatValues,
    /// A command that compares with a revision takes it as `--base <REV>`,
    /// and no other option or argument takes one.
    Base,
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Rule::DryRun => "every command that writes takes --dry-run",
            Rule::Force => "every command that writes takes --force",
            Rule::CheckWrites => "only a command that writes takes --check",
            Rule::ExitCode => "--deny-warnings goes with --exit-code",
            Rule::Json => "every command takes --format json",
            Rule::FormatValues => "--format takes the shared values, with concise beside text",
            Rule::Base => "a revision to compare with is --base <REV>",
        })
    }
}

/// Today's exceptions: each rule, with the commands that don't follow it
/// yet. The pull requests that follow #965 remove them, a group of commands
/// at a time; a renamed option keeps its old spelling as a hidden alias for
/// a release.
const EXCEPTIONS: &[(Rule, &[&str])] = &[
    (
        Rule::DryRun,
        &[
            "init",
            "check",
            "fmt",
            "rename",
            "baseline write",
            "baseline prune",
            "permalinks sync",
            "permalinks retire",
            "addresses init",
            "addresses move",
            "addresses retire",
            "sources fetch",
            "sources update",
            "observe",
            "templates eject",
            "vale eject",
            "agents sync",
            "agents index",
        ],
    ),
    (
        Rule::Force,
        &[
            "init",
            "check",
            "fmt",
            "rename",
            "baseline write",
            "baseline prune",
            "permalinks sync",
            "permalinks retire",
            "addresses init",
            "addresses move",
            "addresses retire",
            "sources fetch",
            "sources update",
            "observe",
            "templates eject",
            "vale eject",
            "agents sync",
            "agents index",
        ],
    ),
    (Rule::ExitCode, &["check"]),
    (
        Rule::Json,
        &[
            "sources fetch",
            "templates list",
            "vale eject",
            "agents sync",
            "agents prompt",
        ],
    ),
    (
        Rule::FormatValues,
        &[
            "init",
            "fmt",
            "build",
            "diff",
            "drift",
            "report",
            "explain",
            "model",
            "outline",
            "link",
            "refs",
            "search",
            "resolve",
            "rename",
            "baseline write",
            "baseline prune",
            "versions list",
            "versions changes",
            "versions release",
            "versions export",
            "versions retire",
            "permalinks sync",
            "permalinks list",
            "permalinks verify",
            "permalinks retire",
            "addresses init",
            "addresses move",
            "addresses retire",
            "addresses list",
            "sources update",
            "sources status",
            "observe",
            "templates eject",
            "templates status",
            "agents index",
        ],
    ),
];

fn excepted(name: &str) -> BTreeSet<Rule> {
    EXCEPTIONS
        .iter()
        .filter(|(_, names)| names.contains(&name))
        .map(|(rule, _)| *rule)
        .collect()
}

/// Every command that does its own work, by its full name (`versions
/// release`), with its definition: a group's subcommands, not the group. A
/// hidden command, which `--help` doesn't list, isn't one.
fn leaves() -> Vec<(String, Command)> {
    fn walk(prefix: &str, command: &Command, out: &mut Vec<(String, Command)>) {
        for sub in command.get_subcommands().filter(|sub| !sub.is_hide_set()) {
            let name = if prefix.is_empty() {
                sub.get_name().to_string()
            } else {
                format!("{prefix} {}", sub.get_name())
            };
            if sub.has_subcommands() {
                walk(&name, sub, out);
            } else {
                out.push((name, sub.clone()));
            }
        }
    }
    let mut out = Vec::new();
    walk("", &Cli::command(), &mut out);
    out
}

fn long<'a>(command: &'a Command, name: &str) -> Option<&'a Arg> {
    command
        .get_arguments()
        .find(|arg| arg.get_long() == Some(name))
}

fn takes_revision(arg: &Arg) -> bool {
    arg.get_value_names()
        .is_some_and(|names| names.iter().any(|name| name.as_str() == "REV"))
}

/// The rules `command`, named `name`, breaks.
fn check(name: &str, command: &Command) -> BTreeSet<Rule> {
    let mut broken = BTreeSet::new();
    let writes = WRITES.contains(&name);
    if writes && long(command, "dry-run").is_none() {
        broken.insert(Rule::DryRun);
    }
    if writes && long(command, "force").is_none() {
        broken.insert(Rule::Force);
    }
    if !writes && long(command, "check").is_some() {
        broken.insert(Rule::CheckWrites);
    }
    if long(command, "deny-warnings").is_some() && long(command, "exit-code").is_none() {
        broken.insert(Rule::ExitCode);
    }
    let format = long(command, "format");
    let values: BTreeSet<String> = format
        .map(|arg| {
            arg.get_possible_values()
                .iter()
                .flat_map(|value| {
                    value
                        .get_name_and_aliases()
                        .map(str::to_string)
                        .collect::<Vec<_>>()
                })
                .collect()
        })
        .unwrap_or_default();
    if !NO_JSON.contains(&name) && !values.contains("json") {
        broken.insert(Rule::Json);
    }
    if values
        .iter()
        .any(|value| !FORMATS.contains(&value.as_str()))
        || (values.contains("text") && !values.contains("concise"))
    {
        broken.insert(Rule::FormatValues);
    }
    let base_ok = |arg: &Arg| {
        arg.get_long() == Some("base")
            || arg
                .get_long()
                .is_some_and(|option| OTHER_REVISIONS.contains(&(name, option)))
    };
    // A hidden argument takes a revision only to say `--base` takes it.
    if command
        .get_arguments()
        .any(|arg| takes_revision(arg) && !arg.is_hide_set() && !base_ok(arg))
        || (COMPARES.contains(&name)
            && long(command, "base").is_none_or(|arg| !takes_revision(arg)))
    {
        broken.insert(Rule::Base);
    }
    broken
}

/// Every command is in [`WRITES`] or [`READS`], once, and every name in the
/// lists above is a command.
#[test]
fn every_command_says_whether_it_writes() {
    let names: BTreeSet<String> = leaves().into_iter().map(|(name, _)| name).collect();
    for name in &names {
        let count = WRITES.iter().chain(READS).filter(|n| **n == name).count();
        assert_eq!(
            count, 1,
            "`ascribe {name}` must be in exactly one of WRITES and READS in crates/ascribe-cli/src/conventions.rs"
        );
    }
    let listed = WRITES
        .iter()
        .chain(READS)
        .chain(NO_JSON)
        .chain(COMPARES)
        .chain(OTHER_REVISIONS.iter().map(|(name, _)| name))
        .chain(EXCEPTIONS.iter().flat_map(|(_, names)| names.iter()));
    for name in listed {
        assert!(names.contains(*name), "`ascribe {name}` isn't a command");
    }
}

/// Every command follows the conventions, apart from [`EXCEPTIONS`], and
/// every exception is still needed.
#[test]
fn every_command_follows_the_conventions() {
    let mut problems = Vec::new();
    for (name, command) in leaves() {
        let broken = check(&name, &command);
        let excepted = excepted(&name);
        for rule in broken.difference(&excepted) {
            problems.push(format!(
                "`ascribe {name}` breaks \"{rule}\" (Rule::{rule:?})"
            ));
        }
        for rule in excepted.difference(&broken) {
            problems.push(format!(
                "`ascribe {name}` follows \"{rule}\" now: remove it from Rule::{rule:?}'s EXCEPTIONS"
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "the command-line conventions are in CONTRIBUTING.md; fix the command, \
         or, only for what it did before #965, add its exception:\n{}",
        problems.join("\n")
    );
}

/// The files Ascribe names in a project's folder follow the naming rule:
/// one you write is `ascribe.<name>.toml`, and a record Ascribe writes for
/// you to commit is `ascribe.<name>`, with no extension. The files are the
/// rows of `docs/content/command/project-files.md` with no folder in their
/// name, so a new one is held once that page lists it.
#[test]
fn project_files_follow_the_naming_rule() {
    /// Names another tool chooses.
    const OTHER_TOOLS: &[&str] = &["AGENTS.md", "CLAUDE.md", ".vale.ini", ".mcp.json"];
    /// Files named before the rule, which keep their names, since renaming
    /// one costs every project a migration: `ascribe.nav` is yours, and
    /// TOML, without `.toml`.
    const BEFORE: &[&str] = &["ascribe.nav"];

    let page = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/content/command/project-files.md");
    // Outside FileSystem: a page of this repository's docs, read as a test's
    // fixture, not as a project.
    let page = std::fs::read_to_string(page).unwrap();
    let mut seen = Vec::new();
    for row in page.lines().filter(|line| line.starts_with("| `")) {
        let mut cells = row.split(" | ");
        let (file, writer) = (cells.next().unwrap(), cells.next().unwrap());
        let names = file.split('`').skip(1).step_by(2);
        for name in names.filter(|name| !name.contains('/') && !name.starts_with('-')) {
            seen.push(name);
            if OTHER_TOOLS.contains(&name) || BEFORE.contains(&name) {
                continue;
            }
            let rest = name.strip_prefix("ascribe.");
            let ok = if writer.starts_with("You do") {
                name == ascribe_model::MODEL_FILE
                    || rest
                        .and_then(|rest| rest.strip_suffix(".toml"))
                        .is_some_and(|stem| !stem.is_empty() && !stem.contains('.'))
            } else {
                rest.is_some_and(|rest| !rest.is_empty() && !rest.contains('.'))
            };
            assert!(
                ok,
                "`{name}` breaks the naming rule for project files in CONTRIBUTING.md"
            );
        }
    }
    for name in OTHER_TOOLS.iter().chain(BEFORE) {
        assert!(
            seen.contains(name),
            "`{name}` isn't a file project-files.md lists"
        );
    }
}
