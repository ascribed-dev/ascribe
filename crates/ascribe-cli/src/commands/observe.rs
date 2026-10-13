//! `ascribe observe`: runs the test of the `ascribe.examples.toml` in the
//! current folder, and when it passes, writes down what it passed and where:
//! an observation, `examples.observations/default.toml`.
//!
//! This is the one place Ascribe runs a line of shell, and a test holds it
//! to that (`tests/all/program_starts.rs`). It runs only the commands of the
//! config in the folder it's run from: it doesn't look in a parent folder,
//! takes no path to a config, and never runs one a page, a source's copy, or
//! another project names. It says which file and which commands before it
//! runs them, and adds nothing to a command and substitutes nothing into it.
//! The config, the observation, and the hash are `ascribe_model::examples`'s.

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use ascribe_core::{FileId, LineIndex};
use ascribe_model::examples::{
    DEFAULT_OBSERVATION, EXAMPLES_FILE, ExamplesConfig, OBSERVATIONS_FOLDER, Observation, Problem,
    kept_line, observation_path,
};
use ascribe_model::{COPIES_FOLDER, LOCK_FILE, Lock, example_hash};
use clap::{Args as ClapArgs, ValueEnum};
use serde::Serialize;

use crate::cli::Global;
use crate::exit;

/// The version of `ascribe observe --format json`'s schema.
const SCHEMA_VERSION: u32 = 1;

/// Arguments of `ascribe observe`.
#[derive(Debug, ClapArgs)]
pub struct Args {
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
/// test ended, and the observation it wrote.
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
    /// The `[environment]` commands, in the order the config has them, each
    /// with the line it kept.
    environment: Vec<EnvironmentRun>,
    /// The test command, and how it ended.
    test: TestRun,
    /// The observation it wrote, from the current folder: none when the test
    /// failed.
    observation: Option<String>,
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
/// couldn't run: a folder that's a source's copies, no config in the current
/// folder, a config that can't be
/// read, no shell, an environment command that failed or printed nothing,
/// or an observation that can't be written.
pub fn run(_global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = match std::env::current_dir() {
        Ok(folder) => observe(&folder, args.format, &mut out, &mut err),
        Err(e) => {
            let _ = writeln!(err, "error: can't read the current folder: {e}");
            exit::FAILURE
        }
    };
    let _ = out.flush();
    exit::code(code)
}

/// Runs the config in `folder`, writing the report to `out` and what went
/// wrong to `err`, and returns the exit code.
fn observe(folder: &Path, format: Format, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    if let Some(copy) = copies_of(folder) {
        let _ = writeln!(
            err,
            "error: this folder is {copy}, which `ascribe sources fetch` copied from another repository; `ascribe observe` never runs a config that arrived as a copy. Run it in that repository, and `ascribe sources update` brings the observation here"
        );
        return exit::FAILURE;
    }
    let (config, test) = match read_config(folder, err) {
        Ok(read) => read,
        Err(code) => return code,
    };

    // Say what will run, before running anything.
    let json = format == Format::Json;
    if !json {
        let _ = writeln!(out, "Running the commands in {EXAMPLES_FILE}:");
        let _ = writeln!(out, "  test         {}", terminal(&test));
        for entry in &config.environment {
            let _ = writeln!(out, "  environment  {}", terminal(&entry.command));
        }
        let _ = out.flush();
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
        environment,
        test: TestRun {
            command: test.clone(),
            passed: status.success(),
            exit_code: status.code(),
        },
        observation: None,
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
    let path = observation_path(DEFAULT_OBSERVATION);
    if let Err(e) = write_observation(folder, &path, &observation.to_toml()) {
        let _ = writeln!(err, "error: can't write {path}: {e}");
        return exit::FAILURE;
    }
    let tested = observation.tested_line(&config);
    if json {
        report.observation = Some(path);
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
    }
    exit::OK
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

/// The config in `folder`, and its `test`: or, when it can't be read or has
/// no `test`, the exit code, with what's wrong written to `err`.
fn read_config(folder: &Path, err: &mut dyn Write) -> Result<(ExamplesConfig, String), u8> {
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
    let config = match ExamplesConfig::parse(&text) {
        Ok(config) => config,
        Err(problems) => {
            report_problems(err, &text, &problems);
            return Err(exit::FAILURE);
        }
    };
    let Some(test) = config.test.clone() else {
        let _ = writeln!(
            err,
            "error: {EXAMPLES_FILE} has no `test`, the line of shell that tests the examples"
        );
        return Err(exit::FAILURE);
    };
    Ok((config, test))
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

/// Writes the observation whole to a file beside it, then renames that over
/// it, so a run that's stopped part way leaves the old one.
fn write_observation(folder: &Path, path: &str, text: &str) -> io::Result<()> {
    let target = folder.join(path);
    let dir = folder.join(OBSERVATIONS_FOLDER);
    fs::create_dir_all(&dir)?;
    let temporary = dir.join(format!(
        ".{DEFAULT_OBSERVATION}.toml.{}",
        std::process::id()
    ));
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
}
