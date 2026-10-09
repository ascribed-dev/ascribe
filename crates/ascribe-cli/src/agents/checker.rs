//! The checks behind `ascribe agents hook`, and the check server that keeps
//! a project loaded between them.
//!
//! A hook is a new process for each edit, and checking one file means
//! loading and indexing the whole project, which takes over a second on a
//! large one. So the first hook in a project starts a check server: the same
//! binary, `ascribe agents hook-server`, in the background, which keeps the
//! project loaded and answers each hook in milliseconds: for the files an
//! edit wrote, as the language server does (`ascribe_lsp::Watched`), which
//! redoes only what the edit affects; for the whole project, as the MCP
//! server does (`crate::mcp::cache`). Hook and server talk through files
//! in a folder of the user's cache folder named for the project
//! ([`Mailbox`]), not a socket: the binary opens none. It stops after
//! [`IDLE`] without a request, when the project's `ascribe.toml` is gone,
//! or when another server has taken its place. `ASCRIBE_HOOK_SERVER=off`
//! checks in the hook's own process instead.

use std::collections::HashSet;
use std::hash::{BuildHasher, Hash, Hasher};
use std::io::{self, Cursor, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use ascribe_check::{Project, Reported, Severity};
use ascribe_core::RelPath;
use ascribe_core::path::{normalize, relative_path};
use ascribe_resolve::is_source_path;
use serde::{Deserialize, Serialize};

use crate::answer::Projects;
use crate::cli::{Color, Global};
use crate::commands::check;
use crate::mcp::cache::{Stamp, content_listing};
use crate::report::concise;
use ascribe_lsp::Watched;

/// The most errors a hook passes on.
pub const LIMIT: usize = 10;

/// How long a check server waits for a request before it stops.
pub const IDLE: Duration = Duration::from_secs(10 * 60);

/// What a check found, as a hook reports it: the counts, the first
/// [`LIMIT`] errors as `ascribe check --format concise` writes them, and
/// for a quick check the build it covered.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Found {
    /// How many errors.
    pub errors: usize,
    /// How many warnings.
    pub warnings: usize,
    /// The first errors' lines, without their line ends.
    pub lines: Vec<String>,
    /// The build a quick check covered; `None` for every build.
    pub build: Option<String>,
}

/// What a hook asks.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ask {
    /// Check these files, the editor's build only: absolute paths of
    /// source files of the project.
    Files(Vec<PathBuf>),
    /// Check the whole project, every build.
    Project,
}

/// The source files among `files` that belong to the project of
/// `config` and are there. Reads only the content model.
pub fn sources_of(config: &Path, files: &[PathBuf]) -> Vec<PathBuf> {
    let Some(content) = content_dir(config) else {
        return Vec::new();
    };
    // Outside FileSystem: whether a file the agent wrote is there, before
    // the check reads it through the project.
    files
        .iter()
        .filter(|file| in_content(config, &content, file) && file.is_file())
        .cloned()
        .collect()
}

/// Whether `file` (absolute, normalized) is, or was, a source file of the
/// project of `config`: in its content root, Markdown, not hidden, and in
/// no project nested in it. Reads only the content model.
pub fn is_source(config: &Path, file: &Path) -> bool {
    content_dir(config).is_some_and(|content| in_content(config, &content, file))
}

/// The project's content root, absolute and normalized; `None` when the
/// content model doesn't load, and so can't be checked file by file (the
/// full check says why).
fn content_dir(config: &Path) -> Option<PathBuf> {
    let model = Project::load_model(config).ok()?;
    let content = RelPath::parse(&model.model.project.content_root).ok()?;
    let root = normalize(&std::path::absolute(&model.root).unwrap_or(model.root));
    Some(normalize(&root.join(content.as_str())))
}

fn in_content(config: &Path, content: &Path, file: &Path) -> bool {
    relative_path(content, file).is_some_and(|rel| is_source_path(&rel))
        && file
            .parent()
            .and_then(Project::find_config)
            .is_some_and(|c| normalize(&c) == normalize(config))
}

