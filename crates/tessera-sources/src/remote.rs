//! Running `git` against another repository: the only code in Ascribe that
//! does (decision 14 of the docs plan, SPEC §7.4).
//!
//! Each repository is fetched into a bare repository of its own in a cache
//! outside the project, one per URL, so a second run fetches only what's
//! new. Commits come without their files (`--filter=blob:none`), and only
//! the files asked for are fetched after them.
//!
//! Every call runs the `git` executable with a fixed argument list, never
//! through a shell, and with settings that keep a repository someone else
//! controls from doing more than send objects: no hooks, no submodules, only
//! the usual transports, and no prompt, so a missing credential fails at
//! once. Credentials are `git`'s own: whatever lets `git fetch <url>` work in
//! the shell lets Ascribe fetch.

use std::collections::BTreeMap;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tessera_model::file_hash;

use crate::SourcesError;

/// The settings every call runs with.
const SETTINGS: &[&str] = &[
    // Only these transports: not `ext::`, which runs a command.
    "protocol.allow=never",
    "protocol.https.allow=always",
    "protocol.http.allow=always",
    "protocol.ssh.allow=always",
    "protocol.git.allow=always",
    "protocol.file.allow=always",
    "core.quotepath=off",
    "credential.interactive=false",
    "fetch.recurseSubmodules=false",
    "submodule.recurse=false",
    "core.fsmonitor=false",
    "gc.auto=0",
    "maintenance.auto=false",
];

/// Environment variables that would point `git` at another repository than
/// the cache's.
const CLEARED: &[&str] = &[
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_NAMESPACE",
    "GIT_COMMON_DIR",
];

/// The cache of fetched repositories.
#[derive(Clone, Debug)]
pub(crate) struct Cache {
    root: PathBuf,
}

/// One repository in the cache, for one source.
#[derive(Debug)]
pub(crate) struct Remote {
    dir: PathBuf,
    /// The source it's fetched for, to name in errors.
    source: String,
    /// Whether `ssh` should run in batch mode, so it can't prompt.
    batch_ssh: bool,
}

/// A file of a commit's tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TreeFile {
    pub object: String,
    /// `blob` with a mode of a regular file; anything else (a link, a
    /// submodule) isn't copied.
    pub regular: bool,
}

/// The commits between two pins.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Commits {
    /// How many.
    pub count: usize,
    /// The newest, newest first, at most 20.
    pub newest: Vec<CommitLine>,
}

/// Where a new pin is from the old one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum History {
    /// After it, by these commits.
    After(Commits),
    /// Not after it: moved back, or to another line of history.
    Back,
    /// The repository couldn't say.
    Unknown,
}

/// A commit, by its hash and the first line of its message.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct CommitLine {
    /// Its full hash.
    pub commit: String,
    /// The first line of its message.
    pub subject: String,
}

/// How many commits [`Remote::commits`] lists.
pub const NEWEST: usize = 20;

impl Cache {
    pub(crate) fn new(root: PathBuf) -> Cache {
        Cache { root }
    }

    /// The cache's repository for `url`, made when it's first needed.
    pub(crate) fn open(&self, source: &str, url: &str) -> Result<Remote, SourcesError> {
        let name: String = file_hash(url.as_bytes())
            .trim_start_matches("sha256:")
            .chars()
            .take(16)
            .collect();
        let dir = self.root.join("sources").join(format!("{name}.git"));
        let remote = Remote {
            dir,
            source: source.to_owned(),
            batch_ssh: false,
        };
        if !remote.dir.join("HEAD").is_file() {
            std::fs::create_dir_all(&remote.dir).map_err(|e| SourcesError::Cache {
                path: remote.dir.display().to_string(),
                message: e.to_string(),
            })?;
            remote.run(&["init", "--bare", "-q"])?;
        }
        // The URL is only ever the one ascribe.toml gives, and `--` keeps it
        // from being read as an option.
        remote.run(&["config", "--", "remote.origin.url", url])?;
        let batch_ssh = std::env::var_os("GIT_SSH_COMMAND").is_none()
            && std::env::var_os("GIT_SSH").is_none()
            && remote.run(&["config", "--get", "core.sshCommand"]).is_err();
        Ok(Remote {
            batch_ssh,
            ..remote
        })
    }
}

impl Remote {
    /// Fetches `rev` (a commit's hash, a branch, or `HEAD`) and returns the
    /// commit it names. With `shallow`, only that commit, not its history.
    pub(crate) fn fetch(&self, rev: &str, shallow: bool) -> Result<String, SourcesError> {
        let mut args = vec![
            "fetch",
            "--quiet",
            "--no-tags",
            "--no-recurse-submodules",
            "--filter=blob:none",
        ];
        if shallow {
            args.push("--depth=1");
        }
        args.extend(["origin", rev]);
        self.run(&args)?;
        let out = self.run(&["rev-parse", "--verify", "--quiet", "FETCH_HEAD^{commit}"])?;
        Ok(String::from_utf8_lossy(&out).trim().to_owned())
    }

