//! Output ownership (`project-docs/contracts/output-layout.md`): staging,
//! the manifest, and replacing a previous build's output without ever
//! touching a file Tessera didn't write.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tessera_core::RelPath;

/// The manifest's `format`.
const MANIFEST_FORMAT: &str = "tessera-manifest";
/// The manifest format's version.
const MANIFEST_VERSION: u32 = 1;
const LOCK_FILE: &str = ".lock";
const STAGING_DIR: &str = ".staging";

/// What a file in an emitter root is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileKind {
    /// A page.
    Page,
    /// A copy of a file a page refers to.
    Asset,
    /// Anything else Tessera writes, such as a schema.
    Generated,
}

/// What goes in a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Contents {
    /// Text the emitter rendered.
    Text(String),
    /// A copy, byte for byte, of the file at this path on disk.
    Copy(PathBuf),
}

/// A file an emitter wants in its root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmittedFile {
    /// The path inside the emitter root.
    pub path: RelPath,
    /// What it is.
    pub kind: FileKind,
    /// For a page or asset, its source path relative to the content root.
    pub source: Option<RelPath>,
    /// For an asset the consumer must serve, the URL pages use for it.
    pub url: Option<String>,
    /// What's in it.
    pub contents: Contents,
}

/// Why an output couldn't be replaced.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// Another build holds the lock.
    #[error("another ascribe build is writing to {dir}")]
    Locked {
        /// The output directory.
        dir: String,
    },
    /// A file is at a manifest's path but isn't one, so Tessera can't tell
    /// what it owns.
    #[error(
        "{path} isn't a Tessera manifest, so Tessera can't tell which files in its directory it owns; move or remove it"
    )]
    NotManifest {
        /// The path.
        path: String,
    },
    /// A manifest of a version this Tessera doesn't know.
    #[error("{path} has manifest version {version}, which this tessera doesn't know")]
    UnknownVersion {
        /// The path.
        path: String,
        /// The version it has.
        version: u32,
    },
    /// A manifest lists a path that can't be inside the emitter root.
    #[error("{path} lists `{entry}`, which isn't a path inside its output")]
    BadEntry {
        /// The manifest's path.
        path: String,
        /// The entry.
        entry: String,
    },
    /// Files that would be overwritten or can't be created, or paths that
    /// collide.
    #[error("the output can't be written:\n{}", .0.iter().map(|c| format!("  - {c}")).collect::<Vec<_>>().join("\n"))]
    Conflicts(Vec<String>),
    /// A file system operation failed.
    #[error("{action} {path}: {source}")]
    Io {
        /// What was being done.
        action: &'static str,
        /// The path.
        path: String,
        /// The error.
        source: io::Error,
    },
}

fn io_err(action: &'static str, path: &Path) -> impl FnOnce(io::Error) -> StoreError {
    let path = path.display().to_string();
    move |source| StoreError::Io {
        action,
        path,
        source,
    }
}

/// What a replacement did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Replaced {
    /// Files written that weren't there, or were different.
    pub written: usize,
    /// Files left as they were because their content hasn't changed.
    pub unchanged: usize,
    /// Files of the previous output that this build doesn't produce, removed.
    pub removed: usize,
}

#[derive(Serialize, Deserialize)]
struct Manifest {
    format: String,
    version: u32,
    build: String,
    emitter: String,
    files: Vec<ManifestFile>,
}

