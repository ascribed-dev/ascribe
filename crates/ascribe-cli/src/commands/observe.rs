//! `ascribe observe`: runs the test of the `ascribe.examples.toml` in the
//! current folder, and when it passes, writes down what it passed and where:
//! an observation, `examples.observations/<name>.toml`, `default` without
//! `--name`, and says what changed since the one it replaces. With
//! `--check` it runs nothing, and says whether the observation still matches
//! the folder.
//!
//! This is the one place Ascribe runs a line of shell, and a test holds it
//! to that (`tests/all/program_starts.rs`). It runs only the commands of the
//! config in the folder it's run from: it doesn't look in a parent folder,
//! takes no path to a config, and never runs one a page, a source's copy, or
//! another project names. It says which file and which commands before it
//! runs them, and adds nothing to a command and substitutes nothing into it.
//! When standard input is a terminal, it asks before running them, and runs
//! nothing unless the answer is yes; `--yes` skips the question. Where
//! standard input isn't a terminal, as in CI, a hook, or an agent's shell,
//! nobody is there to answer, so it runs nothing unless given `--yes`.
//! The config, the observation, and the hash are `ascribe_model::examples`'s.
//!
//! Once it has written the observation, it looks for the nearest
//! `ascribe.toml` in the folder or one above it, and when that project's
//! sources don't read the config, because it's below a source's root, it
//! says so, with the warning `ascribe check` gives (`examples-config-unread`).
//! Nothing there is run.

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use ascribe_check::{Diagnostic, Project, apply_levels, unread_config_issue};
use ascribe_core::path::relative_path;
use ascribe_core::{FileId, LineIndex};
use ascribe_model::examples::{
    DEFAULT_OBSERVATION, EXAMPLES_FILE, ExamplesConfig, OBSERVATIONS_FOLDER, Observation, Problem,
    is_observation_name, kept_line, observation_path,
};
use ascribe_model::{COPIES_FOLDER, LOCK_FILE, Lock, example_hash};
use ascribe_resolve::{DiskFs, Layout, unread_config};
use clap::{Args as ClapArgs, ValueEnum};
use serde::Serialize;

use crate::cli::Global;
use crate::exit;

/// The version of `ascribe observe --format json`'s schema.
const SCHEMA_VERSION: u32 = 1;

/// Arguments of `ascribe observe`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// The observation to write: `examples.observations/<NAME>.toml`.
    ///
    /// Lowercase words of letters and digits joined by `-`, such as
    /// `node-22`, one for each environment the examples are tested in. Each
    /// observation is whole and separate, and running one doesn't touch
    /// another.
    #[arg(long, value_name = "NAME", default_value = DEFAULT_OBSERVATION)]
    pub name: String,

    /// Run nothing, and exit 1 when the observation is stale against the
    /// folder as it stands: for a pre-commit hook.
    ///
    /// It reads the config and the examples, and starts no program, the
    /// config's commands included.
    #[arg(long)]
    pub check: bool,

    /// Run the commands without asking first.
    ///
    /// It asks only when standard input is a terminal. Without one, as in CI,
    /// a hook, or an agent's shell, it lists the commands and runs nothing
    /// unless given `--yes`.
    #[arg(long)]
    pub yes: bool,

    /// How to show what it did.
    ///
    /// With `json`, what the commands print goes to standard error, so
    /// standard output holds the JSON alone.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// The output format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// What it runs and what it wrote, for people.
    Text,
    /// One JSON document, for tools.
    Json,
}

/// What `ascribe observe --format json` writes: the commands it ran, how the
/// test ended, the observation it wrote, and what changed since the one it
/// replaced. With `--check`, nothing ran: the observation it compared, and
/// whether it's current.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct ObserveReport {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    schema_version: u32,
    /// The version of Ascribe that wrote it.
    ascribe_version: &'static str,
    /// The config it ran, `ascribe.examples.toml`, in the current folder.
    config: String,
    /// Whether it was run with `--check`, which runs nothing.
    check: bool,
    /// The `[environment]` commands, in the order the config has them, each
    /// with the line it kept. Empty with `--check`.
    environment: Vec<EnvironmentRun>,
    /// The test command, and how it ended. Absent with `--check`.
    test: Option<TestRun>,
    /// The observation it wrote, from the current folder: none when the test
    /// failed. With `--check`, the one it compared.
    observation: Option<String>,
    /// With `--check`, whether the observation is current: it lists every
    /// example in the folder, with the hash it has now, and nothing else.
    /// Absent without `--check`.
    current: Option<bool>,
    /// What changed since the observation it replaced: absent when there was
    /// none, or the test failed. With `--check`, what differs between the
    /// observation and the folder.
    changes: Option<Changes>,
    /// The environment the observation records, by name: `os`, `arch`, and
    /// each entry's kept line. Empty when the test failed.
    recorded: BTreeMap<String, String>,
    /// The hash of each example, by path from the current folder. Empty when
    /// the test failed.
    files: BTreeMap<String, String>,
    /// What a page says it was tested with: `Node 24.9.0 on Linux`. Absent
    /// when the test failed.
    tested: Option<String>,
}