    /// Whether the cache has `commit`, without fetching anything.
    pub(crate) fn has_commit(&self, commit: &str) -> bool {
        let spec = format!("{commit}^{{commit}}");
        self.command(&["cat-file", "-e", &spec])
            .env("GIT_NO_LAZY_FETCH", "1")
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    }

    /// Every file of a commit's tree, by path.
    pub(crate) fn tree(&self, commit: &str) -> Result<BTreeMap<String, TreeFile>, SourcesError> {
        let out = self.run(&["ls-tree", "-r", "-z", "--full-tree", commit])?;
        let mut files = BTreeMap::new();
        for entry in out.split(|b| *b == 0) {
            // `<mode> SP <type> SP <object> TAB <path>`
            let Some(tab) = entry.iter().position(|b| *b == b'\t') else {
                continue;
            };
            let head = String::from_utf8_lossy(&entry[..tab]);
            let mut parts = head.split(' ');
            let (Some(mode), Some(kind), Some(object)) = (parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            let Ok(path) = std::str::from_utf8(&entry[tab + 1..]) else {
                continue;
            };
            files.insert(
                path.to_owned(),
                TreeFile {
                    object: object.to_owned(),
                    regular: kind == "blob" && matches!(mode, "100644" | "100755"),
                },
            );
        }
        Ok(files)
    }

    /// Each object's size, then the contents of those no larger than
    /// `limit`, fetching the objects the cache doesn't have yet in one go
    /// first. `None` for an object over the limit.
    pub(crate) fn read(
        &self,
        objects: &[&str],
        limit: usize,
    ) -> Result<Vec<Option<Vec<u8>>>, SourcesError> {
        if objects.is_empty() {
            return Ok(Vec::new());
        }
        // One fetch for every missing file, rather than one each as `git`
        // would fetch them lazily. A host that won't send an object by name
        // still sends it lazily below.
        let mut args = vec![
            "fetch",
            "--quiet",
            "--no-tags",
            "--no-recurse-submodules",
            "--no-write-fetch-head",
            "--filter=blob:none",
            "origin",
        ];
        let missing: Vec<&str> = objects
            .iter()
            .copied()
            .filter(|o| !self.has_object(o))
            .collect();
        if !missing.is_empty() {
            args.extend(&missing);
            let _ = self.run(&args);
        }
        let sizes = self.batch("--batch-check", objects, |reader, object| {
            let mut header = String::new();
            reader.read_line(&mut header)?;
            parse_size(&header, object)
        })?;
        let wanted: Vec<&str> = objects
            .iter()
            .zip(&sizes)
            .filter(|(_, size)| **size <= limit)
            .map(|(o, _)| *o)
            .collect();
        let mut contents = self
            .batch("--batch", &wanted, |reader, object| {
                let mut header = String::new();
                reader.read_line(&mut header)?;
                let size = parse_size(&header, object)?;
                let mut bytes = vec![0; size];
                reader.read_exact(&mut bytes)?;
                let mut newline = [0u8; 1];
                reader.read_exact(&mut newline)?;
                Ok(bytes)
            })?
            .into_iter();
        Ok(sizes
            .iter()
            .map(|size| {
                if *size <= limit {
                    contents.next()
                } else {
                    None
                }
            })
            .collect())
    }

    /// Where `to` is from `from`: the commits after `from` up to `to` (how
    /// many, and the newest), or that `to` isn't after `from` at all.
    /// `Unknown` when the cache can't tell, as when `from` is gone from the
    /// repository.
    ///
    /// Pins are fetched without their history, so the cache may hold each
    /// commit with nothing between them, and counting there counts only
    /// what it holds. The history is fetched first, with commits and trees
    /// and no files.
    pub(crate) fn history(&self, from: &str, to: &str) -> History {
        if !self.has_commit(from) && self.fetch(from, true).is_err() {
            return History::Unknown;
        }
        if self.is_shallow() && self.unshallow(to).is_err() {
            return History::Unknown;
        }
        let after = self
            .command(&["merge-base", "--is-ancestor", from, to])
            .stderr(Stdio::null())
            .status();
        match after.map(|s| s.code()) {
            Ok(Some(0)) => {}
            Ok(Some(1)) => return History::Back,
            _ => return History::Unknown,
        }
        self.commits(from, to)
            .map_or(History::Unknown, History::After)
    }

    fn commits(&self, from: &str, to: &str) -> Option<Commits> {
        let range = format!("{from}..{to}");
        let count = self.run(&["rev-list", "--count", &range]).ok()?;
        let count = String::from_utf8_lossy(&count).trim().parse().ok()?;
        let limit = format!("-n{NEWEST}");
        let log = self
            .run(&["log", &limit, "--format=%H%x00%s", &range, "--"])
            .ok()?;
        let newest = String::from_utf8_lossy(&log)
            .lines()
            .filter_map(|line| line.split_once('\0'))
            .map(|(commit, subject)| CommitLine {
                commit: commit.to_owned(),
                subject: subject.to_owned(),
            })
            .collect();
        Some(Commits { count, newest })
    }

    fn is_shallow(&self) -> bool {
        self.run(&["rev-parse", "--is-shallow-repository"])
            .is_ok_and(|out| out.trim_ascii() == b"true")
    }

    /// Fetches the whole history of `rev`, without files.
    fn unshallow(&self, rev: &str) -> Result<(), SourcesError> {
        self.run(&[
            "fetch",
            "--quiet",
            "--no-tags",
            "--no-recurse-submodules",
            "--filter=blob:none",
            "--unshallow",
            "origin",
            rev,
        ])
        .map(drop)
    }

    fn has_object(&self, object: &str) -> bool {
        self.command(&["cat-file", "-e", object])
            .env("GIT_NO_LAZY_FETCH", "1")
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    }

    /// Runs `git cat-file <mode>` with one request per object, reading each
    /// answer with `read`.
    fn batch<T>(
        &self,
        mode: &str,
        objects: &[&str],
        read: impl Fn(&mut BufReader<std::process::ChildStdout>, &str) -> io::Result<T>,
    ) -> Result<Vec<T>, SourcesError> {
        if objects.is_empty() {
            return Ok(Vec::new());
        }
        let args = ["cat-file", mode];
        let failed = |message: String| SourcesError::Git {
            name: self.source.clone(),
            message,
        };
        let mut child = self
            .command(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| self.spawn_error(e))?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| failed("no standard input".into()))?;
        let requests: Vec<u8> = objects
            .iter()
            .flat_map(|o| format!("{o}\n").into_bytes())
            .collect();
        // Written from another thread: `git` answers as it reads, and a full
        // pipe on both sides would block forever.
        let writer = std::thread::spawn(move || stdin.write_all(&requests));
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| failed("no standard output".into()))?;
        let mut reader = BufReader::new(stdout);
        let mut out = Vec::with_capacity(objects.len());
        let mut error = None;
        for object in objects {
            match read(&mut reader, object) {
                Ok(item) => out.push(item),
                Err(e) => {
                    error = Some(e);
                    break;
                }
            }
        }
        drop(reader);
        let _ = writer.join();
        let output = child
            .wait_with_output()
            .map_err(|e| failed(e.to_string()))?;
        match error {
            None => Ok(out),
            Some(e) => {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
                Err(failed(if stderr.is_empty() {
                    e.to_string()
                } else {
                    stderr
                }))
            }
        }
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new("git");
        for setting in SETTINGS {
            command.args(["-c", setting]);
        }
        // A hooks folder that doesn't exist: the cache runs no hooks.
        let hooks = format!("core.hooksPath={}", self.dir.join("no-hooks").display());
        command
            .args(["-c", &hooks])
            .args(args)
            .current_dir(&self.dir)
            .env("GIT_DIR", &self.dir)
            .env("GIT_TERMINAL_PROMPT", "0")
            .stdin(Stdio::null());
        for name in CLEARED {
            command.env_remove(name);
        }
        if self.batch_ssh {
            command.env("GIT_SSH_COMMAND", "ssh -o BatchMode=yes");
        }
        command
    }

    /// Runs `git` and returns its standard output, or its own message.
    fn run(&self, args: &[&str]) -> Result<Vec<u8>, SourcesError> {
        let out = self
            .command(args)
            .output()
            .map_err(|e| self.spawn_error(e))?;
        if out.status.success() {
            Ok(out.stdout)
        } else {
            Err(SourcesError::Git {
                name: self.source.clone(),
                message: String::from_utf8_lossy(&out.stderr).trim().to_owned(),
            })
        }
    }

    fn spawn_error(&self, e: io::Error) -> SourcesError {
        if e.kind() == io::ErrorKind::NotFound {
            SourcesError::GitNotFound
        } else {
            SourcesError::Git {
                name: self.source.clone(),
                message: e.to_string(),
            }
        }
    }
}

/// The size in a `cat-file` header: `<object> blob <size>`.
fn parse_size(header: &str, object: &str) -> io::Result<usize> {
    let header = header.trim_end();
    match header.rsplit_once(' ') {
        Some((_, size)) if !header.ends_with(" missing") => size
            .parse::<usize>()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, header.to_owned())),
        _ => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("object {object} is missing"),
        )),
    }
}

/// The folder the cache is in: `ASCRIBE_CACHE_DIR`, or `ascribe` in the
/// user's cache folder.
pub(crate) fn cache_root() -> Option<PathBuf> {
    let var = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty());
    if let Some(dir) = var("ASCRIBE_CACHE_DIR") {
        return Some(PathBuf::from(dir));
    }
    let base = if cfg!(windows) {
        var("LOCALAPPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        var("HOME").map(|home| Path::new(&home).join("Library").join("Caches"))
    } else {
        var("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| var("HOME").map(|home| Path::new(&home).join(".cache")))
    };
    base.map(|dir| dir.join("ascribe"))
}
