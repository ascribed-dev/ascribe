//! `ascribe agents hook`: the hook's protocol for each harness. What's
//! checked is the same for all of them ([`super::checker`]); a harness
//! decides how the hook's input is read and how the answer reaches the
//! model.
//!
//! After an edit, the answer is the errors in the files the tool wrote, as
//! `additionalContext`: Claude Code's and Codex's `hookSpecificOutput`, and
//! Copilot's top-level field, each added to what the model sees beside the
//! tool's result. Before the agent finishes, it's `decision: "block"` with a
//! `reason`, which every harness reads the same way and which keeps the
//! agent working. With nothing to say, the hook prints nothing and exits 0;
//! Codex expects JSON from a stop hook that exits 0, so it gets `{}`.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use ascribe_check::Project;
use ascribe_core::path::{normalize, relative_path};
use ascribe_diff::Repository;
use clap::ValueEnum;
use serde_json::{Value, json};

use super::checker::{self, Ask, Found, LIMIT};
use crate::answer::FromDisk;
use crate::exit;

/// The agent whose hook runs the command.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Harness {
    /// Claude Code: `PostToolUse` and `Stop`.
    ClaudeCode,
    /// Codex: `PostToolUse` and `Stop`.
    Codex,
    /// GitHub Copilot: `postToolUse` and `agentStop`.
    Copilot,
}

/// When the hook runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Event {
    /// After the agent writes a file: report the errors in it.
    Edit,
    /// When the agent is about to finish: check the whole project, and
    /// keep it working while there are errors.
    Stop,
}

/// How long each event waits for its check before it lets the agent go on
/// without one.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// After an edit.
    pub edit: Duration,
    /// Before the agent finishes.
    pub stop: Duration,
    /// Whether to check through the project's check server, which keeps
    /// it loaded: unless `ASCRIBE_HOOK_SERVER` is `off`.
    pub server: bool,
}

impl Default for Limits {
    /// Two seconds after an edit: a kept project answers in milliseconds,
    /// and the first check of a 3,000-page project takes about 1.3 s.
    /// Twenty before the agent finishes: checking every build of such a
    /// project takes 2 to 4 s.
    fn default() -> Limits {
        Limits {
            edit: Duration::from_secs(2),
            stop: Duration::from_secs(20),
            server: std::env::var("ASCRIBE_HOOK_SERVER").map_or(true, |v| v != "off"),
        }
    }
}

/// What the hook writes, and its exit code.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Answer {
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// The exit code.
    pub code: u8,
}

impl Answer {
    fn quiet(harness: Harness, event: Event) -> Answer {
        Answer {
            stdout: if harness == Harness::Codex && event == Event::Stop {
                "{}\n".to_owned()
            } else {
                String::new()
            },
            ..Answer::default()
        }
    }
}

/// Runs the hook on its input. Exit codes: 0 whatever was found, and when
/// the check takes too long; 1 when the input can't be read, which every
/// harness shows its user and not the model.
pub fn run(harness: Harness, event: Event, input: io::Result<String>, limits: Limits) -> Answer {
    let input = match input
        .map_err(|e| e.to_string())
        .and_then(|text| serde_json::from_str::<Value>(&text).map_err(|e| e.to_string()))
    {
        Ok(input) if input.is_object() => input,
        Ok(_) => return unreadable("it isn't a JSON object"),
        Err(e) => return unreadable(&e),
    };
    let cwd = input
        .get("cwd")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .map(|dir| normalize(&std::path::absolute(&dir).unwrap_or(dir)))
        .unwrap_or_default();
    match event {
        Event::Edit => edit(harness, &input, &cwd, limits.edit, limits.server),
        Event::Stop => stop(harness, &input, &cwd, limits.stop, limits.server),
    }
}

fn unreadable(why: &str) -> Answer {
    Answer {
        stderr: format!("ascribe agents hook: can't read the hook's input: {why}\n"),
        code: exit::PROBLEMS,
        ..Answer::default()
    }
}