/// What differs between an observation and the one after it, or the folder.
#[derive(Debug, Default, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct Changes {
    /// Examples it lists that the earlier one didn't, by path.
    added: Vec<String>,
    /// Examples the earlier one listed that it doesn't.
    removed: Vec<String>,
    /// Examples both list, with different hashes.
    changed: Vec<String>,
    /// Environment values that differ, in the newer one's order, then those
    /// only the earlier one had.
    environment: Vec<EnvironmentChange>,
}

/// An environment value that differs.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct EnvironmentChange {
    /// The entry's key: `os`, `arch`, or the config's.
    key: String,
    /// What the earlier observation had; absent when it had none.
    was: Option<String>,
    /// What the newer one has; absent when it has none.
    now: Option<String>,
}

impl Changes {
    /// What differs from `was` to `now`.
    fn between(was: &Observation, now: &Observation) -> Changes {
        let mut changes = Changes::default();
        changes.files(&was.files, &now.files);
        for (key, value) in &now.environment {
            let before = was.value(key);
            if before != Some(value.as_str()) {
                changes.environment.push(EnvironmentChange {
                    key: key.clone(),
                    was: before.map(str::to_owned),
                    now: Some(value.clone()),
                });
            }
        }
        for (key, value) in &was.environment {
            if now.value(key).is_none() {
                changes.environment.push(EnvironmentChange {
                    key: key.clone(),
                    was: Some(value.clone()),
                    now: None,
                });
            }
        }
        changes
    }

    /// The files added, removed, and changed from `was` to `now`.
    fn files(&mut self, was: &BTreeMap<String, String>, now: &BTreeMap<String, String>) {
        for (path, hash) in now {
            match was.get(path) {
                None => self.added.push(path.clone()),
                Some(before) if before != hash => self.changed.push(path.clone()),
                Some(_) => {}
            }
        }
        self.removed
            .extend(was.keys().filter(|p| !now.contains_key(*p)).cloned());
    }

    fn is_empty(&self) -> bool {
        *self == Changes::default()
    }

    /// The changes as clauses of a sentence: `added a.ts`, `changed b.ts`,
    /// `node was v22.4.0, now v24.9.0`. Every path and value is escaped for
    /// a terminal.
    fn clauses(&self) -> Vec<String> {
        let paths = |list: &[String]| -> String {
            list.iter()
                .map(|p| terminal(p))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let mut out = Vec::new();
        for (word, list) in [
            ("added", &self.added),
            ("removed", &self.removed),
            ("changed", &self.changed),
        ] {
            if !list.is_empty() {
                out.push(format!("{word} {}", paths(list)));
            }
        }
        for change in &self.environment {
            let key = terminal(&change.key);
            out.push(match (&change.was, &change.now) {
                (Some(was), Some(now)) => {
                    format!("{key} was {}, now {}", terminal(was), terminal(now))
                }
                (None, Some(now)) => format!("{key} is new, {}", terminal(now)),
                (Some(was), None) => format!("{key} is gone, was {}", terminal(was)),
                (None, None) => key,
            });
        }
        out
    }
}

/// An `[environment]` command that ran.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct EnvironmentRun {
    /// The entry's key.
    key: String,
    /// The line of shell.
    command: String,
    /// The first line it printed, as the observation records it.
    kept: String,
}

/// The test command, and how it ended.
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct TestRun {
    /// The line of shell.
    command: String,
    /// Whether it exited with 0.
    passed: bool,
    /// Its exit code; absent when a signal stopped it.
    exit_code: Option<i32>,
}

