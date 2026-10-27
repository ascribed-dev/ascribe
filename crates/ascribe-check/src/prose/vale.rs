//! Running Vale and reading what it says.
//!
//! Vale is a program the project installs (content checks decision 5): it's
//! run with a fixed argument list, never through a shell, on copies of the
//! pages' prose written to a folder of its own, and its JSON is read back.
//! It isn't linked, and nothing is downloaded.

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde::Deserialize;

use crate::tool;

/// What to run Vale on, and how.
#[derive(Clone, Debug)]
pub struct Request {
    /// The command, as `[checks.vale] command` gives it. One with a path
    /// separator is relative to `root`; a bare name is looked up as the
    /// shell would.
    pub command: String,
    /// The project root.
    pub root: PathBuf,
    /// The configuration files Vale reads, in order; a later one adds to
    /// the earlier ones.
    pub configs: Vec<PathBuf>,
    /// Each file's path (relative, `/`-separated) and text.
    pub files: Vec<(String, String)>,
    /// How long Vale may take.
    pub timeout: Duration,
}

/// One of Vale's alerts, as its JSON gives it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Alert {
    /// The rule, `Style.Rule`.
    pub check: String,
    /// What it says.
    pub message: String,
    /// `suggestion`, `warning`, or `error`.
    pub severity: String,
    /// The line, from 1.
    pub line: usize,
    /// The first and last characters on the line, from 1.
    pub span: (usize, usize),
    /// The address of the rule's documentation, or empty.
    pub link: String,
    /// What Vale suggests doing.
    pub action: Action,
    /// The text that would replace the match, when the rule gives some.
    pub suggestions: Vec<String>,
}

/// An alert's action.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Action {
    /// `replace`, `remove`, `edit`, `suggest`, or empty.
    pub name: String,
    /// What the action works with: for `replace`, the replacements; for
    /// `edit`, how to edit the match (`["truncate", " "]`). Vale before 3.21
    /// gives no `Suggestions`, only this.
    pub params: Vec<String>,
}

/// Vale couldn't say anything about the prose: it couldn't be run, failed,
/// or ran out of time.
pub type ValeError = crate::tool::ToolError;

/// What runs Vale: the program itself, or a stand-in in tests.
pub trait Linter: Send + Sync {
    /// Vale's alerts for each file of the request, by its path.
    ///
    /// # Errors
    ///
    /// Vale couldn't be run, failed, or ran out of time.
    fn lint(&self, request: &Request) -> Result<BTreeMap<String, Vec<Alert>>, ValeError>;

    /// Why it won't start `command`, as `[checks.vale] command` gives it,
    /// for the project at `root`, when it won't: what the
    /// `prose-not-checked` advice says instead. Nothing is written for
    /// Vale, or run, then.
    fn refuses(&self, _command: &str, _root: &Path) -> Option<String> {
        None
    }
}

/// Whether `command`, as `[checks.vale] command` gives it, is a program in
/// the project rather than one found by name on the `PATH`: a relative
/// path, which is from the project root, or an absolute one that leads
/// into the project's folder.
pub fn in_project(command: &str, root: &Path) -> bool {
    if !command.contains(['/', '\\']) {
        return false;
    }
    let path = Path::new(command);
    if path.is_relative() {
        return true;
    }
    // Through links, so a link can't hide that it's inside, and as far as
    // the path exists, so a program not there yet compares like its folder.
    let real = crate::scope::real;
    real(path).starts_with(real(root))
}

/// Runs Vale as `inner` does, unless `[checks.vale] command` is a program
/// in the project ([`in_project`]): for a caller that can't ask anyone
/// first, such as the agents' hook. A bare name found on the `PATH` runs.
pub struct OnPath<'a> {
    /// What runs Vale when it may.
    pub inner: &'a dyn Linter,
    /// Who doesn't start it, for the advice: "the agents' hook".
    pub who: &'static str,
}

impl Linter for OnPath<'_> {
    fn lint(&self, request: &Request) -> Result<BTreeMap<String, Vec<Alert>>, ValeError> {
        self.inner.lint(request)
    }

    fn refuses(&self, command: &str, root: &Path) -> Option<String> {
        if in_project(command, root) {
            Some(format!(
                "{} doesn't start `{command}`, a program in the project",
                self.who
            ))
        } else {
            self.inner.refuses(command, root)
        }
    }
}

/// Starts no program at all, and says why: for a caller that never runs
/// Vale, such as the MCP server, whose tools say they only read, or one
/// the person said no to.
pub struct NotStarted(pub String);

impl Linter for NotStarted {
    fn lint(&self, _request: &Request) -> Result<BTreeMap<String, Vec<Alert>>, ValeError> {
        Ok(BTreeMap::new())
    }

