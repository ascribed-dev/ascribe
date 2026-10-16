//! Output ownership: staging,
//! the manifest, and replacing a previous build's output without ever
//! touching a file Ascribe didn't write.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io;
use std::path::{Component, Path, PathBuf};

use ascribe_core::RelPath;
use serde::{Deserialize, Serialize};

/// The manifest's `format`.
const MANIFEST_FORMAT: &str = "ascribe-manifest";
/// The manifest format's version.
const MANIFEST_VERSION: u32 = 1;
const LOCK_FILE: &str = ".lock";
const STAGING_DIR: &str = ".staging";
/// The folder above the builds' folders that holds what they share.
const SHARED_DIR: &str = "_ascribe";
/// The shared folder's manifest, inside it.
const SHARED_MANIFEST: &str = "manifest.json";
/// The manifest's `emitter` for an archive, and its staging folder.
const ARCHIVE: &str = "archive";
/// An archive's manifest, at the top of the archive's directory.
const ARCHIVE_MANIFEST: &str = "archive.manifest.json";

/// Where one output's files and manifest are.
struct Place {
    /// The manifest's `build`.
    build: String,
    /// The manifest's `emitter`.
    emitter: String,
    /// The folder the output's folder is in, which must be a folder.
    parent: PathBuf,
    /// The folder the files are written in.
    files: PathBuf,
    /// The manifest.
    manifest: PathBuf,
    /// Where the manifest is written before it's renamed into place.
    temp: PathBuf,
}

/// What a file in an emitter root is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileKind {
    /// A page.
    Page,
    /// A copy of a file a page refers to.
    Asset,
    /// Anything else Ascribe writes, such as a schema.
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
    /// A file is at a manifest's path but isn't one, so Ascribe can't tell
    /// what it owns.
    #[error(
        "{path} isn't an Ascribe manifest, so Ascribe can't tell which files in its directory it owns; move or remove it"
    )]
    NotManifest {
        /// The path.
        path: String,
    },
    /// A manifest of a version this Ascribe doesn't know.
    #[error("{path} has manifest version {version}, which this ascribe doesn't know")]
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
    /// A symbolic link, or on Windows a junction, is on the way to where the
    /// build writes or removes. It isn't followed, replaced, or removed.
    #[error(
        "{path} is a link, and the output is never written or removed through one; replace it with a folder, or remove it"
    )]
    Link {
        /// The link.
        path: String,
    },
    /// An archive was to be written in a directory that has files and no
    /// archive manifest, so isn't an archive Ascribe wrote.
    #[error(
        "{dir} isn't empty, and isn't an archive Ascribe wrote; name an empty folder, or one an archive was written in"
    )]
    NotEmpty {
        /// The directory.
        dir: String,
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

impl ascribe_core::Coded for StoreError {
    fn code(&self) -> &'static str {
        match self {
            StoreError::Locked { .. } => "output_locked",
            StoreError::NotManifest { .. } => "not_a_manifest",
            StoreError::UnknownVersion { .. } => "unknown_manifest_version",
            StoreError::BadEntry { .. } => "bad_manifest_entry",
            StoreError::Link { .. } => "output_link",
            StoreError::NotEmpty { .. } => "archive_folder_not_empty",
            StoreError::Conflicts(_) => "output_conflicts",
            StoreError::Io { .. } => "output_io",
        }
    }
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

/// What the manifest records about an output besides its files.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OutputOptions {
    /// The site output has source anchors (site-render contract §7).
    pub anchors: bool,
}