/// Runs the command. Exit codes: 0 when the test passed and the observation
/// is written, 1 when the test failed and nothing was written, 2 when it
/// couldn't run: a name that isn't one, a folder that's a source's copies,
/// no config in the current folder, a config that can't be read, the
/// question answered with anything but yes, no terminal to ask at and no
/// `--yes`, no shell, an environment command
/// that failed or printed nothing, or an observation that can't be written. With `--check`: 0 when the observation is current,
/// 1 when it's stale or missing, 2 when the config, the examples, or the
/// observation can't be read.
pub fn run(_global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let stderr = io::stderr();
    let mut err = stderr.lock();
    // The question is asked only of a person at a terminal, and never with
    // `--check`, which runs nothing. Without a terminal or `--yes`, `observe`
    // runs nothing.
    let stdin = io::stdin();
    let mut answers = (!args.yes && !args.check && stdin.is_terminal()).then(|| stdin.lock());
    let ask = answers.as_mut().map(|a| a as &mut dyn BufRead);
    let code = match std::env::current_dir() {
        Ok(folder) => observe(&folder, &args, ask, &mut out, &mut err),
        Err(e) => {
            let _ = writeln!(err, "error: can't read the current folder: {e}");
            exit::FAILURE
        }
    };
    let _ = out.flush();
    exit::code(code)
}

/// Runs the config in `folder`, writing the report to `out` and what went
/// wrong to `err`, and returns the exit code. With `ask`, it asks on `err`
/// before running anything, and reads the answer from `ask`; without it, it
/// runs the commands only with `--yes`.
fn observe(
    folder: &Path,
    args: &Args,
    ask: Option<&mut dyn BufRead>,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    if !is_observation_name(&args.name) {
        let _ = writeln!(
            err,
            "error: `{}` isn't an observation's name: lowercase words of letters and digits joined by `-`, such as `node-22`",
            terminal(&args.name)
        );
        return exit::FAILURE;
    }
    if let Some(copy) = copies_of(folder) {
        let _ = writeln!(
            err,
            "error: this folder is {copy}, which `ascribe sources fetch` copied from another repository; `ascribe observe` never runs a config that arrived as a copy. Run it in that repository, and `ascribe sources update` brings the observation here"
        );
        return exit::FAILURE;
    }
    if args.check {
        return check(folder, args, out, err);
    }
    let (config, test) = match read_config(folder, err) {
        Ok((config, Some(test))) => (config, test),
        Ok((_, None)) => {
            let _ = writeln!(
                err,
                "error: {EXAMPLES_FILE} has no `test`, the line of shell that tests the examples"
            );
            return exit::FAILURE;
        }
        Err(code) => return code,
    };
    let path = observation_path(&args.name);
    // The observation this run replaces, when there's one this version can
    // read: what changed is said against it.
    let previous = read_observation(folder, &path).and_then(Result::ok);

    let json = args.format == Format::Json;
    if !consented(&config, &test, args, ask, out, err) {
        return exit::FAILURE;
    }

    let environment = match run_environment(&config, folder, err) {
        Ok(environment) => environment,
        Err(code) => return code,
    };

    // The test's output goes where it would if it were run by hand, as it
    // comes; with `--format json`, to standard error.
    let mut command = shell(&test, folder);
    if json {
        command.stdout(io::stderr());
    }
    let status = match command.stderr(Stdio::inherit()).status() {
        Ok(status) => status,
        Err(e) => return no_shell(err, &e),
    };
    let mut report = ObserveReport {
        schema_version: SCHEMA_VERSION,
        ascribe_version: env!("ASCRIBE_VERSION"),
        config: EXAMPLES_FILE.to_owned(),
        check: false,
        environment,
        test: Some(TestRun {
            command: test.clone(),
            passed: status.success(),
            exit_code: status.code(),
        }),
        observation: None,
        current: None,
        changes: None,
        recorded: BTreeMap::new(),
        files: BTreeMap::new(),
        tested: None,
    };
    if !status.success() {
        let _ = writeln!(
            err,
            "{} failed ({}); nothing was written, and the old observation is as it was.",
            terminal(&test),
            ended(status.code()),
        );
        if json {
            write_json(out, &report);
        }
        return exit::PROBLEMS;
    }

    let files = match examples(folder, &config) {
        Ok(files) => files,
        Err(e) => {
            let _ = writeln!(err, "error: can't read the examples: {e}");
            return exit::FAILURE;
        }
    };
    let mut recorded = vec![
        ("os".to_owned(), std::env::consts::OS.to_owned()),
        ("arch".to_owned(), arch().to_owned()),
    ];
    recorded.extend(
        report
            .environment
            .iter()
            .map(|e| (e.key.clone(), e.kept.clone())),
    );
    let observation = Observation {
        ascribe: env!("CARGO_PKG_VERSION").to_owned(),
        command: test.clone(),
        environment: recorded,
        files,
    };
    if let Err(e) = write_observation(folder, &args.name, &observation.to_toml()) {
        let _ = writeln!(err, "error: can't write {path}: {e}");
        return exit::FAILURE;
    }
    let tested = observation.tested_line(&config);
    let changes = previous.map(|was| Changes::between(&was, &observation));
    if json {
        report.observation = Some(path);
        report.changes = changes;
        report.recorded = observation.environment.into_iter().collect();
        report.files = observation.files;
        report.tested = Some(tested);
        write_json(out, &report);
    } else {
        let count = observation.files.len();
        let _ = writeln!(out, "{} passed.", terminal(&test));
        let _ = writeln!(
            out,
            "Wrote {path}: {count} {}, {}.",
            if count == 1 { "file" } else { "files" },
            terminal(&tested),
        );
        say_changes(out, changes);
    }
    if let Some(warning) = unread_warning(folder) {
        let _ = out.flush();
        let _ = writeln!(err, "{warning}");
    }
    exit::OK
}