#[derive(Clone, Serialize, Deserialize)]
struct ManifestFile {
    path: String,
    kind: FileKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

/// The output directory, locked for one build (output-layout contract, §4,
/// step 1). The lock is released when this is dropped, or by the operating
/// system if the process dies.
#[derive(Debug)]
pub struct OutputDir {
    root: PathBuf,
    // Held for its lock.
    _lock: File,
}

impl OutputDir {
    /// Takes an exclusive advisory lock on `<root>/.lock`, creating the
    /// directory and the file if needed.
    ///
    /// # Errors
    ///
    /// [`StoreError::Locked`] when another build holds it.
    pub fn lock(root: &Path) -> Result<OutputDir, StoreError> {
        fs::create_dir_all(root).map_err(io_err("can't create", root))?;
        let lock_path = root.join(LOCK_FILE);
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&lock_path)
            .map_err(io_err("can't open", &lock_path))?;
        match file.try_lock() {
            Ok(()) => Ok(OutputDir {
                root: root.to_owned(),
                _lock: file,
            }),
            Err(TryLockError::WouldBlock) => Err(StoreError::Locked {
                dir: root.display().to_string(),
            }),
            Err(TryLockError::Error(e)) => Err(io_err("can't lock", &lock_path)(e)),
        }
    }

    /// The output directory.
    pub fn path(&self) -> &Path {
        &self.root
    }

    /// The emitter root for a build and emitter.
    pub fn emitter_root(&self, build: &str, emitter: &str) -> PathBuf {
        self.root.join(build).join(emitter)
    }

    /// The manifest of a build and emitter.
    pub fn manifest_path(&self, build: &str, emitter: &str) -> PathBuf {
        self.root
            .join(build)
            .join(format!("{emitter}.manifest.json"))
    }

    /// Replaces the output of one build and emitter with `files`
    /// (output-layout contract, §4, steps 2 to 9): stages them, checks that
    /// nothing that isn't Tessera's is in the way, records ownership, moves
    /// the files into place, removes what the previous output had and this
    /// one doesn't, and writes the final manifest.
    ///
    /// Nothing is touched when the check fails. A file is deleted only if the
    /// previous manifest lists it.
    ///
    /// # Errors
    ///
    /// A [`StoreError`]; the previous output is untouched unless a file
    /// system operation fails after the check.
    pub fn replace(
        &self,
        build: &str,
        emitter: &str,
        files: &[EmittedFile],
    ) -> Result<Replaced, StoreError> {
        for name in [build, emitter] {
            if name.is_empty() || name.starts_with('.') || name.contains(['/', '\\']) {
                return Err(StoreError::Conflicts(vec![format!(
                    "`{name}` can't be a directory name in the output"
                )]));
            }
        }
        let staging = self.root.join(STAGING_DIR).join(build).join(emitter);
        let result = self.replace_staged(build, emitter, files, &staging);
        // Step 9: clean up, whatever happened. A failure to clean up doesn't
        // change what the build did.
        let _ = fs::remove_dir_all(&staging);
        let _ = fs::remove_dir(self.root.join(STAGING_DIR).join(build));
        let _ = fs::remove_dir(self.root.join(STAGING_DIR));
        result
    }

