//! The file system the source index reads, and two implementations.
//!
//! Everything the index needs from disk goes through [`FileSystem`], so the
//! language server can index unsaved buffers, and tests can build a project in
//! memory. A [`DiskFs`] reads a real project; a [`MemoryFs`] holds one.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use ascribe_core::RelPath;

use crate::layout::Layout;
use crate::project::Unreadable;

/// What a path names, checked with exact, case-sensitive names on every
/// platform (SPEC §9.4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Probe {
    /// A regular file, or a symbolic link to one.
    File,
    /// Nothing, or something that isn't a regular file (a directory).
    Missing,
    /// No file has this exact name, but one differs only in case. Holds the
    /// path as the file system spells it, relative to the project root.
    CaseMismatch(RelPath),
}

/// The name of a project's content model file. A directory below the content
/// root that holds one is another project's folder, unless it's the project's
/// own.
pub(crate) const MODEL_FILE: &str = "ascribe.toml";

/// The source files a file system holds, and the places it couldn't look.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sources {
    /// Every source file, as content paths, in any order.
    pub paths: Vec<RelPath>,
    /// Directories and entries that couldn't be read. They aren't skipped
    /// silently: the project lists them ([`crate::Project::unreadable`]).
    pub unreadable: Vec<Unreadable>,
    /// The directories skipped because they hold an `ascribe.toml`: the
    /// folders of other projects nested in the content root, as content
    /// paths, in any order. One nested in another isn't listed.
    pub nested: Vec<RelPath>,
    /// The project's own folder, the directory that holds its `ascribe.toml`,
    /// as a content path, when it's below the content root (a content root
    /// above the project root, such as `..`). It's never skipped.
    pub own_folder: Option<RelPath>,
}

/// Whether a content path names a source file by its path alone: exactly
/// `.md`, inside the content root (no leading `..`), and not in or under
/// anything whose name starts with `.`. A file that passes is still not a
/// source when it's inside a nested project's folder ([`in_nested_project`]),
/// which takes knowing which directories hold an `ascribe.toml`. Discovery
/// ([`FileSystem::sources`]) and the incremental update ([`crate::Change`])
/// use the same rule; [`crate::Project::is_source`] applies both parts.
pub fn is_source_path(path: &RelPath) -> bool {
    path.is_inside()
        && path.extension() == Some("md")
        && !path.segments().any(|s| s.starts_with('.'))
}

/// Whether a content path is in or under one of `nested`, the folders of
/// projects nested in the content root ([`Sources::nested`]).
pub fn in_nested_project(path: &RelPath, nested: &[RelPath]) -> bool {
    nested.iter().any(|dir| path.starts_with(dir))
}

/// The files of a project.
pub trait FileSystem {
    /// Every source file under the content root: exactly the files whose name
    /// ends in `.md`, skipping any file or directory whose name starts with `.`
    /// (`.github/`, `.vitepress/`, editor state), and any directory below the
    /// content root that holds an `ascribe.toml` (another project's folder,
    /// listed in [`Sources::nested`]). Neither the content root itself nor
    /// the project's own folder is ever skipped, so a project whose content
    /// root is its own folder, or above it, keeps its files. Directories are
    /// followed through symbolic links once each.
    // The same rule as `ascribe check`'s discovery, which
    // raised it: which files count as sources.
    fn sources(&self) -> Sources;

    /// The text of a source file, by content path. A file whose symbolic
    /// links lead out of the content root can't be read (SPEC §2.1).
    fn read(&self, path: &RelPath) -> io::Result<String>;

    /// Whether a file exists at a path relative to the project root. The
    /// path may start with `..` when the content root is above the project
    /// root; the boundary check has already decided the path may be read.
    fn probe(&self, project_path: &RelPath) -> Probe;

    /// The bytes of a file at a path relative to the project root: a code
    /// file a snippet reads through a source (SPEC §4.8). The path may start
    /// with `..`; the source has already decided it may be read.
    fn read_file(&self, project_path: &RelPath) -> io::Result<Vec<u8>>;

