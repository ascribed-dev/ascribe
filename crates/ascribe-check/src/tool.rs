//! Running a program the project installs, such as Vale or a link checker: with a
//! fixed argument list, never through a shell, until it ends or its time
//! runs out (content checks decision 5). What it writes is read back by the
//! caller.

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// A program couldn't say anything.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ToolError {
    /// The command can't be run: it isn't installed, or isn't where the
    /// project's settings say.
    #[error("`{command}` couldn't be run: {reason}")]
    NotRun {
        /// The command tried.
        command: String,
        /// Why.
        reason: String,
    },
    /// It ran and failed, or wrote something that isn't what it writes.
    #[error("`{command}` failed: {reason}")]
    Failed {
        /// The command tried.
        command: String,
        /// What it said.
        reason: String,
    },
    /// It didn't finish in time, and was stopped.
    #[error("`{command}` didn't finish within {seconds} seconds")]
    TimedOut {
        /// The command tried.
        command: String,
        /// The time it had.
        seconds: u64,
    },
}

impl ascribe_core::Coded for ToolError {
    fn code(&self) -> &'static str {
        match self {
            ToolError::NotRun { .. } => "tool_not_run",
            ToolError::Failed { .. } => "tool_failed",
            ToolError::TimedOut { .. } => "tool_timed_out",
        }
    }
}

impl ToolError {
    /// The command it's about.
    pub fn command(&self) -> &str {
        match self {
            ToolError::NotRun { command, .. }
            | ToolError::Failed { command, .. }
            | ToolError::TimedOut { command, .. } => command,
        }
    }

    /// A failure of `command`, from an I/O error.
    pub fn failed(command: &str, e: &io::Error) -> ToolError {
        ToolError::Failed {
            command: command.to_owned(),
            reason: e.to_string(),
        }
    }
}

/// What a program wrote: its standard output and the start of its standard
/// error.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Output {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// The most of a program's standard output that's read.
const MAX_OUTPUT: u64 = 64 << 20;

/// The most of its standard error that's kept, for the reason it gives.
const MAX_ERROR: u64 = 64 << 10;

/// The program to run: a command with a path separator is relative to the
/// project root, and a bare name is found as the shell would find it.
pub(crate) fn program(command: &str, root: &Path) -> PathBuf {
    let path = Path::new(command);
    if command.contains(['/', '\\']) && path.is_relative() {
        root.join(path)
    } else {
        path.to_path_buf()
    }
}

/// Runs `program` (as `command` names it, for messages) with `args` in
/// `folder`, giving it `input` on standard input, until it ends or `timeout`
/// passes. Its exit status isn't looked at: what it wrote tells.
pub(crate) fn run(
    command: &str,
    program: &Path,
    args: &[String],
    folder: &Path,
    input: &[u8],
    timeout: Duration,
) -> Result<Output, ToolError> {
    let mut child = Command::new(program)
        .args(args)
        .current_dir(folder)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| ToolError::NotRun {
            command: command.to_owned(),
            reason: if e.kind() == io::ErrorKind::NotFound {
                "it isn't installed, or isn't on the PATH".to_owned()
            } else {
                e.to_string()
            },
        })?;
    // Standard input is written from a thread of its own, and each output
    // pipe read to its end, so the program never blocks on a full pipe; only
    // the first `limit` bytes of each are kept.
    let feed = child.stdin.take().map(|mut pipe| {
        let input = input.to_vec();
        std::thread::spawn(move || {
            let _ = pipe.write_all(&input);
        })
    });
    let drain = |pipe: Option<Box<dyn Read + Send>>, limit: u64| {
        std::thread::spawn(move || {
            let mut out = Vec::new();
            let mut whole = true;
            if let Some(mut pipe) = pipe {
                let _ = pipe.by_ref().take(limit).read_to_end(&mut out);
                whole = matches!(io::copy(&mut pipe, &mut io::sink()), Ok(0));
            }
            (out, whole)
        })
    };
    let stdout = drain(
        child
            .stdout
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
        MAX_OUTPUT,
    );
    let stderr = drain(
        child
            .stderr
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
        MAX_ERROR,
    );
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(ToolError::TimedOut {
                    command: command.to_owned(),
                    seconds: timeout.as_secs(),
                });
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
            Err(e) => return Err(ToolError::failed(command, &e)),
        }
    }
    if let Some(feed) = feed {
        let _ = feed.join();
    }
    let (stdout, whole) = stdout.join().unwrap_or_default();
    let (stderr, _) = stderr.join().unwrap_or_default();
    if !whole {
        return Err(ToolError::Failed {
            command: command.to_owned(),
            reason: format!("it wrote more than {} MiB", MAX_OUTPUT >> 20),
        });
    }
    Ok(Output { stdout, stderr })
}

/// What a program said when it didn't write what it writes: its standard
/// error, else its standard output, on one line and at most 500 characters.
pub(crate) fn said(output: &Output) -> String {
    let error = String::from_utf8_lossy(&output.stderr);
    let text = if error.trim().is_empty() {
        String::from_utf8_lossy(&output.stdout)
    } else {
        error
    };
    let line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut short: String = line.chars().take(500).collect();
    if short.is_empty() {
        short = "it wrote nothing".to_owned();
    }
    short
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn a_missing_program_isn_t_run() {
        let error = run(
            "ascribe-test-no-such-tool",
            Path::new("ascribe-test-no-such-tool"),
            &[],
            Path::new("."),
            b"",
            Duration::from_secs(5),
        )
        .unwrap_err();
        assert!(matches!(error, ToolError::NotRun { .. }), "{error:?}");
        assert!(error.to_string().contains("isn't installed"), "{error}");
    }

    #[test]
    fn a_relative_command_is_from_the_project_root() {
        let root = Path::new("project");
        assert_eq!(program("bin/lychee", root), root.join("bin/lychee"));
        assert_eq!(program("lychee", root), PathBuf::from("lychee"));
    }

    #[cfg(unix)]
    fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join(name);
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    #[cfg(unix)]
    #[test]
    fn input_goes_in_and_output_comes_back() {
        let dir = tempfile::tempdir().unwrap();
        let cat = script(dir.path(), "echo-tool", "cat; echo said >&2");
        let output = run(
            "echo-tool",
            &cat,
            &[],
            dir.path(),
            b"one\ntwo\n",
            Duration::from_secs(5),
        )
        .unwrap();
        assert_eq!(output.stdout, b"one\ntwo\n");
        assert_eq!(said(&output), "said");
    }

    #[cfg(unix)]
    #[test]
    fn a_slow_program_is_stopped() {
        let dir = tempfile::tempdir().unwrap();
        let slow = script(dir.path(), "slow-tool", "sleep 10");
        let started = Instant::now();
        let error = run(
            "slow-tool",
            &slow,
            &[],
            dir.path(),
            b"",
            Duration::from_millis(200),
        )
        .unwrap_err();
        assert!(matches!(error, ToolError::TimedOut { .. }), "{error:?}");
        assert!(started.elapsed() < Duration::from_secs(5));
    }
}