    fn replace_staged(
        &self,
        build: &str,
        emitter: &str,
        files: &[EmittedFile],
        staging: &Path,
    ) -> Result<Replaced, StoreError> {
        let manifest_path = self.manifest_path(build, emitter);
        let emitter_root = self.emitter_root(build, emitter);

        // Step 2: the previous manifest.
        let previous = read_manifest(&manifest_path)?;
        let previous_paths: BTreeSet<&str> = previous.iter().map(|f| f.path.as_str()).collect();

        // Step 4, first part: two new files with one path, or paths that differ
        // only in case, would be one file on macOS and Windows.
        let mut problems = Vec::new();
        let mut seen: BTreeMap<String, &RelPath> = BTreeMap::new();
        for file in files {
            if let Some(other) = seen.insert(file.path.as_str().to_lowercase(), &file.path) {
                problems.push(if other == &file.path {
                    format!("two files would be written at {}", file.path)
                } else {
                    format!(
                        "{other} and {} differ only in case, so they would be one file on some systems",
                        file.path
                    )
                });
            }
        }
        // Step 4, second part: nothing that isn't Tessera's is in the way.
        let mut blockers: BTreeSet<PathBuf> = BTreeSet::new();
        for dir in [self.root.join(build), emitter_root.clone()] {
            if fs::symlink_metadata(&dir).is_ok() && !dir.is_dir() {
                problems.push(format!(
                    "{} is a file, and a directory is needed there",
                    dir.display()
                ));
            }
        }
        for file in files {
            let dest = emitter_root.join(file.path.as_str());
            if fs::symlink_metadata(&dest).is_ok() {
                if dest.is_dir() {
                    // Resolved Q117: a directory where this build writes a file.
                    problems.push(format!(
                        "{} is a directory, and a file would be written there",
                        dest.display()
                    ));
                } else if !previous_paths.contains(file.path.as_str()) {
                    problems.push(format!(
                        "{} exists and isn't a file Tessera wrote; move or remove it",
                        dest.display()
                    ));
                }
            }
            let mut prefix = String::new();
            let segments: Vec<&str> = file.path.segments().collect();
            for segment in &segments[..segments.len().saturating_sub(1)] {
                if !prefix.is_empty() {
                    prefix.push('/');
                }
                prefix.push_str(segment);
                let at = emitter_root.join(&prefix);
                if fs::symlink_metadata(&at).is_ok() && !at.is_dir() {
                    // Resolved Q117: a file of Tessera's where this build
                    // needs a directory.
                    if previous_paths.contains(prefix.as_str()) {
                        // Tessera's own file, which this build no longer
                        // produces as a file: it's removed before the
                        // directory is made.
                        blockers.insert(at);
                    } else {
                        problems.push(format!(
                            "{} is a file that isn't Tessera's, and a directory is needed there",
                            at.display()
                        ));
                    }
                }
            }
        }
        if !problems.is_empty() {
            problems.sort();
            problems.dedup();
            return Err(StoreError::Conflicts(problems));
        }

        // Step 3: emit into staging.
        if staging.exists() {
            fs::remove_dir_all(staging).map_err(io_err("can't remove", staging))?;
        }
        fs::create_dir_all(staging).map_err(io_err("can't create", staging))?;
        for file in files {
            let dest = staging.join(file.path.as_str());
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).map_err(io_err("can't create", parent))?;
            }
            match &file.contents {
                Contents::Text(text) => {
                    fs::write(&dest, text).map_err(io_err("can't write", &dest))?;
                }
                Contents::Copy(from) => {
                    fs::copy(from, &dest).map_err(io_err("can't copy", from))?;
                }
            }
        }

        // Step 5: record ownership first: the previous files and the new ones.
        let new_entries: Vec<ManifestFile> = files
            .iter()
            .map(|f| ManifestFile {
                path: f.path.to_string(),
                kind: f.kind,
                source: f.source.as_ref().map(ToString::to_string),
                url: f.url.clone(),
            })
            .collect();
        let mut union: BTreeMap<String, ManifestFile> = previous
            .iter()
            .map(|f| (f.path.clone(), f.clone()))
            .collect();
        union.extend(new_entries.iter().map(|f| (f.path.clone(), f.clone())));
        self.write_manifest(build, emitter, union.into_values().collect())?;

        // Step 6: move the new files into place.
        for blocker in &blockers {
            fs::remove_file(blocker).map_err(io_err("can't remove", blocker))?;
        }
        let mut replaced = Replaced::default();
        for file in files {
            let staged = staging.join(file.path.as_str());
            let dest = emitter_root.join(file.path.as_str());
            if same_content(&staged, &dest) {
                replaced.unchanged += 1;
                continue;
            }
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).map_err(io_err("can't create", parent))?;
            }
            fs::rename(&staged, &dest).map_err(io_err("can't move into place", &dest))?;
            replaced.written += 1;
        }

        // Step 7: remove what the previous output had and this one doesn't.
        let produced: BTreeSet<&str> = files.iter().map(|f| f.path.as_str()).collect();
        for old in previous
            .iter()
            .filter(|f| !produced.contains(f.path.as_str()))
        {
            let path = emitter_root.join(&old.path);
            match fs::symlink_metadata(&path) {
                Ok(meta) if !meta.is_dir() => {
                    fs::remove_file(&path).map_err(io_err("can't remove", &path))?;
                    replaced.removed += 1;
                }
                Ok(_) | Err(_) => {}
            }
            // A directory this emptied goes too, up to the emitter root.
            let mut dir = path.parent().map(Path::to_owned);
            while let Some(d) = dir {
                if d == emitter_root || fs::remove_dir(&d).is_err() {
                    break;
                }
                dir = d.parent().map(Path::to_owned);
            }
        }

        // Step 8: the final manifest lists only this build's files.
        self.write_manifest(build, emitter, new_entries)?;
        Ok(replaced)
    }

    /// Writes a manifest by writing a temporary file in `.staging/` and
    /// renaming it into place, so it's never half-written.
    fn write_manifest(
        &self,
        build: &str,
        emitter: &str,
        mut files: Vec<ManifestFile>,
    ) -> Result<(), StoreError> {
        files.sort_by(|a, b| a.path.cmp(&b.path));
        let manifest = Manifest {
            format: MANIFEST_FORMAT.to_owned(),
            version: MANIFEST_VERSION,
            build: build.to_owned(),
            emitter: emitter.to_owned(),
            files,
        };
        let mut text = serde_json::to_string_pretty(&manifest).map_err(|e| StoreError::Io {
            action: "can't serialize the manifest",
            path: emitter.to_owned(),
            source: io::Error::other(e),
        })?;
        text.push('\n');
        let staging = self.root.join(STAGING_DIR);
        fs::create_dir_all(&staging).map_err(io_err("can't create", &staging))?;
        let temp = staging.join(format!(".manifest-{build}-{emitter}.tmp"));
        fs::write(&temp, text).map_err(io_err("can't write", &temp))?;
        let dest = self.manifest_path(build, emitter);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(io_err("can't create", parent))?;
        }
        fs::rename(&temp, &dest).map_err(io_err("can't move into place", &dest))
    }
}