    /// Where a file or folder is once every symbolic link on the way to it is
    /// followed, as a path relative to the project root: the path itself when
    /// none of its segments is a link. It starts with at least as many `..`
    /// as `project_path` does, so a path through a folder above the project
    /// root compares with that folder's own real path. `None` when it isn't there, or a link
    /// leads where no path from the project root reaches (another drive). A
    /// snippet reads a file only when this is inside its source (SPEC §4.8).
    fn real_path(&self, project_path: &RelPath) -> Option<RelPath>;

    /// The size in bytes of a file at a path relative to the project root;
    /// `None` when it isn't there. A file system that can't tell reads the
    /// file.
    fn size(&self, project_path: &RelPath) -> Option<u64> {
        self.read_file(project_path)
            .ok()
            .map(|bytes| bytes.len() as u64)
    }

    /// Every file in a folder and the folders in it, as paths relative to
    /// the project root, in path order: the copies of a source in another
    /// repository (SPEC §7.4). A symbolic link is listed, not followed.
    /// Empty when the folder isn't there; a file system that can't list
    /// lists nothing.
    fn files_in(&self, _project_dir: &RelPath) -> Vec<RelPath> {
        Vec::new()
    }
}

/// A project on disk.
#[derive(Clone, Debug)]
pub struct DiskFs {
    project_root: PathBuf,
    content_root: RelPath,
    /// Directory listings, kept for the life of the value when it was made
    /// with [`DiskFs::with_listing_cache`]; `None` lists every time.
    listings: Option<Arc<Mutex<Listings>>>,
    /// The files [`FileSystem::read_file`] read, kept as the listings are.
    files: Option<Arc<Mutex<Files>>>,
}

/// Each file's bytes, or why it couldn't be read.
type Files = HashMap<PathBuf, Result<Arc<[u8]>, (io::ErrorKind, String)>>;

/// Each directory's entry names, or `None` for one that can't be listed.
type Listings = HashMap<PathBuf, Option<Arc<[String]>>>;

impl DiskFs {
    /// The project in `project_root` (the directory with `ascribe.toml`).
    /// Every probe reads the disk, so a long-lived value sees files come and
    /// go (the language server's).
    pub fn new(project_root: impl Into<PathBuf>, layout: &Layout) -> DiskFs {
        DiskFs {
            project_root: project_root.into(),
            content_root: layout.content_root.clone(),
            listings: None,
            files: None,
        }
    }

    /// Like [`DiskFs::new`], but each directory a probe looks in is listed
    /// once and remembered, for a one-shot command (`ascribe check`,
    /// `ascribe build`) that reads a disk that doesn't change while it runs.
    /// A probe lists every directory on its path, so without this a large
    /// project lists the same directories tens of thousands of times. Code
    /// files snippets read are remembered too, so the file-level checks and
    /// the source index read each once.
    pub fn with_listing_cache(project_root: impl Into<PathBuf>, layout: &Layout) -> DiskFs {
        DiskFs {
            listings: Some(Arc::default()),
            files: Some(Arc::default()),
            ..DiskFs::new(project_root, layout)
        }
    }

    fn content_dir(&self) -> PathBuf {
        self.project_root.join(self.content_root.as_str())
    }

    /// Whether `real`, a canonical path, is a source file of the content root
    /// whose canonical path is `root`: under it, named as a source is, and
    /// not in another project's folder.
    fn is_source_at(&self, root: &Path, real: &Path) -> bool {
        let Ok(rest) = real.strip_prefix(root) else {
            return false;
        };
        let Some(rel) = rest
            .to_str()
            .and_then(|r| RelPath::parse(&r.replace('\\', "/")).ok())
        else {
            return false;
        };
        if !is_source_path(&rel) {
            return false;
        }
        let own = self.project_root.canonicalize().ok();
        // A folder below the content root that holds an `ascribe.toml` is
        // another project's, unless it's this project's own.
        real.ancestors()
            .skip(1)
            .take_while(|dir| *dir != root)
            .all(|dir| own.as_deref() == Some(dir) || !dir.join(MODEL_FILE).is_file())
    }