fn edit(harness: Harness, input: &Value, cwd: &Path, limit: Duration, server: bool) -> Answer {
    let deadline = Instant::now() + limit;
    let quiet = Answer::quiet(harness, Event::Edit);
    let written: Vec<PathBuf> = written_files(harness, input)
        .into_iter()
        .filter(|f| Path::new(f).extension().is_some_and(|e| e == "md"))
        .map(|f| normalize(&cwd.join(f)))
        .collect();
    let mut found = Vec::new();
    for (config, files) in by_project(&written) {
        let files = checker::sources_of(&config, &files);
        if files.is_empty() {
            continue;
        }
        let shown: Vec<String> = files.iter().map(|f| from(cwd, f)).collect();
        match check_by(&config, Ask::Files(files), deadline, server) {
            Some(Ok(f)) if f.errors > 0 => found.push((shown, f)),
            // No errors, a project that can't be checked (the stop hook
            // says why), or out of time: nothing to say.
            _ => {}
        }
    }
    if found.is_empty() {
        return quiet;
    }
    let text = found
        .iter()
        .map(|(files, f)| edit_text(files, f))
        .collect::<Vec<_>>()
        .join("\n");
    let stdout = match harness {
        Harness::ClaudeCode | Harness::Codex => json!({
            "hookSpecificOutput": {
                "hookEventName": "PostToolUse",
                "additionalContext": text,
            }
        }),
        Harness::Copilot => json!({ "additionalContext": text }),
    };
    Answer {
        stdout: format!("{stdout}\n"),
        ..quiet
    }
}

/// What the model reads after an edit: the errors, then which build was
/// checked.
fn edit_text(files: &[String], found: &Found) -> String {
    let mut lines = vec![format!(
        "`ascribe check {}` reports {}:",
        files.join(" "),
        count(found.errors, "error")
    )];
    lines.extend(found.lines.iter().cloned());
    if found.errors > LIMIT {
        lines.push(format!(
            "and {} more: `ascribe check {} --editor-build --format concise`",
            found.errors - LIMIT,
            files.join(" ")
        ));
    }
    let build = found.build.as_deref().unwrap_or_default();
    lines.push(format!(
        "Checked build `{build}` only. Run `ascribe explain <code>` for any you don't recognize."
    ));
    lines.join("\n") + "\n"
}

fn stop(harness: Harness, input: &Value, cwd: &Path, limit: Duration, server: bool) -> Answer {
    let deadline = Instant::now() + limit;
    let quiet = Answer::quiet(harness, Event::Stop);
    // Every harness names it this way, Copilot too.
    if input
        .get("stop_hook_active")
        .or_else(|| input.get("stopHookActive"))
        .and_then(Value::as_bool)
        == Some(true)
    {
        return quiet;
    }
    let mut reasons = Vec::new();
    for config in changed_projects(cwd) {
        let dir = config.parent().map(|d| from(cwd, d)).unwrap_or_default();
        // Out of time, the agent goes on; a project that can't be checked
        // at all, as `ascribe check` would refuse it, holds it.
        match check_by(&config, Ask::Project, deadline, server) {
            Some(Ok(found)) if found.errors > 0 => reasons.push(stop_text(&dir, &found)),
            Some(Err(why)) => reasons.push(unchecked_text(&dir, &why)),
            Some(Ok(_)) | None => {}
        }
    }
    if reasons.is_empty() {
        return quiet;
    }
    let stdout = json!({ "decision": "block", "reason": reasons.join("\n") });
    Answer {
        stdout: format!("{stdout}\n"),
        ..quiet
    }
}

/// The command that checks the project in `dir`, from the agent's folder.
fn check_command(dir: &str) -> String {
    if dir == "." {
        "ascribe check".to_owned()
    } else {
        format!("ascribe check --config {}", crate::shell::quote(dir))
    }
}

/// Why the agent can't finish yet: the project can't be checked.
fn unchecked_text(dir: &str, why: &str) -> String {
    let command = check_command(dir);
    format!(
        "`{command}` can't check the project:\n{}\nFix this, then run `{command}`.\n",
        why.trim_end()
    )
}

