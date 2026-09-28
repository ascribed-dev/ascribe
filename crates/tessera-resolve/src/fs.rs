//! The file system the source index reads, and two implementations.
//!
//! Everything the index needs from disk goes through [`FileSystem`], so the
//! language server can index unsaved buffers, and tests can build a project in
//! memory. A [`DiskFs`] reads a real project; a [`MemoryFs`] holds one.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use tessera_core::RelPath;

use crate::layout::Layout;

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

/// The files of a project.
pub trait FileSystem {
    /// Every `.md` file under the content root, as content paths, in any
    /// order. Directories are followed through symbolic links once each.
    fn sources(&self) -> Vec<RelPath>;

    /// The text of a source file, by content path.
    fn read(&self, path: &RelPath) -> io::Result<String>;

    /// Whether a file exists at a path relative to the project root. The
    /// path may start with `..` when the content root is above the project
    /// root; the boundary check has already decided the path may be read.
    fn probe(&self, project_path: &RelPath) -> Probe;
}

/// A project on disk.
#[derive(Clone, Debug)]
pub struct DiskFs {
    project_root: PathBuf,
    content_root: RelPath,
}

impl DiskFs {
    /// The project in `project_root` (the directory with `tessera.toml`).
    pub fn new(project_root: impl Into<PathBuf>, layout: &Layout) -> DiskFs {
        DiskFs {
            project_root: project_root.into(),
            content_root: layout.content_root.clone(),
        }
    }

    fn content_dir(&self) -> PathBuf {
        self.project_root.join(self.content_root.as_str())
    }
}

impl FileSystem for DiskFs {
    fn sources(&self) -> Vec<RelPath> {
        let mut out = Vec::new();
        let mut seen = BTreeSet::new();
        walk(&self.content_dir(), &RelPath::root(), &mut seen, &mut out);
        out.sort();
        out
    }

    fn read(&self, path: &RelPath) -> io::Result<String> {
        fs::read_to_string(self.content_dir().join(path.as_str()))
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
            let Ok(entries) = fs::read_dir(&dir) else {
                return Probe::Missing;
            };
            let mut exact = None;
            let mut folded = None;
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name == *segment {
                    exact = Some(name);
                    break;
                }
                if folded.is_none() && name.to_lowercase() == segment.to_lowercase() {
                    folded = Some(name);
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
}

/// Collects `.md` files under `dir`, as paths relative to the content root.
/// `seen` holds the canonical directories already walked, so a symbolic link
/// back up the tree can't loop.
fn walk(dir: &Path, rel: &RelPath, seen: &mut BTreeSet<PathBuf>, out: &mut Vec<RelPath>) {
    let Ok(canonical) = fs::canonicalize(dir) else {
        return;
    };
    if !seen.insert(canonical) {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();
        // `metadata` follows symbolic links.
        let Ok(meta) = fs::metadata(&path) else {
            continue;
        };
        let Ok(child) = rel.join(&name) else {
            continue;
        };
        if meta.is_dir() {
            walk(&path, &child, seen, out);
        } else if meta.is_file() && name.ends_with(".md") {
            out.push(child);
        }
    }
}

/// A project held in memory: every file, source or not, by path relative to
/// the project root.
#[derive(Clone, Debug, Default)]
pub struct MemoryFs {
    content_root: RelPath,
    files: BTreeMap<RelPath, String>,
}

impl MemoryFs {
    /// An empty project with this layout.
    pub fn new(layout: &Layout) -> MemoryFs {
        MemoryFs {
            content_root: layout.content_root.clone(),
            files: BTreeMap::new(),
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
    fn sources(&self) -> Vec<RelPath> {
        self.files
            .keys()
            .filter(|p| p.extension() == Some("md"))
            .filter_map(|p| self.content_path(p))
            .collect()
    }

    fn read(&self, path: &RelPath) -> io::Result<String> {
        self.content_root
            .join(path.as_str())
            .ok()
            .and_then(|p| self.files.get(&p))
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such file"))
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
            output_dir: p(".tessera/build"),
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
        let mut sources = fs.sources();
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
    fn disk_fs_reads_a_real_directory() {
        let dir = std::env::temp_dir().join(format!("tessera-resolve-fs-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("docs/_f")).expect("create dirs");
        fs::write(dir.join("docs/index.md"), "hi").expect("write");
        fs::write(dir.join("docs/_f/a.md"), "a").expect("write");
        fs::write(dir.join("docs/p.png"), "").expect("write");
        fs::write(dir.join("docs/notes.txt"), "").expect("write");

        let disk = DiskFs::new(&dir, &layout());
        assert_eq!(disk.sources(), [p("_f/a.md"), p("index.md")]);
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