/// Checks what `ask` names in the project of `config`, as `ascribe check`
/// does: `--editor-build` with the files, or the whole project.
///
/// # Errors
///
/// The project can't be loaded or the files can't be checked: what
/// `ascribe check` would say.
pub fn check(projects: &dyn Projects, config: &Path, ask: &Ask) -> Result<Found, String> {
    let global = Global {
        config: Some(config.to_owned()),
        color: Color::Never,
    };
    let (paths, editor_build) = match ask {
        Ask::Files(files) => (files.clone(), true),
        Ask::Project => (Vec::new(), false),
    };
    let args = check::Args {
        paths,
        stdin: false,
        path: None,
        build: Vec::new(),
        editor_build,
        summary: false,
        format: check::Format::Concise,
        deny_warnings: false,
    };
    let outcome = match check::run_check(projects, &global, &args, &mut Cursor::new("")) {
        Ok(outcome) => outcome,
        Err(stopped) => {
            let mut out = Vec::new();
            let mut err = Vec::new();
            check::report_stopped(stopped, &args, false, &mut out, &mut err);
            out.extend(err);
            return Err(String::from_utf8_lossy(&out).trim_end().to_owned());
        }
    };
    let (reported, files, build) = outcome.parts();
    let errors: Vec<&Reported> = concise::order(files, reported)
        .into_iter()
        .map(|i| &reported[i])
        .filter(|r| r.diagnostic.severity == Severity::Error)
        .collect();
    let mut lines = Vec::new();
    for r in errors.iter().take(LIMIT) {
        let mut line = Vec::new();
        concise::write_line(&mut line, files, r).map_err(|e| e.to_string())?;
        lines.push(String::from_utf8_lossy(&line).trim_end().to_owned());
    }
    Ok(Found {
        errors: outcome.counts.errors,
        warnings: outcome.counts.warnings,
        lines,
        build: editor_build.then(|| build.map(str::to_owned)).flatten(),
    })
}

/// No check server can be started: there's no cache folder for its
/// files, or the binary can't be run again.
#[derive(Debug)]
pub struct NoServer;