/// Why the agent can't finish yet: the counts, the first errors, and what
/// to do.
fn stop_text(dir: &str, found: &Found) -> String {
    let command = check_command(dir);
    let mut lines = vec![format!(
        "`{command}` reports {} and {}:",
        count(found.errors, "error"),
        count(found.warnings, "warning")
    )];
    lines.extend(found.lines.iter().cloned());
    if found.errors > LIMIT {
        lines.push(format!(
            "and {} more errors: `{command} --format concise`",
            found.errors - LIMIT
        ));
    }
    lines.push(format!("Fix these, then run `{command}`."));
    lines.join("\n") + "\n"
}

fn count(n: usize, what: &str) -> String {
    if n == 1 {
        format!("1 {what}")
    } else {
        format!("{n} {what}s")
    }
}

/// The files the tool wrote, as its input names them.
fn written_files(harness: Harness, input: &Value) -> Vec<String> {
    let path_in = |args: &Value| {
        ["file_path", "path", "filePath"]
            .iter()
            .find_map(|key| args.get(key).and_then(Value::as_str))
            .map(str::to_owned)
    };
    match harness {
        Harness::ClaudeCode => input
            .get("tool_input")
            .and_then(path_in)
            .into_iter()
            .collect(),
        Harness::Codex => {
            let tool_input = input.get("tool_input");
            match tool_input.and_then(|t| t.get("command")) {
                Some(command) => patched_files(command),
                None => tool_input.and_then(path_in).into_iter().collect(),
            }
        }
        Harness::Copilot => {
            // `toolArgs` is the tool's arguments, as an object or as JSON
            // text; the Claude-compatible form calls it `tool_input`.
            let args = input.get("toolArgs").or_else(|| input.get("tool_input"));
            let parsed = match args {
                Some(Value::String(text)) => serde_json::from_str(text).ok(),
                other => other.cloned(),
            };
            parsed.as_ref().and_then(path_in).into_iter().collect()
        }
    }
}

/// The files a Codex patch adds, updates, or moves to: its `*** Add File:`,
/// `*** Update File:`, and `*** Move to:` lines. The command is the patch,
/// or `apply_patch` and the patch as a list of words.
fn patched_files(command: &Value) -> Vec<String> {
    let text = match command {
        Value::String(text) => text.clone(),
        Value::Array(words) => words
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    };
    text.lines()
        .filter_map(|line| {
            ["*** Add File: ", "*** Update File: ", "*** Move to: "]
                .iter()
                .find_map(|prefix| line.trim_end().strip_prefix(prefix))
        })
        .map(str::to_owned)
        .collect()
}

/// `files` grouped by the nearest `ascribe.toml` at or above each, in
/// order; files outside every project are left out.
fn by_project(files: &[PathBuf]) -> Vec<(PathBuf, Vec<PathBuf>)> {
    let mut groups: Vec<(PathBuf, Vec<PathBuf>)> = Vec::new();
    for file in files {
        let Some(config) = file.parent().and_then(Project::find_config) else {
            continue;
        };
        let config = normalize(&config);
        match groups.iter_mut().find(|(c, _)| *c == config) {
            Some((_, group)) => group.push(file.clone()),
            None => groups.push((config, vec![file.clone()])),
        }
    }
    groups
}

/// The projects to check before the agent finishes: in a git repository,
/// those whose pages or content model the working tree changes; outside
/// one, the project at or above `cwd`.
fn changed_projects(cwd: &Path) -> Vec<PathBuf> {
    let repository = match Repository::discover(cwd) {
        Ok(repository) => repository,
        Err(_) => {
            return Project::find_config(cwd)
                .map(|c| normalize(&c))
                .into_iter()
                .collect();
        }
    };
    let Ok(changed) = repository.working_changes() else {
        return Vec::new();
    };
    let root = PathBuf::from(&repository.root);
    let mut configs: Vec<PathBuf> = Vec::new();
    for path in changed {
        let file = normalize(&root.join(path.as_str()));
        let model = file
            .file_name()
            .is_some_and(|n| n == ascribe_check::MODEL_FILE);
        if !model && path.extension() != Some("md") {
            continue;
        }
        let Some(config) = file.parent().and_then(Project::find_config) else {
            continue;
        };
        let config = normalize(&config);
        if configs.contains(&config) {
            continue;
        }
        if model || checker::is_source(&config, &file) {
            configs.push(config);
        }
    }
    configs
}