/// Says what will run, before running anything, and whether it may: with
/// `ask`, the answer; without it, `--yes`. With `--format json`, the list
/// goes only to a person asked, or told why nothing ran, and on standard
/// error, so standard output holds the JSON alone.
fn consented(
    config: &ExamplesConfig,
    test: &str,
    args: &Args,
    ask: Option<&mut dyn BufRead>,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> bool {
    let json = args.format == Format::Json;
    match ask {
        Some(answers) => {
            let shown: &mut dyn Write = if json { &mut *err } else { &mut *out };
            list_commands(shown, "The commands in", config, test);
            let _ = shown.flush();
            if !agreed(answers, err) {
                let _ = writeln!(err, "Nothing ran, and nothing was written.");
                return false;
            }
            true
        }
        None if args.yes => {
            if !json {
                list_commands(out, "Running the commands in", config, test);
                let _ = out.flush();
            }
            true
        }
        None => {
            let shown: &mut dyn Write = if json { &mut *err } else { &mut *out };
            list_commands(shown, "The commands in", config, test);
            let _ = shown.flush();
            let _ = writeln!(
                err,
                "error: standard input isn't a terminal, so nobody was asked before running them. Read them, then run `ascribe observe` again with `--yes` to run them. Nothing ran, and nothing was written."
            );
            false
        }
    }
}

/// What changed since the observation a run replaces, when it replaced one.
fn say_changes(out: &mut dyn Write, changes: Option<Changes>) {
    match changes {
        None => {}
        Some(changes) if changes.is_empty() => {
            let _ = writeln!(out, "Nothing changed since the observation it replaces.");
        }
        Some(changes) => {
            let _ = writeln!(
                out,
                "Since the observation it replaces: {}.",
                changes.clauses().join("; ")
            );
        }
    }
}

/// The warning `ascribe check` gives the config in `folder` when the project
/// of the nearest `ascribe.toml`, in the folder or one above it, has it in a
/// source's folder but no source reads it: `None` when there's no such
/// project, it can't be loaded, a source reads the config, or `[checks]`
/// turns the check off.
fn unread_warning(folder: &Path) -> Option<String> {
    let config = Project::find_config(folder)?;
    let file = Project::load_model(&config).ok()?;
    // Outside FileSystem: the folder `ascribe observe` is run in, and the
    // project's, made comparable before the project's file system exists.
    let root = fs::canonicalize(&file.root).ok()?;
    let here = fs::canonicalize(folder).ok()?;
    let path = relative_path(&root, &here)?.join(EXAMPLES_FILE).ok()?;
    let layout = Layout::from_model(&file.model);
    let disk = DiskFs::new(&root, &layout);
    let unread = unread_config(&file.model, &disk, &path)?;
    let found = vec![Diagnostic::from_issue(&unread_config_issue(
        &unread,
        FileId::new(0),
    ))];
    let warning = apply_levels(&file.model.checks, found).into_iter().next()?;
    Some(format!(
        "{}: [{}] {}",
        warning.severity.as_str(),
        warning.code,
        terminal(&warning.message),
    ))
}

/// The config's commands, under `heading` and the config's name: the test,
/// then each `[environment]` command.
fn list_commands(out: &mut dyn Write, heading: &str, config: &ExamplesConfig, test: &str) {
    let _ = writeln!(out, "{heading} {EXAMPLES_FILE}:");
    let _ = writeln!(out, "  test         {}", terminal(test));
    for entry in &config.environment {
        let _ = writeln!(out, "  environment  {}", terminal(&entry.command));
    }
}

/// Asks on `err` whether to run the commands, and reads the answer from
/// `answers`: `y` or `yes`, in any case, agrees. Anything else, an empty
/// line, or no answer at all declines.
fn agreed(answers: &mut dyn BufRead, err: &mut dyn Write) -> bool {
    let _ = write!(err, "Run them? [y/N] ");
    let _ = err.flush();
    let mut line = String::new();
    if answers.read_line(&mut line).is_err() {
        return false;
    }
    matches!(line.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

/// `--check`: compares the observation named with the folder's examples as
/// they are, and starts no program.
fn check(folder: &Path, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let config = match read_config(folder, err) {
        Ok((config, _)) => config,
        Err(code) => return code,
    };
    let path = observation_path(&args.name);
    let previous = match read_observation(folder, &path) {
        Some(Ok(observation)) => Some(observation),
        Some(Err(problems)) => {
            for problem in problems {
                let _ = writeln!(err, "error: {path}: {}", terminal(&problem.detail));
            }
            return exit::FAILURE;
        }
        None => None,
    };
    let files = match examples(folder, &config) {
        Ok(files) => files,
        Err(e) => {
            let _ = writeln!(err, "error: can't read the examples: {e}");
            return exit::FAILURE;
        }
    };
    let mut changes = Changes::default();
    changes.files(
        &previous
            .as_ref()
            .map(|o| o.files.clone())
            .unwrap_or_default(),
        &files,
    );
    let current = previous.is_some() && changes.is_empty();
    let again = if args.name == DEFAULT_OBSERVATION {
        "`ascribe observe`".to_owned()
    } else {
        format!("`ascribe observe --name {}`", args.name)
    };
    if args.format == Format::Json {
        let report = ObserveReport {
            schema_version: SCHEMA_VERSION,
            ascribe_version: env!("ASCRIBE_VERSION"),
            config: EXAMPLES_FILE.to_owned(),
            check: true,
            environment: Vec::new(),
            test: None,
            observation: Some(path),
            current: Some(current),
            changes: previous.is_some().then_some(changes),
            recorded: BTreeMap::new(),
            files,
            tested: None,
        };
        write_json(out, &report);
    } else if previous.is_none() {
        let _ = writeln!(
            out,
            "There's no {path}, so no test run has covered the examples. Run {again} here to test them."
        );
    } else if current {
        let count = files.len();
        let _ = writeln!(
            out,
            "{path} is current: {count} {} as observed.",
            if count == 1 { "file" } else { "files" },
        );
    } else {
        let _ = writeln!(
            out,
            "{path} is stale: {}. Run {again} here to test the examples again.",
            changes.clauses().join("; ")
        );
    }
    if current { exit::OK } else { exit::PROBLEMS }
}

/// What `folder` is a copy of, when it's in a source's copies,
/// `sources/<name>/`, by an `ascribe.lock` in a folder above it: `the copies
/// of source `api``. A lock that can't be read makes everything in its
/// `sources/` a copy.
fn copies_of(folder: &Path) -> Option<String> {
    // Outside FileSystem: the folder `ascribe observe` is run in, and the
    // lock of a docs project it may be in, read only to refuse to run in its
    // copies.
    let folder = fs::canonicalize(folder).unwrap_or_else(|_| folder.to_path_buf());
    for root in folder.ancestors().skip(1) {
        // Outside FileSystem: as above.
        let Ok(text) = fs::read_to_string(root.join(LOCK_FILE)) else {
            continue;
        };
        let Ok(rest) = folder.strip_prefix(root.join(COPIES_FOLDER)) else {
            continue;
        };
        let Some(name) = rest.components().next() else {
            continue;
        };
        let name = name.as_os_str().to_string_lossy();
        let pinned = match Lock::parse(&text, FileId::new(0)) {
            Ok(lock) => lock.sources.iter().any(|s| s.name == name),
            Err(_) => true,
        };
        if pinned {
            return Some(format!(
                "{COPIES_FOLDER}/{}, the copies of source `{}`",
                terminal(&name),
                terminal(&name)
            ));
        }
    }
    None
}

/// The config in `folder`, and its `test`: or, when it can't be read, the
/// exit code, with what's wrong written to `err`.
fn read_config(folder: &Path, err: &mut dyn Write) -> Result<(ExamplesConfig, Option<String>), u8> {
    let config_path = folder.join(EXAMPLES_FILE);
    // Outside FileSystem: the config of the folder `ascribe observe` is run
    // in, which isn't a docs project.
    let text = match fs::read_to_string(&config_path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let _ = writeln!(
                err,
                "error: there's no {EXAMPLES_FILE} in this folder. `ascribe observe` runs the commands of the one in the folder it's run from"
            );
            return Err(exit::FAILURE);
        }
        Err(e) => {
            let _ = writeln!(err, "error: can't read {EXAMPLES_FILE}: {e}");
            return Err(exit::FAILURE);
        }
    };
    match ExamplesConfig::parse(&text) {
        Ok(config) => {
            let test = config.test.clone();
            Ok((config, test))
        }
        Err(problems) => {
            report_problems(err, &text, &problems);
            Err(exit::FAILURE)
        }
    }
}

/// The observation at `path` in `folder`, as read: `None` when there's none.
fn read_observation(folder: &Path, path: &str) -> Option<Result<Observation, Vec<Problem>>> {
    // Outside FileSystem: an observation in the folder `ascribe observe` is
    // run in, which isn't a docs project.
    let bytes = fs::read(folder.join(path)).ok()?;
    Some(Observation::parse(&String::from_utf8_lossy(&bytes)))
}

/// Runs each `[environment]` command, keeping the first line it prints: or,
/// when one fails or prints nothing, the exit code, with what's wrong
/// written to `err`.
fn run_environment(
    config: &ExamplesConfig,
    folder: &Path,
    err: &mut dyn Write,
) -> Result<Vec<EnvironmentRun>, u8> {
    let mut environment = Vec::new();
    for entry in &config.environment {
        let printed = match shell(&entry.command, folder)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output()
        {
            Ok(output) if output.status.success() => output.stdout,
            Ok(output) => {
                let _ = writeln!(
                    err,
                    "error: `{}` failed ({}), so the observation couldn't say where the test ran; nothing was written",
                    terminal(&entry.command),
                    ended(output.status.code()),
                );
                return Err(exit::FAILURE);
            }
            Err(e) => return Err(no_shell(err, &e)),
        };
        let Some(kept) = kept_line(&String::from_utf8_lossy(&printed)) else {
            let _ = writeln!(
                err,
                "error: `{}` printed nothing, so the observation couldn't say where the test ran; nothing was written",
                terminal(&entry.command)
            );
            return Err(exit::FAILURE);
        };
        environment.push(EnvironmentRun {
            key: entry.key.clone(),
            command: entry.command.clone(),
            kept,
        });
    }
    Ok(environment)
}

/// Each problem in the config, as `error: ascribe.examples.toml:LINE:COL: …`.
fn report_problems(err: &mut dyn Write, text: &str, problems: &[Problem]) {
    let index = LineIndex::new(text);
    for problem in problems {
        let at = index
            .line_col(problem.span.start())
            .map(|lc| format!(":{}:{}", lc.line + 1, lc.col + 1))
            .unwrap_or_default();
        let _ = writeln!(
            err,
            "error: {EXAMPLES_FILE}{at}: {}",
            terminal(&problem.detail)
        );
    }
}

fn no_shell(err: &mut dyn Write, e: &io::Error) -> u8 {
    let _ = writeln!(
        err,
        "error: can't start {}, which runs the config's commands: {e}",
        SHELL.0
    );
    exit::FAILURE
}

fn write_json(out: &mut dyn Write, report: &ObserveReport) {
    let _ = serde_json::to_writer_pretty(&mut *out, report)
        .map_err(io::Error::from)
        .and_then(|()| writeln!(out));
}

/// How a command ended, for a message.
fn ended(code: Option<i32>) -> String {
    match code {
        Some(code) => format!("exit code {code}"),
        None => "stopped by a signal".to_owned(),
    }
}

/// The platform's shell, and the argument before the line it runs.
#[cfg(not(windows))]
const SHELL: (&str, &str) = ("sh", "-c");
#[cfg(windows)]
const SHELL: (&str, &str) = ("cmd", "/C");

/// A line of shell, run as written by the platform's shell, in `folder`,
/// with the user's environment and standard input closed.
fn shell(line: &str, folder: &Path) -> Command {
    let mut command = Command::new(SHELL.0);
    command.arg(SHELL.1);
    #[cfg(windows)]
    {
        // `cmd` reads the rest of its command line as the line to run, so
        // it's passed as it is, not quoted as an argument.
        use std::os::windows::process::CommandExt;
        command.raw_arg(line);
    }
    #[cfg(not(windows))]
    command.arg(line);
    command.current_dir(folder).stdin(Stdio::null());
    command
}

/// The machine's architecture, by the names Node and Ascribe's own packages
/// use.
fn arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        "x86" => "ia32",
        other => other,
    }
}