    /// The names in a directory, or `None` when it can't be listed.
    fn list(&self, dir: &Path) -> Option<Arc<[String]>> {
        let read = || -> Option<Arc<[String]>> {
            let entries = fs::read_dir(dir).ok()?;
            Some(
                entries
                    .flatten()
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .collect(),
            )
        };
        let Some(listings) = &self.listings else {
            return read();
        };
        let mut listings = listings
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        listings
            .entry(dir.to_path_buf())
            .or_insert_with(read)
            .clone()
    }
}

impl FileSystem for DiskFs {
    fn sources(&self) -> Sources {
        let mut out = Sources::default();
        let mut seen = BTreeSet::new();
        let walk_from = Walk {
            own: fs::canonicalize(&self.project_root).ok(),
        };
        walk_from.walk(&self.content_dir(), &RelPath::root(), &mut seen, &mut out);
        out.paths.sort();
        out.nested.sort();
        out
    }

    fn read(&self, path: &RelPath) -> io::Result<String> {
        let file = self.content_dir().join(path.as_str());
        // A source file is read where its links lead, and that must be a
        // source file too, so a link can't put any other file on disk in a
        // page (SPEC §2.1).
        let root = self.content_dir().canonicalize()?;
        let real = file.canonicalize()?;
        if !self.is_source_at(&root, &real) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                link_out(path),
            ));
        }
        fs::read_to_string(file)
    }

    fn probe(&self, project_path: &RelPath) -> Probe {
        let mut dir = self.project_root.clone();
        let mut actual: Vec<String> = Vec::new();
        let mut differs = false;
        let segments: Vec<&str> = project_path.segments().collect();
        for segment in &segments {
            if *segment == ".." {
                dir.push("..");
                actual.push("..".to_owned());
                continue;
            }
            let Some(entries) = self.list(&dir) else {
                return Probe::Missing;
            };
            let mut exact = None;
            let mut folded = None;
            for name in entries.iter() {
                if name == segment {
                    exact = Some(name.clone());
                    break;
                }
                if folded.is_none() && name.to_lowercase() == segment.to_lowercase() {
                    folded = Some(name.clone());
                }
            }
            let name = match (exact, folded) {
                (Some(name), _) => name,
                (None, Some(name)) => {
                    differs = true;
                    name
                }
                (None, None) => return Probe::Missing,
            };
            dir.push(&name);
            actual.push(name);
        }
        if segments.is_empty() || !dir.is_file() {
            return Probe::Missing;
        }
        if differs {
            match RelPath::parse(&actual.join("/")) {
                Ok(path) => Probe::CaseMismatch(path),
                Err(_) => Probe::Missing,
            }
        } else {
            Probe::File
        }
    }

    fn read_file(&self, project_path: &RelPath) -> io::Result<Vec<u8>> {
        let path = project_path
            .segments()
            .fold(self.project_root.clone(), |p, s| p.join(s));
        let Some(files) = &self.files else {
            return fs::read(&path);
        };
        let mut files = files
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let read = files.entry(path).or_insert_with_key(|path| {
            fs::read(path)
                .map(Arc::from)
                .map_err(|e| (e.kind(), e.to_string()))
        });
        match read {
            Ok(bytes) => Ok(bytes.to_vec()),
            Err((kind, message)) => Err(io::Error::new(*kind, message.clone())),
        }
    }

    fn real_path(&self, project_path: &RelPath) -> Option<RelPath> {
        // Measured from the folder the path's leading `..`s reach, so the
        // answer keeps them: `../docs/ascribe.toml`, not `ascribe.toml`.
        let ups = project_path.up_count();
        let root = self.project_root.canonicalize().ok()?;
        let base = root.ancestors().nth(ups)?;
        let path = project_path
            .segments()
            .fold(self.project_root.clone(), |p, s| p.join(s));
        let rest = ascribe_core::path::relative_path(base, &path.canonicalize().ok()?)?;
        let mut segments = vec![".."; ups];
        segments.extend(rest.segments());
        RelPath::parse(&segments.join("/")).ok()
    }

    fn size(&self, project_path: &RelPath) -> Option<u64> {
        let path = project_path
            .segments()
            .fold(self.project_root.clone(), |p, s| p.join(s));
        fs::metadata(path)
            .ok()
            .filter(|m| m.is_file())
            .map(|m| m.len())
    }

    fn files_in(&self, project_dir: &RelPath) -> Vec<RelPath> {
        let dir = project_dir
            .segments()
            .fold(self.project_root.clone(), |p, s| p.join(s));
        let mut out = Vec::new();
        list_files(&dir, project_dir, &mut out);
        out.sort();
        out
    }
}