/// Checks through the project's check server with `server`, or in this
/// process without, or when no server can be started. `None` when the
/// deadline passes first.
fn check_by(
    config: &Path,
    ask: Ask,
    deadline: Instant,
    server: bool,
) -> Option<Result<Found, String>> {
    if server {
        // Without a server, check here.
        if let Ok(answer) = checker::ask_server(config, &ask, deadline) {
            return answer;
        }
    }
    // On a thread of its own, so the hook can give up on it.
    let (send, receive) = mpsc::channel();
    let config = config.to_owned();
    std::thread::spawn(move || {
        let _ = send.send(checker::check(&FromDisk, &config, &ask));
    });
    receive
        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
        .ok()
}

/// `path` as seen from `cwd`, `/`-separated; `.` for `cwd` itself. Both
/// are resolved first where they exist, so two spellings of one folder
/// (on Windows, a short `RUNNER~1` and its long name) meet.
fn from(cwd: &Path, path: &Path) -> String {
    // Outside FileSystem: resolving the agent's folder and a project's,
    // only to name one from the other.
    let real = |p: &Path| std::fs::canonicalize(p).ok();
    let resolved = match (real(cwd), real(path)) {
        (Some(cwd), Some(path)) => relative_path(&cwd, &path),
        _ => None,
    };
    match resolved.or_else(|| relative_path(cwd, path)) {
        Some(rel) if rel.is_root() => ".".to_owned(),
        Some(rel) => rel.to_string(),
        None => path.to_string_lossy().replace('\\', "/"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::write(
            root.join("ascribe.toml"),
            "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(root.join("docs")).unwrap();
        std::fs::write(
            root.join("docs/a.md"),
            "---\ntitle: A\n---\n\n[x](gone.md)\n",
        )
        .unwrap();
        dir
    }

    #[test]
    fn out_of_time_the_agent_goes_on() {
        let dir = project();
        let limits = Limits {
            edit: Duration::ZERO,
            stop: Duration::ZERO,
            server: false,
        };
        let input = json!({
            "cwd": dir.path(),
            "tool_input": { "file_path": dir.path().join("docs").join("a.md") },
        })
        .to_string();
        let answer = run(Harness::ClaudeCode, Event::Edit, Ok(input.clone()), limits);
        assert_eq!(answer, Answer::default());
        let answer = run(Harness::Codex, Event::Stop, Ok(input.clone()), limits);
        assert_eq!(answer.stdout, "{}\n");
        // With time, the same input reports the error.
        let limits = Limits {
            edit: Duration::from_secs(60),
            ..limits
        };
        let answer = run(Harness::ClaudeCode, Event::Edit, Ok(input), limits);
        assert!(answer.stdout.contains("[ASC036]"), "{answer:?}");
    }

    #[test]
    fn a_codex_patch_names_the_files_it_writes() {
        let patch = "*** Begin Patch\n*** Add File: docs/new.md\n+# New\n*** Update File: docs/old.md\n\
                     *** Move to: docs/moved.md\n@@\n-a\n+b\n*** Delete File: docs/gone.md\n*** End Patch";
        assert_eq!(
            patched_files(&json!(patch)),
            ["docs/new.md", "docs/old.md", "docs/moved.md"]
        );
        assert_eq!(
            patched_files(&json!(["apply_patch", patch])),
            ["docs/new.md", "docs/old.md", "docs/moved.md"]
        );
    }
}