    fn refuses(&self, _command: &str, _root: &Path) -> Option<String> {
        Some(self.0.clone())
    }
}

/// Runs the `vale` program.
#[derive(Clone, Copy, Debug, Default)]
pub struct Program;

impl Linter for Program {
    fn lint(&self, request: &Request) -> Result<BTreeMap<String, Vec<Alert>>, ValeError> {
        if request.files.is_empty() {
            return Ok(BTreeMap::new());
        }
        let program = tool::program(&request.command, &request.root);
        check_version(request, &program)?;
        let folder = Scratch::new().map_err(|e| failed(request, &e))?;
        let text = folder.path.join("text");
        // Library write: the text Vale lints, in a scratch folder of its own
        // that goes when the lint is done.
        for (path, contents) in &request.files {
            let file = text.join(path);
            if let Some(parent) = file.parent() {
                std::fs::create_dir_all(parent).map_err(|e| failed(request, &e))?;
            }
            std::fs::write(&file, contents).map_err(|e| failed(request, &e))?;
        }
        let mut args = vec!["--no-global".to_owned(), "--output=JSON".to_owned()];
        match request.configs.as_slice() {
            [one] => args.push(format!("--config={}", one.display())),
            configs => {
                // Vale reads every file `--sources` lists, in order, but
                // won't start unless it also finds a `.vale.ini` above the
                // folder it's run in; that one is never read.
                // Library write: as above.
                std::fs::write(folder.path.join(".vale.ini"), "")
                    .map_err(|e| failed(request, &e))?;
                let list: Vec<String> = configs.iter().map(|c| c.display().to_string()).collect();
                args.push(format!("--sources={}", list.join(",")));
            }
        }
        args.push(".".to_owned());
        let output = tool::run(
            &request.command,
            &program,
            &args,
            &text,
            b"",
            request.timeout,
        )?;
        read(request, &output.stdout, &output.stderr)
    }
}

/// The oldest Vale that works: before it, a vocabulary didn't exempt its
/// words from `repetition` rules, and a second configuration given with
/// `--sources` had to have its own styles folder.
pub const MIN_VERSION: (u32, u32, u32) = (3, 16, 0);

/// Fails when `vale --version` names a Vale older than [`MIN_VERSION`]. A
/// version it can't read, such as a build from source, is let through.
fn check_version(request: &Request, program: &Path) -> Result<(), ValeError> {
    let output = tool::run(
        &request.command,
        program,
        &["--version".to_owned()],
        &request.root,
        b"",
        request.timeout,
    )?;
    let said = String::from_utf8_lossy(&output.stdout);
    match version(&said) {
        Some(found) if found < MIN_VERSION => {
            let (major, minor, _) = MIN_VERSION;
            let (a, b, c) = found;
            Err(ValeError::Failed {
                command: request.command.clone(),
                reason: format!(
                    "it's Vale {a}.{b}.{c}, and Ascribe needs Vale {major}.{minor} or later"
                ),
            })
        }
        _ => Ok(()),
    }
}

/// The version in what `vale --version` says (`vale version 3.24.0`).
fn version(said: &str) -> Option<(u32, u32, u32)> {
    let word = said.split_whitespace().last()?;
    let mut parts = word.trim_start_matches('v').split('.');
    let mut next = || parts.next()?.parse::<u32>().ok();
    Some((next()?, next()?, next().unwrap_or(0)))
}

/// What Vale said: its alerts by file, or why it couldn't lint. Vale exits
/// with 1 when an alert is an error, which isn't a failure; whether the
/// output is its JSON of alerts is what tells.
fn read(
    request: &Request,
    stdout: &[u8],
    stderr: &[u8],
) -> Result<BTreeMap<String, Vec<Alert>>, ValeError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Problem {
        text: String,
    }
    let text = String::from_utf8_lossy(stdout);
    let text = text.trim();
    if let Ok(alerts) = serde_json::from_str::<BTreeMap<String, Vec<Alert>>>(text) {
        // Paths as Vale wrote them, `/`-separated whatever the platform.
        return Ok(alerts
            .into_iter()
            .map(|(path, alerts)| {
                let path = path.replace('\\', "/");
                let path = path.strip_prefix("./").unwrap_or(&path).to_owned();
                (path, alerts)
            })
            .collect());
    }
    let reason = match serde_json::from_str::<Problem>(text) {
        Ok(problem) => problem.text,
        Err(_) => {
            let said = String::from_utf8_lossy(stderr).trim().to_owned();
            if said.is_empty() {
                text.chars().take(500).collect()
            } else {
                said.chars().take(500).collect()
            }
        }
    };
    Err(ValeError::Failed {
        command: request.command.clone(),
        reason: one_line(&reason),
    })
}

fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn failed(request: &Request, e: &io::Error) -> ValeError {
    ValeError::failed(&request.command, e)
}

/// A folder of its own in the system's temporary folder, removed when it's
/// dropped.
struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new() -> io::Result<Scratch> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.subsec_nanos());
        let name = format!(
            "ascribe-vale-{}-{nanos}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join(name);
        // Library write: the scratch folder, under the system's temporary one.
        std::fs::create_dir_all(&path)?;
        Ok(Scratch { path })
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // Library write: as in `new`.
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn request(command: &str) -> Request {
        Request {
            command: command.to_owned(),
            root: PathBuf::from("."),
            configs: vec![PathBuf::from(".vale.ini")],
            files: vec![("a.md".to_owned(), "Text.\n".to_owned())],
            timeout: Duration::from_secs(5),
        }
    }

    #[test]
    fn reads_vale_s_json() {
        let json = br#"{"a.md": [{"Action": {"Name": "replace", "Params": ["the"]},
            "Suggestions": ["the"], "Span": [3, 5], "Check": "Ascribe.Typos",
            "Description": "", "Link": "https://example.com/typos",
            "Message": "Use 'the'.", "Severity": "suggestion", "Match": "teh", "Line": 2}],
            ".\\sub\\b.md": []}"#;
        let alerts = read(&request("vale"), json, b"").unwrap();
        let a = &alerts["a.md"][0];
        assert_eq!(a.check, "Ascribe.Typos");
        assert_eq!((a.line, a.span), (2, (3, 5)));
        assert_eq!(a.suggestions, ["the"]);
        assert_eq!(a.action.name, "replace");
        assert_eq!(a.link, "https://example.com/typos");
        assert!(alerts.contains_key("sub/b.md"));
    }

    #[test]
    fn reads_vale_s_errors() {
        let json = br#"{"Line": 0, "Path": "", "Text": "path 'x.ini' does not exist", "Code": "E100", "Span": 0}"#;
        let error = read(&request("vale"), json, b"").unwrap_err();
        assert_eq!(
            error,
            ValeError::Failed {
                command: "vale".to_owned(),
                reason: "path 'x.ini' does not exist".to_owned()
            }
        );
        let error = read(&request("vale"), b"", b"panic:\n  something\n").unwrap_err();
        assert_eq!(error.to_string(), "`vale` failed: panic: something");
    }

    #[test]
    fn a_missing_program_isn_t_run() {
        let error = Program
            .lint(&request("ascribe-test-no-such-vale"))
            .unwrap_err();
        assert!(matches!(error, ValeError::NotRun { .. }), "{error:?}");
        assert!(error.to_string().contains("isn't installed"), "{error}");
    }

    /// What `Program` says of a shell script with `body`, run as Vale with
    /// `timeout`. Another test's fork can hold the new script open for
    /// writing for a moment, which Linux reports as a busy text file, so a
    /// run that meets one is tried again.
    #[cfg(unix)]
    fn lint_script(body: &str, timeout: Duration) -> ValeError {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("vale");
        std::fs::write(&script, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let mut request = request(&script.display().to_string());
        request.timeout = timeout;
        for _ in 0..50 {
            let error = Program.lint(&request).unwrap_err();
            if !error.to_string().contains("Text file busy") {
                return error;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        Program.lint(&request).unwrap_err()
    }

    #[cfg(unix)]
    #[test]
    fn a_slow_program_is_stopped() {
        let started = std::time::Instant::now();
        let error = lint_script("sleep 10", Duration::from_millis(200));
        assert!(matches!(error, ValeError::TimedOut { .. }), "{error:?}");
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[cfg(unix)]
    #[test]
    fn a_program_that_writes_without_end_is_a_failure() {
        let bytes = (64 << 20) + 1;
        let error = lint_script(
            &format!("head -c {bytes} /dev/zero"),
            Duration::from_secs(5),
        );
        assert!(matches!(error, ValeError::Failed { .. }), "{error:?}");
        assert!(error.to_string().contains("64 MiB"), "{error}");
    }

    #[test]
    fn reads_vale_s_version() {
        assert_eq!(version("vale version 3.24.0\n"), Some((3, 24, 0)));
        assert_eq!(version("vale version v3.12.1"), Some((3, 12, 1)));
        assert_eq!(version("vale version master"), None);
        assert!(version("vale version 3.12.0").unwrap() < MIN_VERSION);
    }

    #[cfg(unix)]
    #[test]
    fn an_old_vale_isn_t_run() {
        let error = lint_script("echo vale version 3.12.0", Duration::from_secs(5));
        assert!(
            error
                .to_string()
                .contains("it's Vale 3.12.0, and Ascribe needs Vale 3.16 or later"),
            "{error}"
        );
    }
}