/// Reads the manifest at `path`. No file means no previous output. A file that
/// isn't a manifest is an error (output-layout contract, §4, step 2).
fn read_manifest(path: &Path) -> Result<Vec<ManifestFile>, StoreError> {
    let shown = path.display().to_string();
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(_) if path.is_dir() => return Err(StoreError::NotManifest { path: shown }),
        Err(e) => return Err(io_err("can't read", path)(e)),
    };
    let manifest: Manifest = match serde_json::from_slice(&bytes) {
        Ok(m) => m,
        Err(_) => return Err(StoreError::NotManifest { path: shown }),
    };
    if manifest.format != MANIFEST_FORMAT {
        return Err(StoreError::NotManifest { path: shown });
    }
    if manifest.version != MANIFEST_VERSION {
        return Err(StoreError::UnknownVersion {
            path: shown,
            version: manifest.version,
        });
    }
    // Tessera deletes what a manifest lists, so an entry that could reach
    // outside the emitter root is refused, not followed.
    for entry in &manifest.files {
        let inside = RelPath::parse(&entry.path)
            .is_ok_and(|p| !p.is_root() && p.up_count() == 0 && p.as_str() == entry.path);
        if !inside || entry.path.contains('\\') {
            return Err(StoreError::BadEntry {
                path: shown,
                entry: entry.path.clone(),
            });
        }
    }
    Ok(manifest.files)
}

/// Whether the file at `dest` exists with the same bytes as `staged`.
fn same_content(staged: &Path, dest: &Path) -> bool {
    let (Ok(a), Ok(b)) = (fs::metadata(staged), fs::metadata(dest)) else {
        return false;
    };
    if !b.is_file() || a.len() != b.len() {
        return false;
    }
    matches!((fs::read(staged), fs::read(dest)), (Ok(x), Ok(y)) if x == y)
}