/// Every example in `folder`, by path from it, with its hash, in path order.
/// The observations' folder, and `.git`, aren't looked in. A symbolic link
/// to a folder isn't followed, so a link can't lead the walk in a circle.
fn examples(folder: &Path, config: &ExamplesConfig) -> io::Result<BTreeMap<String, String>> {
    let mut files = BTreeMap::new();
    let mut folders: Vec<(PathBuf, String)> = vec![(folder.to_path_buf(), String::new())];
    while let Some((dir, prefix)) = folders.pop() {
        // Outside FileSystem: the examples of the folder `ascribe observe`
        // is run in, which isn't a docs project.
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = format!("{prefix}{name}");
            let kind = entry.file_type()?;
            // A link to a folder is skipped, not followed: `file_type`
            // doesn't follow links, so it isn't `is_dir`, and reading it
            // as a file would fail the run.
            if kind.is_symlink() && entry.path().is_dir() {
                continue;
            }
            if kind.is_dir() {
                if prefix.is_empty() && (name == OBSERVATIONS_FOLDER || name == ".git") {
                    continue;
                }
                folders.push((entry.path(), format!("{path}/")));
            } else if config.is_example(&path) {
                // Outside FileSystem: as above. A link to a file is read
                // where it leads.
                let bytes = fs::read(entry.path())?;
                files.insert(path, example_hash(&bytes));
            }
        }
    }
    Ok(files)
}

