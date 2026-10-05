//! A code repository and a docs project in temporary folders, for the
//! commands that copy files from one to the other. The code repository is
//! reached by a `file://` URL: a real `git` fetch, with no network.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use tessera_core::{FileId, RelPath};
use tessera_resolve::{DiskFs, Layout};
use tessera_sources::{Options, Workspace};

/// Runs `git` in `dir`, with a fixed identity and no user configuration.
pub fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .current_dir(dir)
        .args(["-c", "user.name=Test", "-c", "user.email=test@example.com"])
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

/// A path on disk, from `/`-separated segments.
pub fn path(root: &Path, rel: &str) -> PathBuf {
    rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s))
}

pub fn write(root: &Path, rel: &str, text: impl AsRef<[u8]>) {
    let path = path(root, rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

pub fn read(root: &Path, rel: &str) -> Option<String> {
    std::fs::read_to_string(path(root, rel)).ok()
}

/// The `file://` URL of a folder.
pub fn file_url(dir: &Path) -> String {
    let text = dir.display().to_string().replace('\\', "/");
    if text.starts_with('/') {
        format!("file://{text}")
    } else {
        format!("file:///{text}")
    }
}

/// A repository of code: a `main` branch, committed to as a test goes.
pub struct Code {
    pub dir: tempfile::TempDir,
}

impl Code {
    pub fn new() -> Code {
        let code = Code {
            dir: tempfile::tempdir().unwrap(),
        };
        git(code.root(), &["init", "-q", "-b", "main"]);
        // Fetching one blob by name, as a partial clone does.
        git(code.root(), &["config", "uploadpack.allowFilter", "true"]);
        git(
            code.root(),
            &["config", "uploadpack.allowAnySHA1InWant", "true"],
        );
        code
    }

    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    pub fn url(&self) -> String {
        file_url(self.root())
    }

    pub fn write(&self, rel: &str, text: impl AsRef<[u8]>) -> &Code {
        write(self.root(), rel, text);
        self
    }

    /// Tags a commit.
    pub fn git_tag(&self, name: &str, commit: &str) {
        git(self.root(), &["tag", name, commit]);
    }

    /// Commits everything, and returns the commit.
    pub fn commit(&self, message: &str) -> String {
        git(self.root(), &["add", "-A"]);
        git(
            self.root(),
            &["commit", "-q", "--allow-empty", "-m", message],
        );
        git(self.root(), &["rev-parse", "HEAD"])
    }
}

/// A docs project: `ascribe.toml`, pages under `docs/`, and a cache.
pub struct Docs {
    pub dir: tempfile::TempDir,
    pub cache: tempfile::TempDir,
}

impl Docs {
    /// A project whose `[sources]` are `sources` (TOML).
    pub fn new(sources: &str) -> Docs {
        let docs = Docs {
            dir: tempfile::tempdir().unwrap(),
            cache: tempfile::tempdir().unwrap(),
        };
        write(
            docs.root(),
            "ascribe.toml",
            format!("spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n{sources}"),
        );
        std::fs::create_dir_all(docs.root().join("docs")).unwrap();
        docs
    }

    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    pub fn page(&self, rel: &str, text: &str) -> &Docs {
        write(self.root(), &format!("docs/{rel}"), text);
        self
    }

    pub fn options(&self) -> Options {
        Options {
            cache: self.cache.path().to_path_buf(),
        }
    }

    /// The project as the commands see it, read from disk now.
    pub fn workspace(&self) -> Workspace {
        let text = std::fs::read_to_string(self.root().join("ascribe.toml")).unwrap();
        let model = tessera_model::load_str(&text, FileId::new(0)).unwrap();
        let layout = Layout {
            content_root: RelPath::parse(&model.project.content_root).unwrap(),
            output_dir: RelPath::parse(&model.project.output_dir).unwrap(),
        };
        let fs = DiskFs::new(self.root(), &layout);
        let project = tessera_resolve::Project::load(Arc::new(model.clone()), layout, &fs);
        Workspace::new(self.root(), &model, &project).unwrap()
    }

    pub fn read(&self, rel: &str) -> Option<String> {
        read(self.root(), rel)
    }

    pub fn exists(&self, rel: &str) -> bool {
        path(self.root(), rel).exists()
    }

    /// The diagnostics' slugs of `ascribe check`'s file-level checks.
    pub fn check(&self) -> Vec<String> {
        let project = tessera_check::Project::load(&self.root().join("ascribe.toml")).unwrap();
        tessera_check::check_files(&project)
            .into_iter()
            .map(|d| format!("{} {}", d.slug.as_str(), d.message))
            .collect()
    }
}
