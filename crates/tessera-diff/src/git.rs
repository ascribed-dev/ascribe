//! Running `git`: finding the repository, the base revision, and reading a
//! revision's tree and files.
//!
//! Every call runs the `git` executable with a fixed argument list (never
//! through a shell), with `core.quotepath` off so paths come back as they
//! are, and `-z` wherever a listing is parsed. No git library is linked.

use std::collections::BTreeMap;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tessera_core::RelPath;

use crate::DiffError;

/// A git repository, as seen from a directory inside it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repository {
    /// The directory `git` runs in: the project root.
    dir: PathBuf,
    /// The repository's top-level directory, as `git` prints it (with `/`
    /// separators on every platform).
    pub root: String,
    /// The path from the repository's root to `dir`, `/`-separated, with a
    /// trailing `/` unless it's empty (`git rev-parse --show-prefix`).
    pub prefix: String,
}

/// The revision a comparison starts from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Base {
    /// The revision asked for, or the default branch found.
    pub requested: String,
    /// The commit `requested` names.
    pub commit: String,
    /// The commit compared against: the merge base of `requested` and
    /// `HEAD`, or `commit` itself when the base is exact.
    pub merge_base: Option<String>,
}

impl Base {
    /// The commit whose files the comparison reads.
    pub fn compared(&self) -> &str {
        self.merge_base.as_deref().unwrap_or(&self.commit)
    }
}

impl Repository {
    /// The repository `dir` is in.
    ///
    /// # Errors
    ///
    /// [`DiffError::GitNotFound`] when `git` can't be run, and
    /// [`DiffError::NotARepository`] when `dir` isn't in a repository.
    pub fn discover(dir: &Path) -> Result<Repository, DiffError> {
        let probe = Repository {
            dir: dir.to_path_buf(),
            root: String::new(),
            prefix: String::new(),
        };
        let out = probe
            .run(&["rev-parse", "--show-toplevel", "--show-prefix"])
            .map_err(|e| match e {
                DiffError::Git { message, .. } => DiffError::NotARepository {
                    dir: dir.display().to_string(),
                    message,
                },
                other => other,
            })?;
        let text = String::from_utf8_lossy(&out);
        let mut lines = text.lines();
        let root = lines.next().unwrap_or_default().to_owned();
        let prefix = lines.next().unwrap_or_default().to_owned();
        Ok(Repository {
            dir: dir.to_path_buf(),
            root,
            prefix,
        })
    }

    /// The project's folder inside the repository.
    pub fn project_dir(&self) -> RelPath {
        RelPath::parse(&self.prefix).unwrap_or_default()
    }

    /// The base to compare against. `requested` is a revision; without one,
    /// the repository's default branch: `origin/HEAD`, then `main`, then
    /// `master`. Unless `exact`, the comparison starts from the merge base of
    /// the revision and `HEAD`, as a pull request shows it.
    ///
    /// # Errors
    ///
    /// [`DiffError::UnknownRevision`] when the revision doesn't name a commit,
    /// [`DiffError::NoDefaultBranch`] when none of the defaults exists, and
    /// [`DiffError::Git`] when there is no merge base.
    pub fn base(&self, requested: Option<&str>, exact: bool) -> Result<Base, DiffError> {
        let (requested, commit) = match requested {
            Some(rev) => {
                let commit = self
                    .commit_of(rev)
                    .ok_or_else(|| DiffError::UnknownRevision(rev.to_owned()))?;
                (rev.to_owned(), commit)
            }
            None => ["origin/HEAD", "main", "master"]
                .iter()
                .find_map(|rev| self.commit_of(rev).map(|c| ((*rev).to_owned(), c)))
                .ok_or(DiffError::NoDefaultBranch)?,
        };
        let merge_base = if exact {
            None
        } else {
            let out = self.run(&["merge-base", &commit, "HEAD"])?;
            Some(String::from_utf8_lossy(&out).trim().to_owned())
        };
        Ok(Base {
            requested,
            commit,
            merge_base,
        })
    }

    /// The commit a revision names, or `None`.
    fn commit_of(&self, rev: &str) -> Option<String> {
        // `--end-of-options` keeps a revision that starts with `-` from being
        // read as an option.
        let spec = format!("{rev}^{{commit}}");
        let out = self
            .run(&[
                "rev-parse",
                "--verify",
                "--quiet",
                "--end-of-options",
                &spec,
            ])
            .ok()?;
        let commit = String::from_utf8_lossy(&out).trim().to_owned();
        (!commit.is_empty()).then_some(commit)
    }