/// Asks the project's check server, starting one if there's none, and
/// waits for its answer until `deadline`: `Ok(None)` when the deadline
/// passes first.
///
/// # Errors
///
/// [`NoServer`] when there's none and one can't be started.
pub fn ask_server(
    config: &Path,
    ask: &Ask,
    deadline: Instant,
) -> Result<Option<Result<Found, String>>, NoServer> {
    let mailbox = Mailbox::of(config).ok_or(NoServer)?;
    let mut started = false;
    let mut start = || -> Result<(), NoServer> {
        if !started {
            start_server(config).ok_or(NoServer)?;
            started = true;
        }
        Ok(())
    };
    if !mailbox.served() {
        start()?;
    }
    // A request waits in the folder for a server to answer it, one
    // starting included.
    let name = mailbox.post(ask).map_err(|_| NoServer)?;
    let mut looked = Instant::now();
    loop {
        if let Some(answer) = mailbox.take_answer(&name) {
            return Ok(Some(answer));
        }
        if Instant::now() >= deadline {
            mailbox.withdraw(&name);
            return Ok(None);
        }
        // A server that stopped after the request was written.
        if looked.elapsed() >= Duration::from_millis(250) {
            if !mailbox.served() {
                start()?;
            }
            looked = Instant::now();
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Starts the project's check server in the background, detached from
/// this process, with nothing on its standard streams.
fn start_server(config: &Path) -> Option<()> {
    let exe = std::env::current_exe().ok()?;
    let mut command = std::process::Command::new(exe);
    command
        .args(["agents", "hook-server", "--config"])
        .arg(config)
        // Not the project's folder, which a running process would keep
        // from being removed on Windows.
        .current_dir(std::env::temp_dir())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    detach(&mut command);
    command.spawn().ok().map(|_| ())
}

#[cfg(unix)]
fn detach(command: &mut std::process::Command) {
    use std::os::unix::process::CommandExt;
    // Its own process group, so a harness that stops the hook's group
    // leaves the server running.
    command.process_group(0);
}

#[cfg(windows)]
fn detach(command: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    command.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
}

#[cfg(not(any(unix, windows)))]
fn detach(_command: &mut std::process::Command) {}

/// The binary's version: each version has a folder, and a server, of its
/// own.
const VERSION: &str = env!("ASCRIBE_VERSION");

/// How often a server says it's running, by touching its file.
const HEARTBEAT: Duration = Duration::from_secs(1);

/// How long after its last heartbeat a server is taken to have stopped.
const STALE: Duration = Duration::from_secs(5);

/// How long a request or an answer nobody took is kept.
const ABANDONED: Duration = Duration::from_secs(60);

/// How long after a request the server looks for the next one often.
const BUSY: Duration = Duration::from_secs(30);

/// A server, as its `server.json` names it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Server {
    pid: u32,
    // Tells it from a later process given the same id.
    nonce: String,
}

/// A project's folder in the user's cache folder, through which hooks
/// and its check server talk: the server's `server.json`, the requests,
/// `req-*.json`, and the answers, `res-*.json`. Each file is written
/// whole or not at all, through a file beside it renamed into place.
/// Nothing else can reach it: the folder is the user's own.
struct Mailbox {
    dir: PathBuf,
}

impl Mailbox {
    /// The folder for the project of `config`, and this binary's version.
    fn of(config: &Path) -> Option<Mailbox> {
        let config = normalize(&std::path::absolute(config).ok()?);
        // The same binary hashes it the same way, which is all a name
        // needs; `DefaultHasher::new()` has fixed keys.
        let mut hasher = std::hash::DefaultHasher::new();
        config.hash(&mut hasher);
        let dir = ascribe_sources::cache_root()?
            .join("hooks")
            .join(format!("{:016x}", hasher.finish()))
            .join(VERSION);
        Some(Mailbox { dir })
    }

    fn server_file(&self) -> PathBuf {
        self.dir.join("server.json")
    }

    /// The server its file names.
    fn server(&self) -> Option<Server> {
        // Outside FileSystem: the check server's own file, in the user's
        // cache folder.
        let text = std::fs::read_to_string(self.server_file()).ok()?;
        serde_json::from_str(&text).ok()
    }

    /// Whether a server is running: its file was touched lately.
    fn served(&self) -> bool {
        // Outside FileSystem: the check server's own file, in the user's
        // cache folder.
        std::fs::metadata(self.server_file())
            .and_then(|m| m.modified())
            .is_ok_and(|touched| touched.elapsed().map_or(true, |since| since < STALE))
    }

    /// Writes `text` to `name` in the folder, whole or not at all.
    fn write(&self, name: &str, text: &str) -> io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        let partial = self.dir.join(format!("{name}.tmp"));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        options.open(&partial)?.write_all(text.as_bytes())?;
        std::fs::rename(&partial, self.dir.join(name))
    }

    /// Writes a request for `ask`, and returns its name.
    fn post(&self, ask: &Ask) -> io::Result<String> {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let name = format!("{}-{nanos}.json", std::process::id());
        let text = serde_json::to_string(ask).map_err(io::Error::other)?;
        self.write(&format!("req-{name}"), &text)?;
        Ok(name)
    }

    /// The answer to the request `name`, once it's there; taken from the
    /// folder.
    fn take_answer(&self, name: &str) -> Option<Result<Found, String>> {
        let path = self.dir.join(format!("res-{name}"));
        // Outside FileSystem: the check server's answer, in the user's
        // cache folder.
        let text = std::fs::read_to_string(&path).ok()?;
        let _ = std::fs::remove_file(&path);
        serde_json::from_str(&text).ok()
    }

    /// Removes the request `name`, unanswered, and an answer that came
    /// too late.
    fn withdraw(&self, name: &str) {
        let _ = std::fs::remove_file(self.dir.join(format!("req-{name}")));
        let _ = std::fs::remove_file(self.dir.join(format!("res-{name}")));
    }

    /// Takes the requests waiting, oldest first: each one's name and what
    /// it asks. A request nobody took for [`ABANDONED`] is removed
    /// instead, and so is an answer.
    fn take_requests(&self) -> Vec<(String, Ask)> {
        // Outside FileSystem: the requests and answers, in the user's cache
        // folder.
        let Ok(entries) = std::fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut requests = Vec::new();
        for entry in entries.flatten() {
            let file = entry.file_name();
            let Some(file) = file.to_str() else { continue };
            let old = entry
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.elapsed().ok())
                .is_some_and(|age| age >= ABANDONED);
            if let Some(name) = file.strip_prefix("req-") {
                // Outside FileSystem: a hook's request, in the user's
                // cache folder.
                let text = std::fs::read_to_string(entry.path());
                if std::fs::remove_file(entry.path()).is_err() || old {
                    continue;
                }
                if let Some(ask) = text.ok().and_then(|t| serde_json::from_str(&t).ok()) {
                    requests.push((name.to_owned(), ask));
                }
            } else if old && (file.starts_with("res-") || file.ends_with(".tmp")) {
                let _ = std::fs::remove_file(entry.path());
            }
        }
        requests.sort_by(|a, b| a.0.cmp(&b.0));
        requests
    }

    /// Answers the request `name`.
    fn answer(&self, name: &str, found: &Result<Found, String>) {
        if let Ok(text) = serde_json::to_string(found) {
            let _ = self.write(&format!("res-{name}"), &text);
        }
    }
}