/// Why a file reached through a symbolic link can't be read: the link leads
/// to a file that isn't a source file of the content root. `path` is the
/// file's content path.
fn link_out(path: impl std::fmt::Display) -> String {
    format!(
        "`{path}` is a symbolic link, or is in a linked folder, that leads to a file that isn't a source file of the content root"
    )
}

impl ascribe_core::SourceBoundary for DiskFs {
    fn check_link(&self, path: &Path) -> Result<(), String> {
        // Everything is compared once links are followed, so no other
        // spelling of the content root (a link to it, another case, `\\?\`)
        // gets past the check.
        let (Ok(root), Ok(real)) = (self.content_dir().canonicalize(), path.canonicalize()) else {
            // Not there: reading it reports that.
            return Ok(());
        };
        if self.is_source_at(&root, &real) {
            return Ok(());
        }
        // The path as written, `..` and all: the system follows a link before
        // it applies the `..` after it.
        let file = if path.is_absolute() {
            path.to_owned()
        } else {
            match std::env::current_dir() {
                Ok(dir) => dir.join(path),
                Err(_) => return Ok(()),
            }
        };
        // The nearest folder on the way to the file that is the content root,
        // however it's spelled. With none, the file isn't in the content root.
        let Some((dir, rest)) = file
            .ancestors()
            .skip(1)
            .find(|dir| dir.canonicalize().is_ok_and(|d| d == root))
            .and_then(|dir| Some((dir, file.strip_prefix(dir).ok()?)))
        else {
            return Ok(());
        };
        // A file reached through no link is where its path says, and is
        // formatted as named.
        let mut at = dir.to_owned();
        let mut linked = false;
        for part in rest.components() {
            at.push(part);
            if matches!(part, std::path::Component::Normal(_))
                && fs::symlink_metadata(&at).is_ok_and(|m| m.file_type().is_symlink())
            {
                linked = true;
                break;
            }
        }
        if !linked {
            return Ok(());
        }
        Err(link_out(rest.to_string_lossy().replace('\\', "/")))
    }
}

/// The files under `dir` (at `rel` from the project root), without following
/// a symbolic link: a link is listed as a file.
fn list_files(dir: &Path, rel: &RelPath, out: &mut Vec<RelPath>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        let name = entry.file_name().to_string_lossy().into_owned();
        let Ok(child) = rel.join(&name) else {
            continue;
        };
        if kind.is_dir() {
            list_files(&entry.path(), &child, out);
        } else {
            out.push(child);
        }
    }
}

/// What a walk of the content root needs to know besides the tree.
struct Walk {
    /// The project root, canonical: the project's own folder, whose
    /// `ascribe.toml` doesn't make it another project's.
    own: Option<PathBuf>,
}