    /// Every file of a commit's tree, by its path from the repository's root.
    ///
    /// # Errors
    ///
    /// [`DiffError::Git`] when the tree can't be listed.
    pub fn tree(&self, commit: &str) -> Result<Tree, DiffError> {
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
            // A submodule is a commit, not a file of this repository.
            if kind != "blob" {
                continue;
            }
            // A path that isn't UTF-8 can't be a content path.
            let Ok(path) = std::str::from_utf8(&entry[tab + 1..]) else {
                continue;
            };
            let Ok(path) = RelPath::parse(path) else {
                continue;
            };
            files.insert(
                path,
                TreeEntry {
                    object: object.to_owned(),
                    symlink: mode == "120000",
                },
            );
        }
        Ok(Tree { files })
    }

    /// The contents of several blobs, read through one `git cat-file --batch`.
    ///
    /// # Errors
    ///
    /// [`DiffError::Git`] when `git` fails or its output can't be read.
    pub fn read_blobs(&self, objects: &[&str]) -> Result<Vec<Vec<u8>>, DiffError> {
        let args = ["cat-file", "--batch"];
        if objects.is_empty() {
            return Ok(Vec::new());
        }
        let mut child = self
            .command(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| self.spawn_error(&args, e))?;
        let failed = |message: String| DiffError::Git {
            command: command_line(&args),
            message,
        };
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| failed("no standard input".into()))?;
        let requests: Vec<u8> = objects
            .iter()
            .flat_map(|o| format!("{o}\n").into_bytes())
            .collect();
        // Written from another thread: `git` writes each answer as it reads
        // each request, and a full pipe on both sides would block forever.
        let writer = std::thread::spawn(move || stdin.write_all(&requests));
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| failed("no standard output".into()))?;
        let mut reader = BufReader::new(stdout);
        let mut out = Vec::with_capacity(objects.len());
        for object in objects {
            out.push(read_batch_entry(&mut reader, object).map_err(|e| failed(e.to_string()))?);
        }
        drop(reader);
        let _ = writer.join();
        let _ = child.wait();
        Ok(out)
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new("git");
        command
            .current_dir(&self.dir)
            .args(["-c", "core.quotepath=off"])
            .args(args);
        command
    }

    /// Runs `git` and returns its standard output.
    fn run(&self, args: &[&str]) -> Result<Vec<u8>, DiffError> {
        let out = self
            .command(args)
            .stdin(Stdio::null())
            .output()
            .map_err(|e| self.spawn_error(args, e))?;
        if out.status.success() {
            Ok(out.stdout)
        } else {
            Err(DiffError::Git {
                command: command_line(args),
                message: String::from_utf8_lossy(&out.stderr).trim().to_owned(),
            })
        }
    }

    fn spawn_error(&self, args: &[&str], e: io::Error) -> DiffError {
        if e.kind() == io::ErrorKind::NotFound {
            DiffError::GitNotFound
        } else {
            DiffError::Git {
                command: command_line(args),
                message: e.to_string(),
            }
        }
    }
}

fn command_line(args: &[&str]) -> String {
    format!("git {}", args.join(" "))
}

/// One answer of `git cat-file --batch`: `<object> blob <size>`, the
/// contents, and a newline.
fn read_batch_entry(reader: &mut impl BufRead, object: &str) -> io::Result<Vec<u8>> {
    let mut header = String::new();
    reader.read_line(&mut header)?;
    let header = header.trim_end();
    let size = match header.rsplit_once(' ') {
        Some((_, size)) if !header.ends_with(" missing") => size
            .parse::<usize>()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, header.to_owned()))?,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("object {object} is missing"),
            ));
        }
    };
    let mut contents = vec![0; size];
    reader.read_exact(&mut contents)?;
    let mut newline = [0u8; 1];
    reader.read_exact(&mut newline)?;
    Ok(contents)
}

/// The files of a commit's tree.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tree {
    /// Every file, by its path from the repository's root.
    pub files: BTreeMap<RelPath, TreeEntry>,
}

/// A file in a [`Tree`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeEntry {
    /// The blob's object name.
    pub object: String,
    /// Whether it's a symbolic link (its contents are the link's target).
    pub symlink: bool,
}