/// Runs the check server for the project of `config` until it's idle for
/// `idle`, the project's `ascribe.toml` is gone, or another server has
/// taken its place.
///
/// # Errors
///
/// It can't write its file.
pub fn serve(config: &Path, idle: Duration) -> io::Result<()> {
    let config = normalize(&std::path::absolute(config)?);
    let mailbox = Mailbox::of(&config).ok_or_else(|| io::Error::other("no cache folder"))?;
    let me = Server {
        pid: std::process::id(),
        nonce: format!(
            "{:016x}",
            std::collections::hash_map::RandomState::new().hash_one(Instant::now())
        ),
    };
    let text = serde_json::to_string(&me).map_err(io::Error::other)?;
    mailbox.write("server.json", &text)?;
    // The heartbeat goes on while a check runs, which can take longer
    // than `STALE` on a large project.
    let beating = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let heart = {
        let beating = beating.clone();
        let file = mailbox.server_file();
        let me = me.clone();
        let server = Mailbox {
            dir: mailbox.dir.clone(),
        };
        std::thread::spawn(move || {
            while beating.load(std::sync::atomic::Ordering::Relaxed) {
                std::thread::sleep(HEARTBEAT);
                if server.server().as_ref() != Some(&me) {
                    break;
                }
                let _ = std::fs::OpenOptions::new()
                    .write(true)
                    .open(&file)
                    .and_then(|f| f.set_modified(std::time::SystemTime::now()));
            }
        })
    };
    let mut kept = Kept {
        cache: crate::mcp::Cache::default(),
        watched: None,
    };
    let mut last = Instant::now();
    let mut looked = Instant::now();
    loop {
        for (name, ask) in mailbox.take_requests() {
            let found = kept.check(&config, &ask);
            mailbox.answer(&name, &found);
            last = Instant::now();
        }
        // Outside FileSystem: whether the project's `ascribe.toml` is still
        // there.
        if looked.elapsed() >= HEARTBEAT {
            if last.elapsed() >= idle || !config.is_file() || mailbox.server().as_ref() != Some(&me)
            {
                break;
            }
            looked = Instant::now();
        }
        let wait = if last.elapsed() < BUSY { 2 } else { 25 };
        std::thread::sleep(Duration::from_millis(wait));
    }
    beating.store(false, std::sync::atomic::Ordering::Relaxed);
    let _ = heart.join();
    if mailbox.server().as_ref() == Some(&me) {
        let _ = std::fs::remove_file(mailbox.server_file());
        // Only when nothing is waiting in them.
        let _ = std::fs::remove_dir(&mailbox.dir);
        if let Some(project) = mailbox.dir.parent() {
            let _ = std::fs::remove_dir(project);
        }
    }
    Ok(())
}

/// What a check server keeps: the project loaded for a full check, and
/// kept current, as the language server keeps it, for the files an edit
/// wrote.
struct Kept {
    cache: crate::mcp::Cache,
    watched: Option<(Watched, Vec<Stamp>)>,
}

impl Kept {
    fn check(&mut self, config: &Path, ask: &Ask) -> Result<Found, String> {
        match ask {
            Ask::Project => check(&self.cache, config, ask),
            Ask::Files(files) => Ok(self.check_files(config, files)),
        }
    }

    /// The editor's build's problems in `files`, after telling the kept
    /// project what changed on disk since the last request: what the
    /// listing shows, and `files`, whose times may not have moved.
    fn check_files(&mut self, config: &Path, files: &[PathBuf]) -> Found {
        let listing = listing(config);
        let (watched, before) = self
            .watched
            .get_or_insert_with(|| (Watched::load(config), listing.clone()));
        let then: HashSet<&Stamp> = before.iter().collect();
        let mut present: Vec<PathBuf> = listing
            .iter()
            .filter(|now| !then.contains(now))
            .map(|(path, _, _)| path.clone())
            .collect();
        present.extend(files.iter().cloned());
        present.sort();
        present.dedup();
        let now: HashSet<&Path> = listing.iter().map(|(path, _, _)| path.as_path()).collect();
        let gone: Vec<PathBuf> = before
            .iter()
            .filter(|(path, _, _)| !now.contains(path.as_path()))
            .map(|(path, _, _)| path.clone())
            .collect();
        watched.changed(&present, &gone);
        *before = listing;
        let root = watched.root().map(Path::to_owned).unwrap_or_default();
        let mut found = Found {
            build: watched.build(),
            ..Found::default()
        };
        for file in files {
            let shown = relative_path(&root, file).map_or_else(
                || file.to_string_lossy().replace('\\', "/"),
                |r| r.to_string(),
            );
            for problem in watched.problems(file) {
                if !problem.error {
                    found.warnings += 1;
                    continue;
                }
                found.errors += 1;
                if found.lines.len() < LIMIT {
                    found.lines.push(format!(
                        "{shown}:{}: [{}] {}",
                        problem.line, problem.code, problem.message
                    ));
                }
            }
        }
        found
    }
}

/// The files of the project of `config` a check reads, as the MCP server
/// lists them; nothing when its content model doesn't load.
fn listing(config: &Path) -> Vec<Stamp> {
    let Ok(model) = Project::load_model(config) else {
        return Vec::new();
    };
    let layout = ascribe_resolve::Layout::from_model(&model.model);
    let root = normalize(&std::path::absolute(&model.root).unwrap_or(model.root));
    let mut files = content_listing(&root, &layout.content_root, &layout.output_dir);
    files.sort();
    files
}
