//! Running Vale and reading what it says.
//!
//! Vale is a program the project installs (content checks decision 5): it's
//! run with a fixed argument list, never through a shell, on copies of the
//! pages' prose written to a folder of its own, and its JSON is read back.
//! It isn't linked, and nothing is downloaded.

use std::collections::BTreeMap;
use std::io;
use std::path::PathBuf;
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
}

/// Runs the `vale` program.
#[derive(Clone, Copy, Debug, Default)]
pub struct Program;

impl Linter for Program {
    fn lint(&self, request: &Request) -> Result<BTreeMap<String, Vec<Alert>>, ValeError> {
        if request.files.is_empty() {
            return Ok(BTreeMap::new());
        }
        let folder = Scratch::new().map_err(|e| failed(request, &e))?;
        let text = folder.path.join("text");
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
                std::fs::write(folder.path.join(".vale.ini"), "")
                    .map_err(|e| failed(request, &e))?;
                let list: Vec<String> = configs.iter().map(|c| c.display().to_string()).collect();
                args.push(format!("--sources={}", list.join(",")));
            }
        }
        args.push(".".to_owned());
        let output = tool::run(
            &request.command,
            &tool::program(&request.command, &request.root),
            &args,
            &text,
            b"",
            request.timeout,
        )?;
        read(request, &output.stdout, &output.stderr)
    }
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
        std::fs::create_dir_all(&path)?;
        Ok(Scratch { path })
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use std::time::Instant;

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

    #[cfg(unix)]
    #[test]
    fn a_slow_program_is_stopped() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("slow-vale");
        std::fs::write(&script, "#!/bin/sh\nsleep 10\n").unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let mut slow = request(&script.display().to_string());
        slow.timeout = Duration::from_millis(200);
        let started = Instant::now();
        let error = Program.lint(&slow).unwrap_err();
        assert!(matches!(error, ValeError::TimedOut { .. }), "{error:?}");
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[cfg(unix)]
    #[test]
    fn a_program_that_writes_without_end_is_a_failure() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("loud-vale");
        let bytes = (64 << 20) + 1;
        std::fs::write(&script, format!("#!/bin/sh\nhead -c {bytes} /dev/zero\n")).unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let error = Program
            .lint(&request(&script.display().to_string()))
            .unwrap_err();
        assert!(matches!(error, ValeError::Failed { .. }), "{error:?}");
        assert!(error.to_string().contains("64 MiB"), "{error}");
    }
}