#[derive(Serialize, Deserialize)]
struct Manifest {
    format: String,
    version: u32,
    build: String,
    emitter: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    anchors: bool,
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

/// The output directory, locked for one build. The lock is released when this is dropped, or by the operating
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
    /// [`StoreError::Locked`] when another build holds it, and
    /// [`StoreError::Link`] when `root` or its lock file is a link.
    pub fn lock(root: &Path) -> Result<OutputDir, StoreError> {
        refuse_link(root)?;
        fs::create_dir_all(root).map_err(io_err("can't create", root))?;
        let lock_path = root.join(LOCK_FILE);
        refuse_link(&lock_path)?;
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

    /// [`OutputDir::lock`] on the output directory `output_dir` of the project
    /// at `project`, after checking that no folder on the way from `project`
    /// to it is a link. The folders above `project` aren't checked: they're
    /// where the person running the build put the project.
    ///
    /// # Errors
    ///
    /// As [`OutputDir::lock`].
    pub fn lock_in(project: &Path, output_dir: &Path) -> Result<OutputDir, StoreError> {
        if output_dir.is_relative()
            && let Some(link) = ascribe_resolve::link_on_the_way(project, output_dir)
        {
            return Err(link_error(&link));
        }
        OutputDir::lock(&project.join(output_dir))
    }

    /// [`OutputDir::lock`] on a directory an archive is written in, after
    /// checking, before anything is written there, that it's empty or an
    /// archive Ascribe wrote: a folder that's neither gains no lock file.
    ///
    /// # Errors
    ///
    /// [`StoreError::NotEmpty`], and as [`OutputDir::lock`].
    pub fn lock_archive(dir: &Path) -> Result<OutputDir, StoreError> {
        refuse_link(dir)?;
        if fs::symlink_metadata(dir).is_ok() {
            if !dir.is_dir() {
                return Err(StoreError::Conflicts(vec![format!(
                    "{} is a file, and a directory is needed there",
                    dir.display()
                )]));
            }
            if fs::symlink_metadata(dir.join(ARCHIVE_MANIFEST)).is_err() {
                let mut entries = fs::read_dir(dir).map_err(io_err("can't read", dir))?;
                if entries.next().is_some() {
                    return Err(StoreError::NotEmpty {
                        dir: dir.display().to_string(),
                    });
                }
            }
        }
        let output = OutputDir::lock(dir)?;
        output.refuse_unowned_archive()?;
        Ok(output)
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

    /// Replaces the output of one build and emitter with `files`:
    /// stages them, checks that
    /// nothing that isn't Ascribe's is in the way, records ownership, moves
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
        self.replace_with(build, emitter, files, OutputOptions::default())
    }

    /// [`OutputDir::replace`], with the manifest recording `options`.
    ///
    /// # Errors
    ///
    /// As [`OutputDir::replace`].
    pub fn replace_with(
        &self,
        build: &str,
        emitter: &str,
        files: &[EmittedFile],
        options: OutputOptions,
    ) -> Result<Replaced, StoreError> {
        for name in [build, emitter] {
            if name.is_empty() || name.starts_with('.') || name.contains(['/', '\\']) {
                return Err(StoreError::Conflicts(vec![format!(
                    "`{name}` can't be a directory name in the output"
                )]));
            }
        }
        // Nothing on the way to the folders this writes to and removes from is
        // a link, checked before anything is touched, the cleanup below
        // included.
        refuse_links_below(&self.root, &format!("{STAGING_DIR}/{build}/{emitter}"))?;
        refuse_link(&self.manifest_temp(build, emitter))?;
        refuse_links_below(&self.root, &format!("{build}/{emitter}"))?;
        refuse_link(&self.manifest_path(build, emitter))?;
        let place = Place {
            build: build.to_owned(),
            emitter: emitter.to_owned(),
            parent: self.root.join(build),
            files: self.emitter_root(build, emitter),
            manifest: self.manifest_path(build, emitter),
            temp: self.manifest_temp(build, emitter),
        };
        let staging = self.root.join(STAGING_DIR).join(build).join(emitter);
        let result = self.replace_staged(&place, files, &staging, options);
        // Step 9: clean up, whatever happened. A failure to clean up doesn't
        // change what the build did.
        let _ = fs::remove_dir_all(&staging);
        let _ = fs::remove_dir(self.root.join(STAGING_DIR).join(build));
        let _ = fs::remove_dir(self.root.join(STAGING_DIR));
        result
    }

    /// The folder above the builds' folders, `<output-dir>/_ascribe/`, where
    /// the files the builds share are, owned through its own manifest.
    pub fn shared_root(&self) -> PathBuf {
        self.root.join(SHARED_DIR)
    }

    fn shared_place(&self) -> Place {
        let files = self.shared_root();
        Place {
            build: SHARED_DIR.to_owned(),
            emitter: SHARED_DIR.to_owned(),
            parent: files.clone(),
            manifest: files.join(SHARED_MANIFEST),
            temp: self
                .root
                .join(STAGING_DIR)
                .join(format!(".manifest-{SHARED_DIR}.tmp")),
            files,
        }
    }

    /// Replaces the files the builds share, in `<output-dir>/_ascribe/`,
    /// with `files`, by the same rules as a build's output, through the
    /// folder's own manifest, `_ascribe/manifest.json`. A folder there that
    /// has files and no manifest isn't Ascribe's, and is refused.
    ///
    /// # Errors
    ///
    /// As [`OutputDir::replace`].
    pub fn replace_shared(&self, files: &[EmittedFile]) -> Result<Replaced, StoreError> {
        let place = self.shared_place();
        refuse_links_below(&self.root, &format!("{STAGING_DIR}/{SHARED_DIR}"))?;
        refuse_link(&place.temp)?;
        refuse_links_below(&self.root, SHARED_DIR)?;
        refuse_link(&place.manifest)?;
        self.refuse_unowned_shared(&place)?;
        if files.iter().any(|f| f.path.as_str() == SHARED_MANIFEST) {
            return Err(StoreError::Conflicts(vec![format!(
                "two files would be written at {SHARED_DIR}/{SHARED_MANIFEST}"
            )]));
        }
        let staging = self.root.join(STAGING_DIR).join(SHARED_DIR);
        let result = self.replace_staged(&place, files, &staging, OutputOptions::default());
        let _ = fs::remove_dir_all(&staging);
        let _ = fs::remove_dir(self.root.join(STAGING_DIR));
        result
    }

    /// The manifest of an archive written into this directory:
    /// `<dir>/archive.manifest.json`, beside the archive's files.
    pub fn archive_manifest_path(&self) -> PathBuf {
        self.root.join(ARCHIVE_MANIFEST)
    }

    fn archive_place(&self, build: &str) -> Place {
        Place {
            build: build.to_owned(),
            emitter: ARCHIVE.to_owned(),
            parent: self.root.clone(),
            files: self.root.clone(),
            manifest: self.archive_manifest_path(),
            temp: self
                .root
                .join(STAGING_DIR)
                .join(format!(".manifest-{ARCHIVE}.tmp")),
        }
    }

    /// Replaces the archive in this directory with `files`, the archive of
    /// `build`, written at its top rather than under a build's folder, by
    /// the same rules as a build's output, through the manifest
    /// `archive.manifest.json` beside them. A directory that has anything in it besides the lock, and no
    /// manifest, isn't an archive Ascribe wrote, and is refused.
    ///
    /// # Errors
    ///
    /// As [`OutputDir::replace`].
    pub fn replace_archive(
        &self,
        build: &str,
        files: &[EmittedFile],
    ) -> Result<Replaced, StoreError> {
        let place = self.archive_place(build);
        refuse_links_below(&self.root, &format!("{STAGING_DIR}/{ARCHIVE}"))?;
        refuse_link(&place.temp)?;
        refuse_link(&place.manifest)?;
        self.refuse_unowned_archive()?;
        let reserved = [ARCHIVE_MANIFEST, LOCK_FILE, STAGING_DIR];
        let taken: Vec<String> = files
            .iter()
            .filter(|f| {
                f.path
                    .segments()
                    .next()
                    .is_some_and(|first| reserved.contains(&first))
            })
            .map(|f| format!("{} is a name the archive's own files take", f.path))
            .collect();
        if !taken.is_empty() {
            return Err(StoreError::Conflicts(taken));
        }
        let staging = self.root.join(STAGING_DIR).join(ARCHIVE);
        let result = self.replace_staged(&place, files, &staging, OutputOptions::default());
        let _ = fs::remove_dir_all(&staging);
        let _ = fs::remove_dir(self.root.join(STAGING_DIR));
        result
    }

    /// Refuses this directory for an archive when something besides the
    /// lock is in it and no manifest says Ascribe wrote it.
    fn refuse_unowned_archive(&self) -> Result<(), StoreError> {
        if fs::symlink_metadata(self.archive_manifest_path()).is_ok() {
            return Ok(());
        }
        let entries = fs::read_dir(&self.root).map_err(io_err("can't read", &self.root))?;
        for entry in entries {
            let entry = entry.map_err(io_err("can't read", &self.root))?;
            let name = entry.file_name();
            if name != LOCK_FILE && name != STAGING_DIR {
                return Err(StoreError::NotEmpty {
                    dir: self.root.display().to_string(),
                });
            }
        }
        Ok(())
    }

    /// Removes the files the builds share, when a project no longer has any:
    /// each file `_ascribe/manifest.json` lists, then the manifest, then the
    /// folder when nothing else is in it. A folder with no manifest isn't
    /// Ascribe's, and is left alone. Returns how many files were removed.
    ///
    /// # Errors
    ///
    /// The manifest can't be read, or a file can't be removed.
    pub fn remove_shared(&self) -> Result<usize, StoreError> {
        let place = self.shared_place();
        refuse_links_below(&self.root, SHARED_DIR)?;
        if fs::symlink_metadata(&place.manifest).is_err() {
            return Ok(0);
        }
        let previous = read_manifest(&place.manifest)?;
        let mut removed = 0;
        for old in &previous {
            let path = place.files.join(&old.path);
            if let Ok(meta) = fs::symlink_metadata(&path)
                && !meta.is_dir()
            {
                fs::remove_file(&path).map_err(io_err("can't remove", &path))?;
                removed += 1;
            }
            let mut dir = path.parent().map(Path::to_owned);
            while let Some(d) = dir {
                if d == place.files || fs::remove_dir(&d).is_err() {
                    break;
                }
                dir = d.parent().map(Path::to_owned);
            }
        }
        fs::remove_file(&place.manifest).map_err(io_err("can't remove", &place.manifest))?;
        let _ = fs::remove_dir(&place.files);
        Ok(removed)
    }

    /// Refuses `_ascribe/` when something is in it and no manifest says
    /// Ascribe wrote it.
    fn refuse_unowned_shared(&self, place: &Place) -> Result<(), StoreError> {
        if fs::symlink_metadata(&place.manifest).is_ok() {
            return Ok(());
        }
        match fs::symlink_metadata(&place.files) {
            Err(_) => Ok(()),
            Ok(meta) if !meta.is_dir() => Err(StoreError::Conflicts(vec![format!(
                "{} is a file, and a directory is needed there",
                place.files.display()
            )])),
            Ok(_) => {
                let empty = fs::read_dir(&place.files)
                    .map_err(io_err("can't read", &place.files))?
                    .next()
                    .is_none();
                if empty {
                    Ok(())
                } else {
                    Err(StoreError::Conflicts(vec![format!(
                        "{} has files and no manifest, so it isn't a folder Ascribe wrote; move or remove it",
                        place.files.display()
                    )]))
                }
            }
        }
    }

    // Longer than the lint allows from before it was on. Split it only while
    // changing it for another reason.
    #[allow(clippy::too_many_lines)]
    fn replace_staged(
        &self,
        place: &Place,
        files: &[EmittedFile],
        staging: &Path,
        options: OutputOptions,
    ) -> Result<Replaced, StoreError> {
        let manifest_path = &place.manifest;
        let emitter_root = place.files.clone();

        // Step 2: the previous manifest.
        let previous = read_manifest(manifest_path)?;
        let previous_paths: BTreeSet<&str> = previous.iter().map(|f| f.path.as_str()).collect();

        // Nothing between the emitter root and a file this writes or removes
        // is a link, nor is the file.
        let mut links = ascribe_resolve::LinkCheck::new(&emitter_root);
        for path in files
            .iter()
            .map(|f| f.path.as_str())
            .chain(previous_paths.iter().copied())
        {
            if let Some(link) = links.link(Path::new(path)) {
                return Err(link_error(&link));
            }
        }

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
        // Step 4, second part: nothing that isn't Ascribe's is in the way.
        let mut blockers: BTreeSet<PathBuf> = BTreeSet::new();
        for dir in [place.parent.clone(), emitter_root.clone()] {
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
                    problems.push(format!(
                        "{} is a directory, and a file would be written there",
                        dest.display()
                    ));
                } else if !previous_paths.contains(file.path.as_str()) {
                    problems.push(format!(
                        "{} exists and isn't a file Ascribe wrote; move or remove it",
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
                    // A file where this build needs a directory.
                    if previous_paths.contains(prefix.as_str()) {
                        // Ascribe's own file, which this build no longer
                        // produces as a file: it's removed before the
                        // directory is made.
                        blockers.insert(at);
                    } else {
                        problems.push(format!(
                            "{} is a file that isn't Ascribe's, and a directory is needed there",
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

        // Step 3: emit into staging, except a file whose destination already
        // holds exactly its bytes: it stays where it is, untouched, so an
        // unchanged rebuild writes (and reads back) nothing it doesn't need
        // to.
        if staging.exists() {
            fs::remove_dir_all(staging).map_err(io_err("can't remove", staging))?;
        }
        fs::create_dir_all(staging).map_err(io_err("can't create", staging))?;
        let mut in_place: BTreeSet<&RelPath> = BTreeSet::new();
        let mut made: BTreeSet<PathBuf> = BTreeSet::new();
        for file in files {
            let current = emitter_root.join(file.path.as_str());
            let same = match &file.contents {
                Contents::Text(text) => holds(&current, text.as_bytes()),
                Contents::Copy(from) => same_content(from, &current),
            };
            if same {
                in_place.insert(&file.path);
                continue;
            }
            let dest = staging.join(file.path.as_str());
            if let Some(parent) = dest.parent()
                && made.insert(parent.to_path_buf())
            {
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
        self.write_manifest(place, union.into_values().collect(), options)?;

        // Step 6: move the new files into place.
        for blocker in &blockers {
            fs::remove_file(blocker).map_err(io_err("can't remove", blocker))?;
        }
        let mut replaced = Replaced::default();
        for file in files {
            let staged = staging.join(file.path.as_str());
            let dest = emitter_root.join(file.path.as_str());
            if in_place.contains(&file.path) || same_content(&staged, &dest) {
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
        self.write_manifest(place, new_entries, options)?;
        Ok(replaced)
    }

    /// Where a manifest is written before it's renamed into place.
    fn manifest_temp(&self, build: &str, emitter: &str) -> PathBuf {
        self.root
            .join(STAGING_DIR)
            .join(format!(".manifest-{build}-{emitter}.tmp"))
    }

    /// Writes a manifest by writing a temporary file in `.staging/` and
    /// renaming it into place, so it's never half-written.
    fn write_manifest(
        &self,
        place: &Place,
        mut files: Vec<ManifestFile>,
        options: OutputOptions,
    ) -> Result<(), StoreError> {
        files.sort_by(|a, b| a.path.cmp(&b.path));
        let manifest = Manifest {
            format: MANIFEST_FORMAT.to_owned(),
            version: MANIFEST_VERSION,
            build: place.build.clone(),
            emitter: place.emitter.clone(),
            anchors: options.anchors,
            files,
        };
        let mut text = serde_json::to_string_pretty(&manifest).map_err(|e| StoreError::Io {
            action: "can't serialize the manifest",
            path: place.emitter.clone(),
            source: io::Error::other(e),
        })?;
        text.push('\n');
        let staging = self.root.join(STAGING_DIR);
        fs::create_dir_all(&staging).map_err(io_err("can't create", &staging))?;
        let temp = &place.temp;
        fs::write(temp, text).map_err(io_err("can't write", temp))?;
        let dest = &place.manifest;
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(io_err("can't create", parent))?;
        }
        fs::rename(temp, dest).map_err(io_err("can't move into place", dest))
    }
}

/// Reads the manifest at `path`. No file means no previous output. A file that
/// isn't a manifest is an error.
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
    // Ascribe deletes what a manifest lists, and a manifest is a file in the
    // project like any other, so an entry that could reach outside the
    // emitter root is refused, not followed: one with `..`, and one this
    // system reads as absolute or on a drive (`C:/x`, `C:x` on Windows).
    for entry in &manifest.files {
        let inside = RelPath::parse(&entry.path)
            .is_ok_and(|p| !p.is_root() && p.up_count() == 0 && p.as_str() == entry.path)
            && Path::new(&entry.path)
                .components()
                .all(|c| matches!(c, Component::Normal(_)));
        if !inside || entry.path.contains('\\') {
            return Err(StoreError::BadEntry {
                path: shown,
                entry: entry.path.clone(),
            });
        }
    }
    Ok(manifest.files)
}

/// Refuses a link at `path`.
fn refuse_link(path: &Path) -> Result<(), StoreError> {
    if ascribe_resolve::is_link(path) {
        return Err(link_error(path));
    }
    Ok(())
}

/// Refuses a link on the way from `root` to the `/`-separated `rel`, `rel`
/// included.
fn refuse_links_below(root: &Path, rel: &str) -> Result<(), StoreError> {
    match ascribe_resolve::link_on_the_way(root, Path::new(rel)) {
        Some(link) => Err(link_error(&link)),
        None => Ok(()),
    }
}

fn link_error(path: &Path) -> StoreError {
    StoreError::Link {
        path: path.display().to_string(),
    }
}

/// Whether the file at `dest` exists and holds exactly `bytes`.
fn holds(dest: &Path, bytes: &[u8]) -> bool {
    match fs::metadata(dest) {
        Ok(meta) if meta.is_file() && meta.len() == bytes.len() as u64 => {
            fs::read(dest).is_ok_and(|current| current == bytes)
        }
        _ => false,
    }
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