/// Writes the observation `name` whole to a file beside it, then renames
/// that over it, so a run that's stopped part way leaves the old one.
fn write_observation(folder: &Path, name: &str, text: &str) -> io::Result<()> {
    let target = folder.join(observation_path(name));
    let dir = folder.join(OBSERVATIONS_FOLDER);
    fs::create_dir_all(&dir)?;
    let temporary = dir.join(format!(".{name}.toml.{}", std::process::id()));
    fs::write(&temporary, text)?;
    fs::rename(&temporary, &target).inspect_err(|_| {
        let _ = fs::remove_file(&temporary);
    })
}

/// Text from the config, safe to print to a terminal: a control character is
/// written as its escape, so it can't move the cursor or change colors.
fn terminal(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_control() {
                c.escape_default().to_string()
            } else {
                c.to_string()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_text_is_escaped() {
        assert_eq!(terminal("npm test"), "npm test");
        assert_eq!(terminal("a\u{1b}[31mb\n"), "a\\u{1b}[31mb\\n");
    }

    #[test]
    fn architectures_by_their_package_names() {
        assert!(!arch().is_empty());
    }

    // The question, by `observe` with the answer given as text: a test of
    // the binary can't give it a terminal on every platform. The binary's
    // tests (`tests/all/observe.rs`) show that without one it doesn't ask,
    // and runs nothing unless given `--yes`.

    /// A folder whose test writes `ran`, so a test can tell whether it ran.
    fn folder() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(EXAMPLES_FILE),
            "include = [\"*.ts\"]\ntest = \"echo yes > ran\"\n",
        )
        .unwrap();
        fs::write(dir.path().join("a.ts"), "a();\n").unwrap();
        dir
    }

    fn args(format: Format, yes: bool) -> Args {
        Args {
            name: DEFAULT_OBSERVATION.to_owned(),
            check: false,
            yes,
            format,
        }
    }

    /// Runs `observe` in `dir`, answering `answer` when it's asked, or not
    /// asking with `None`: the exit code, standard output, and standard error.
    fn run_in(dir: &Path, format: Format, answer: Option<&str>) -> (u8, String, String) {
        run_with(dir, &args(format, false), answer)
    }

    /// [`run_in`], with `args`.
    fn run_with(dir: &Path, args: &Args, answer: Option<&str>) -> (u8, String, String) {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let mut answers = answer.map(str::as_bytes);
        let ask = answers.as_mut().map(|a| a as &mut dyn BufRead);
        let code = observe(dir, args, ask, &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    const LIST: &str = "The commands in ascribe.examples.toml:\n  test         echo yes > ran\n";

    #[test]
    fn declined_it_shows_the_commands_and_runs_nothing() {
        for answer in ["n\n", "\n", "no\n", "sure\n", ""] {
            let dir = folder();
            let (code, out, err) = run_in(dir.path(), Format::Text, Some(answer));
            assert_eq!(code, exit::FAILURE, "{answer:?}");
            assert_eq!(out, LIST);
            assert_eq!(
                err,
                "Run them? [y/N] Nothing ran, and nothing was written.\n"
            );
            assert!(!dir.path().join("ran").exists(), "{answer:?} ran the test");
            assert!(!dir.path().join(OBSERVATIONS_FOLDER).exists());
        }
    }

    #[test]
    fn agreed_it_runs_them() {
        for answer in ["y\n", "YES\n", " yes \r\n"] {
            let dir = folder();
            let (code, out, err) = run_in(dir.path(), Format::Text, Some(answer));
            assert_eq!(code, exit::OK, "{answer:?}: {err}");
            assert!(out.starts_with(LIST), "{out}");
            assert!(out.contains("echo yes > ran passed."), "{out}");
            assert_eq!(err, "Run them? [y/N] ");
            assert!(dir.path().join("ran").exists());
            assert!(
                dir.path()
                    .join(observation_path(DEFAULT_OBSERVATION))
                    .exists()
            );
        }
    }

    #[test]
    fn with_json_it_asks_on_standard_error() {
        let dir = folder();
        let (code, out, err) = run_in(dir.path(), Format::Json, Some("n\n"));
        assert_eq!(code, exit::FAILURE);
        assert_eq!(out, "");
        assert_eq!(
            err,
            format!("{LIST}Run them? [y/N] Nothing ran, and nothing was written.\n")
        );

        let (code, out, err) = run_in(dir.path(), Format::Json, Some("y\n"));
        assert_eq!(code, exit::OK, "{err}");
        assert!(err.starts_with(&format!("{LIST}Run them? [y/N] ")), "{err}");
        let report: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(report["test"]["passed"], true);
    }

    const REFUSED: &str = "error: standard input isn't a terminal, so nobody was asked before running them. Read them, then run `ascribe observe` again with `--yes` to run them. Nothing ran, and nothing was written.\n";

    #[test]
    fn not_asked_and_without_yes_it_shows_the_commands_and_runs_nothing() {
        let dir = folder();
        let (code, out, err) = run_in(dir.path(), Format::Text, None);
        assert_eq!(code, exit::FAILURE);
        assert_eq!(out, LIST);
        assert_eq!(err, REFUSED);
        assert!(!dir.path().join("ran").exists());
        assert!(!dir.path().join(OBSERVATIONS_FOLDER).exists());

        let (code, out, err) = run_in(dir.path(), Format::Json, None);
        assert_eq!(code, exit::FAILURE);
        assert_eq!(out, "");
        assert_eq!(err, format!("{LIST}{REFUSED}"));
        assert!(!dir.path().join("ran").exists());
    }

    #[test]
    fn not_asked_with_yes_it_runs_them_as_it_says() {
        let dir = folder();
        let (code, out, err) = run_with(dir.path(), &args(Format::Text, true), None);
        assert_eq!(code, exit::OK, "{err}");
        assert!(
            out.starts_with("Running the commands in ascribe.examples.toml:\n"),
            "{out}"
        );
        assert!(!err.contains("Run them?"), "{err}");
        assert!(dir.path().join("ran").exists());
    }
}