impl Walk {
    /// Collects source files under `dir`, as paths relative to the content root,
    /// and the nested projects' folders it skips. `seen` holds the canonical
    /// directories already walked, so a symbolic link back up the tree can't loop.
    fn walk(&self, dir: &Path, rel: &RelPath, seen: &mut BTreeSet<PathBuf>, out: &mut Sources) {
        let unreadable = |out: &mut Sources, path: &Path, err: io::Error| {
            out.unreadable.push(Unreadable {
                path: rel.clone(),
                reason: format!("{}: {err}", path.display()),
            });
        };
        let canonical = match fs::canonicalize(dir) {
            Ok(canonical) => canonical,
            Err(err) => return unreadable(out, dir, err),
        };
        let own = self.own.as_ref() == Some(&canonical);
        if !seen.insert(canonical) {
            return;
        }
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(err) => return unreadable(out, dir, err),
        };
        let mut listed = Vec::new();
        for entry in entries {
            match entry {
                Ok(entry) => listed.push(entry),
                Err(err) => unreadable(out, dir, err),
            }
        }
        if own && !rel.is_root() && out.own_folder.is_none() {
            out.own_folder = Some(rel.clone());
        }
        // A directory below the content root with an `ascribe.toml` is another
        // project's folder, unless it's this project's own: none of it is a
        // source of this one.
        if !rel.is_root()
            && !own
            && listed
                .iter()
                .any(|e| e.file_name() == MODEL_FILE && e.path().is_file())
        {
            out.nested.push(rel.clone());
            return;
        }
        for entry in listed {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let path = entry.path();
            let Ok(child) = rel.join(&name) else {
                continue;
            };
            // `metadata` follows symbolic links.
            let meta = match fs::metadata(&path) {
                Ok(meta) => meta,
                Err(err) => {
                    out.unreadable.push(Unreadable {
                        path: child,
                        reason: format!("{}: {err}", path.display()),
                    });
                    continue;
                }
            };
            if meta.is_dir() {
                self.walk(&path, &child, seen, out);
            } else if meta.is_file() && name.ends_with(".md") {
                out.paths.push(child);
            }
        }
    }
}

/// A project held in memory: every file, source or not, by path relative to
/// the project root.
#[derive(Clone, Debug, Default)]
pub struct MemoryFs {
    content_root: RelPath,
    files: BTreeMap<RelPath, String>,
    own_folder: Option<RelPath>,
}

impl MemoryFs {
    /// An empty project with this layout.
    pub fn new(layout: &Layout) -> MemoryFs {
        MemoryFs {
            content_root: layout.content_root.clone(),
            files: BTreeMap::new(),
            own_folder: None,
        }
    }

    /// Adds a file at a path relative to the project root. Files that aren't
    /// text (images) can have any content.
    pub fn with_file(mut self, project_path: &str, text: &str) -> MemoryFs {
        if let Ok(path) = RelPath::parse(project_path) {
            self.files.insert(path, text.to_owned());
        }
        self
    }

    /// Says which directory below the content root, as a content path, is the
    /// project's own folder, when the content root is above the project root
    /// (`..`): the `ascribe.toml` there is the project's own, so the folder
    /// isn't skipped. Files in it are added with [`MemoryFs::with_source`].
    pub fn with_own_folder(mut self, content_path: &str) -> MemoryFs {
        self.own_folder = RelPath::parse(content_path).ok();
        self
    }

    /// Adds a file at a content path.
    pub fn with_source(self, content_path: &str, text: &str) -> MemoryFs {
        let project_path = self
            .content_root
            .join(content_path)
            .map(|p| p.to_string())
            .unwrap_or_default();
        self.with_file(&project_path, text)
    }

    fn content_path(&self, project_path: &RelPath) -> Option<RelPath> {
        if !project_path.starts_with(&self.content_root) {
            return None;
        }
        let rest: Vec<&str> = project_path
            .segments()
            .skip(self.content_root.segments().count())
            .collect();
        RelPath::parse(&rest.join("/")).ok()
    }
}

impl FileSystem for MemoryFs {
    fn sources(&self) -> Sources {
        let content: BTreeSet<RelPath> = self
            .files
            .keys()
            .filter_map(|p| self.content_path(p))
            .collect();
        // The directories below the content root that hold an `ascribe.toml`,
        // outermost first, keeping only those the walk would reach: not
        // hidden, and not inside another one. The project's own folder isn't
        // another project's.
        let mut nested: Vec<RelPath> = Vec::new();
        let mut holding: Vec<RelPath> = content
            .iter()
            .filter(|p| p.file_name() == Some(MODEL_FILE))
            .filter_map(RelPath::parent)
            .filter(|dir| !dir.is_root() && dir.is_inside())
            .filter(|dir| Some(dir) != self.own_folder.as_ref())
            .filter(|dir| !dir.segments().any(|s| s.starts_with('.')))
            .collect();
        holding.sort_by_key(|dir| dir.segments().count());
        for dir in holding {
            if !in_nested_project(&dir, &nested) {
                nested.push(dir);
            }
        }
        nested.sort();
        let paths = content
            .into_iter()
            .filter(|p| is_source_path(p) && !in_nested_project(p, &nested))
            .collect();
        Sources {
            paths,
            unreadable: Vec::new(),
            nested,
            own_folder: self.own_folder.clone(),
        }
    }

