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
//! server does (`crate::mcp::cache`). It listens on a port of `127.0.0.1` and writes
//! the port, with a token every request must carry, to a file in the user's
//! cache folder named for the project. It stops after [`IDLE`] without a
//! request, when the project's `ascribe.toml` is gone, or when another
//! server has taken its place. `ASCRIBE_HOOK_SERVER=off` checks in the
//! hook's own process instead.

use std::collections::HashSet;
use std::hash::{BuildHasher, Hash, Hasher};
use std::io::{self, BufRead, BufReader, Cursor, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
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

/// No check server can be started: there's no cache folder for its file,
/// or the binary can't be run again.
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
    let place = Rendezvous::of(config).ok_or(NoServer)?;
    let mut stale = None;
    let mut started = false;
    loop {
        match place.read() {
            Some(found) if found.version == VERSION && Some(&found) != stale.as_ref() => {
                match request(&found, ask, deadline) {
                    Some(answer) => return Ok(Some(answer)),
                    None if Instant::now() >= deadline => return Ok(None),
                    // Not listening: a server that has stopped.
                    None => stale = Some(found),
                }
            }
            found => {
                if !started {
                    stale = found;
                    start_server(config).ok_or(NoServer)?;
                    started = true;
                }
            }
        }
        if Instant::now() >= deadline {
            return Ok(None);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Sends `ask` to the server `at` and reads its answer, by `deadline`.
fn request(at: &Listening, ask: &Ask, deadline: Instant) -> Option<Result<Found, String>> {
    let left = |deadline: Instant| {
        deadline
            .checked_duration_since(Instant::now())
            .filter(|d| !d.is_zero())
    };
    let address = (Ipv4Addr::LOCALHOST, at.port).into();
    let mut stream = TcpStream::connect_timeout(&address, left(deadline)?).ok()?;
    stream.set_read_timeout(Some(left(deadline)?)).ok()?;
    stream.set_write_timeout(Some(left(deadline)?)).ok()?;
    let line = serde_json::to_string(&Request {
        token: at.token.clone(),
        ask: ask.clone(),
    })
    .ok()?;
    writeln!(stream, "{line}").ok()?;
    let mut answer = String::new();
    BufReader::new(stream).read_line(&mut answer).ok()?;
    serde_json::from_str::<Answer>(&answer).ok().map(|a| a.0)
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

/// The binary's version: a server of another version isn't asked.
const VERSION: &str = env!("ASCRIBE_VERSION");

/// A request: the server's token, and what to check.
#[derive(Serialize, Deserialize)]
struct Request {
    token: String,
    ask: Ask,
}

/// A server's answer.
#[derive(Serialize, Deserialize)]
struct Answer(Result<Found, String>);

/// Where a server listens, as its file says.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Listening {
    version: String,
    pid: u32,
    port: u16,
    token: String,
}

/// The file a project's server writes where it listens to.
struct Rendezvous {
    path: PathBuf,
}

impl Rendezvous {
    /// The file for the project of `config`, in the user's cache folder.
    fn of(config: &Path) -> Option<Rendezvous> {
        let config = normalize(&std::path::absolute(config).ok()?);
        // The same binary hashes it the same way, which is all a name
        // needs; `DefaultHasher::new()` has fixed keys.
        let mut hasher = std::hash::DefaultHasher::new();
        config.hash(&mut hasher);
        let dir = ascribe_sources::cache_root()?.join("hooks");
        Some(Rendezvous {
            path: dir.join(format!("{:016x}.json", hasher.finish())),
        })
    }

    fn read(&self) -> Option<Listening> {
        // Outside FileSystem: the check server's own file, in the user's
        // cache folder.
        let text = std::fs::read_to_string(&self.path).ok()?;
        serde_json::from_str(&text).ok()
    }

    /// Writes `listening`, whole or not at all: through a file beside it,
    /// renamed into place.
    fn write(&self, listening: &Listening) -> io::Result<()> {
        let dir = self.path.parent().unwrap_or(Path::new("."));
        std::fs::create_dir_all(dir)?;
        let partial = self.path.with_extension(format!("{}.tmp", listening.pid));
        let text = serde_json::to_string(listening).map_err(io::Error::other)?;
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        options.open(&partial)?.write_all(text.as_bytes())?;
        std::fs::rename(&partial, &self.path)
    }
}

/// A token no other process can guess: the hasher's keys come from the
/// operating system's randomness.
fn token() -> String {
    let random = |n: u32| {
        std::collections::hash_map::RandomState::new().hash_one((
            n,
            std::process::id(),
            Instant::now(),
        ))
    };
    format!("{:016x}{:016x}", random(1), random(2))
}

/// Runs the check server for the project of `config` until it's idle for
/// `idle`, the project's `ascribe.toml` is gone, or another server has
/// taken its place.
///
/// # Errors
///
/// It can't listen, or can't write where it listens.
pub fn serve(config: &Path, idle: Duration) -> io::Result<()> {
    let config = normalize(&std::path::absolute(config)?);
    let place = Rendezvous::of(&config).ok_or_else(|| io::Error::other("no cache folder"))?;
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
    let me = Listening {
        version: VERSION.to_owned(),
        pid: std::process::id(),
        port: listener.local_addr()?.port(),
        token: token(),
    };
    place.write(&me)?;
    // Connections are accepted on a thread of their own, and answered on
    // this one, which keeps the projects: they aren't `Send`.
    let (send, receive) = mpsc::channel();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            if send.send(stream).is_err() {
                break;
            }
        }
    });
    let mut kept = Kept {
        cache: crate::mcp::Cache::default(),
        watched: None,
    };
    let mut last = Instant::now();
    loop {
        match receive.recv_timeout(Duration::from_secs(1)) {
            Ok(stream) => {
                answer(&mut kept, &config, &me.token, stream);
                last = Instant::now();
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        if last.elapsed() >= idle || !config.is_file() || place.read().as_ref() != Some(&me) {
            break;
        }
    }
    if place.read().as_ref() == Some(&me) {
        let _ = std::fs::remove_file(&place.path);
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

/// Reads one request from `stream` and writes the answer.
fn answer(kept: &mut Kept, config: &Path, token: &str, stream: TcpStream) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return;
    }
    let Ok(request) = serde_json::from_str::<Request>(&line) else {
        return;
    };
    if request.token != token {
        return;
    }
    let found = kept.check(config, &request.ask);
    if let Ok(text) = serde_json::to_string(&Answer(found)) {
        let mut stream = &stream;
        let _ = writeln!(stream, "{text}");
    }
}