    fn read(&self, path: &RelPath) -> io::Result<String> {
        self.content_root
            .join(path.as_str())
            .ok()
            .and_then(|p| self.files.get(&p))
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such file"))
    }

    fn read_file(&self, project_path: &RelPath) -> io::Result<Vec<u8>> {
        self.files
            .get(project_path)
            .map(|text| text.as_bytes().to_vec())
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such file"))
    }

    /// There are no links in memory.
    fn real_path(&self, project_path: &RelPath) -> Option<RelPath> {
        Some(project_path.clone())
    }

    fn probe(&self, project_path: &RelPath) -> Probe {
        if self.files.contains_key(project_path) {
            return Probe::File;
        }
        let folded = project_path.as_str().to_lowercase();
        match self
            .files
            .keys()
            .find(|p| p.as_str().to_lowercase() == folded)
        {
            Some(actual) => Probe::CaseMismatch(actual.clone()),
            None => Probe::Missing,
        }
    }

    fn files_in(&self, project_dir: &RelPath) -> Vec<RelPath> {
        self.files
            .keys()
            .filter(|p| p.starts_with(project_dir) && *p != project_dir)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> RelPath {
        RelPath::parse(s).expect("valid path")
    }

    fn layout() -> Layout {
        Layout {
            content_root: p("docs"),
            output_dir: p(".ascribe/build"),
        }
    }

    #[test]
    fn memory_fs_lists_sources_and_probes_with_exact_case() {
        let fs = MemoryFs::new(&layout())
            .with_source("index.md", "x")
            .with_source("_f/a.md", "y")
            .with_source("img.png", "")
            .with_file("README.md", "")
            .with_file("shared/logo.png", "");
        let mut sources = fs.sources().paths;
        sources.sort();
        assert_eq!(sources, [p("_f/a.md"), p("index.md")]);
        assert_eq!(fs.read(&p("index.md")).ok().as_deref(), Some("x"));
        assert_eq!(fs.probe(&p("docs/img.png")), Probe::File);
        assert_eq!(fs.probe(&p("docs/nope.png")), Probe::Missing);
        assert_eq!(
            fs.probe(&p("shared/Logo.png")),
            Probe::CaseMismatch(p("shared/logo.png"))
        );
    }

    #[test]
    fn dot_names_are_not_sources() {
        let fs = MemoryFs::new(&layout())
            .with_source("index.md", "x")
            .with_source(".github/PULL_REQUEST_TEMPLATE.md", "y")
            .with_source(".vitepress/notes.md", "y")
            .with_source("guides/.draft.md", "y");
        assert_eq!(fs.sources().paths, [p("index.md")]);

        let dir = std::env::temp_dir().join(format!("ascribe-resolve-dot-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("docs/.github")).expect("create dirs");
        fs::create_dir_all(dir.join("docs/guides")).expect("create dirs");
        fs::write(dir.join("docs/index.md"), "x").expect("write");
        fs::write(dir.join("docs/.github/template.md"), "y").expect("write");
        fs::write(dir.join("docs/guides/.draft.md"), "y").expect("write");
        fs::write(dir.join("docs/guides/a.md"), "y").expect("write");
        // Only a name that is exactly `<name>.md` counts, not `.markdown`.
        fs::write(dir.join("docs/guides/b.markdown"), "y").expect("write");
        let disk = DiskFs::new(&dir, &layout());
        assert_eq!(disk.sources().paths, [p("guides/a.md"), p("index.md")]);
        let _ = fs::remove_dir_all(&dir);
    }

    /// A path under `dir`, from a `/`-separated relative one.
    fn at(dir: &Path, rel: &str) -> PathBuf {
        rel.split('/')
            .fold(dir.to_path_buf(), |path, s| path.join(s))
    }

    #[test]
    fn a_nested_projects_folder_is_skipped() {
        let fs = MemoryFs::new(&layout())
            .with_source("index.md", "x")
            .with_source("guides/a.md", "y")
            .with_source("nested/ascribe.toml", "")
            .with_source("nested/content/index.md", "y")
            .with_source("nested/deeper/ascribe.toml", "")
            .with_source("nested/deeper/b.md", "y")
            .with_source(".hidden/ascribe.toml", "")
            .with_source("guides/shared/ascribe.toml/c.md", "y")
            // Only the exact name counts.
            .with_source("cased/Ascribe.toml", "")
            .with_source("cased/d.md", "y");
        let found = fs.sources();
        assert_eq!(
            found.paths,
            [
                p("cased/d.md"),
                p("guides/a.md"),
                p("guides/shared/ascribe.toml/c.md"),
                p("index.md")
            ]
        );
        assert_eq!(found.nested, [p("nested")]);

        let dir =
            std::env::temp_dir().join(format!("ascribe-resolve-nested-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for sub in [
            "docs/nested/content",
            "docs/nested/deeper",
            "docs/guides",
            "docs/.hidden",
        ] {
            fs::create_dir_all(at(&dir, sub)).expect("create dirs");
        }
        fs::write(at(&dir, "docs/index.md"), "x").expect("write");
        fs::write(at(&dir, "docs/guides/a.md"), "y").expect("write");
        fs::write(at(&dir, "docs/nested/ascribe.toml"), "").expect("write");
        fs::write(at(&dir, "docs/nested/content/index.md"), "y").expect("write");
        fs::write(at(&dir, "docs/nested/deeper/ascribe.toml"), "").expect("write");
        fs::write(at(&dir, "docs/nested/deeper/b.md"), "y").expect("write");
        fs::write(at(&dir, "docs/.hidden/ascribe.toml"), "").expect("write");
        // A directory named `ascribe.toml` isn't a file.
        fs::create_dir_all(at(&dir, "docs/guides/shared/ascribe.toml")).expect("create dirs");
        fs::write(at(&dir, "docs/guides/shared/ascribe.toml/c.md"), "y").expect("write");
        let found = DiskFs::new(&dir, &layout()).sources();
        assert_eq!(
            found.paths,
            [
                p("guides/a.md"),
                p("guides/shared/ascribe.toml/c.md"),
                p("index.md")
            ]
        );
        assert_eq!(found.nested, [p("nested")]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_content_root_at_the_project_root_keeps_its_own_ascribe_toml() {
        let layout = Layout {
            content_root: RelPath::root(),
            output_dir: p(".ascribe/build"),
        };
        let fs = MemoryFs::new(&layout)
            .with_file("ascribe.toml", "")
            .with_file("index.md", "x")
            .with_file("guides/a.md", "y")
            .with_file("nested/ascribe.toml", "")
            .with_file("nested/b.md", "y");
        let found = fs.sources();
        assert_eq!(found.paths, [p("guides/a.md"), p("index.md")]);
        assert_eq!(found.nested, [p("nested")]);

        let dir = std::env::temp_dir().join(format!("ascribe-resolve-root-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(at(&dir, "guides")).expect("create dirs");
        fs::create_dir_all(at(&dir, "nested")).expect("create dirs");
        fs::write(at(&dir, "ascribe.toml"), "").expect("write");
        fs::write(at(&dir, "index.md"), "x").expect("write");
        fs::write(at(&dir, "guides/a.md"), "y").expect("write");
        fs::write(at(&dir, "nested/ascribe.toml"), "").expect("write");
        fs::write(at(&dir, "nested/b.md"), "y").expect("write");
        let found = DiskFs::new(&dir, &layout).sources();
        assert_eq!(found.paths, [p("guides/a.md"), p("index.md")]);
        assert_eq!(found.nested, [p("nested")]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_projects_own_folder_is_never_skipped() {
        // `content-root = ".."`: the project, in `proj/`, is below its own
        // content root.
        let layout = Layout {
            content_root: p(".."),
            output_dir: p("../../out"),
        };
        let fs = MemoryFs::new(&layout)
            .with_own_folder("proj")
            .with_source("index.md", "x")
            .with_source("proj/ascribe.toml", "")
            .with_source("proj/own.md", "y")
            .with_source("proj/sub/ascribe.toml", "")
            .with_source("proj/sub/b.md", "y")
            .with_source("other/ascribe.toml", "")
            .with_source("other/c.md", "y");
        let found = fs.sources();
        assert_eq!(found.paths, [p("index.md"), p("proj/own.md")]);
        assert_eq!(found.nested, [p("other"), p("proj/sub")]);
        assert_eq!(found.own_folder, Some(p("proj")));

        let dir = std::env::temp_dir().join(format!("ascribe-resolve-own-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for sub in ["ws/proj/sub", "ws/other"] {
            fs::create_dir_all(at(&dir, sub)).expect("create dirs");
        }
        for (file, text) in [
            ("ws/index.md", "x"),
            ("ws/proj/ascribe.toml", ""),
            ("ws/proj/own.md", "y"),
            ("ws/proj/sub/ascribe.toml", ""),
            ("ws/proj/sub/b.md", "y"),
            ("ws/other/ascribe.toml", ""),
            ("ws/other/c.md", "y"),
        ] {
            fs::write(at(&dir, file), text).expect("write");
        }
        let found = DiskFs::new(at(&dir, "ws/proj"), &layout).sources();
        assert_eq!(found.paths, [p("index.md"), p("proj/own.md")]);
        assert_eq!(found.nested, [p("other"), p("proj/sub")]);
        assert_eq!(found.own_folder, Some(p("proj")));
        let _ = fs::remove_dir_all(&dir);
    }

    // Unix only: it locks the folder with a mode, which Windows ignores.
    #[cfg(unix)]
    #[test]
    fn an_unreadable_directory_is_reported_not_skipped() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("ascribe-resolve-perm-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("docs/locked")).expect("create dirs");
        fs::write(dir.join("docs/index.md"), "x").expect("write");
        fs::write(dir.join("docs/locked/a.md"), "y").expect("write");
        let locked = dir.join("docs/locked");
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).expect("chmod");
        let readable_anyway = fs::read_dir(&locked).is_ok(); // running as root
        let found = DiskFs::new(&dir, &layout()).sources();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).expect("chmod");
        let _ = fs::remove_dir_all(&dir);
        if !readable_anyway {
            assert_eq!(found.paths, [p("index.md")]);
            assert_eq!(found.unreadable.len(), 1);
            assert_eq!(found.unreadable[0].path, p("locked"));
        }
    }

    #[test]
    fn disk_fs_reads_a_real_directory() {
        let dir = std::env::temp_dir().join(format!("ascribe-resolve-fs-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("docs/_f")).expect("create dirs");
        fs::write(dir.join("docs/index.md"), "hi").expect("write");
        fs::write(dir.join("docs/_f/a.md"), "a").expect("write");
        fs::write(dir.join("docs/p.png"), "").expect("write");
        fs::write(dir.join("docs/notes.txt"), "").expect("write");

        let disk = DiskFs::new(&dir, &layout());
        assert_eq!(disk.sources().paths, [p("_f/a.md"), p("index.md")]);
        assert_eq!(disk.read(&p("_f/a.md")).ok().as_deref(), Some("a"));
        assert_eq!(disk.probe(&p("docs/p.png")), Probe::File);
        assert_eq!(disk.probe(&p("docs/missing.png")), Probe::Missing);
        // A directory isn't a file.
        assert_eq!(disk.probe(&p("docs/_f")), Probe::Missing);
        // The directory listing is what's compared, so this is found as a case
        // mismatch even on a case-insensitive file system.
        match disk.probe(&p("docs/P.png")) {
            Probe::CaseMismatch(actual) => assert_eq!(actual, p("docs/p.png")),
            other => panic!("expected a case mismatch, got {other:?}"),
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
