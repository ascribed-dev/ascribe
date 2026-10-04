//! Reading a project from a git revision, in a temporary repository.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::Command;

use tessera_core::RelPath;
use tessera_diff::{DiffError, Repository, Revision};
use tessera_resolve::{FileSystem, Probe};

const MODEL: &str = "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n[builds.site]\n";

/// A temporary repository with a `main` branch.
struct Repo {
    dir: tempfile::TempDir,
}

impl Repo {
    fn new() -> Repo {
        let repo = Repo {
            dir: tempfile::tempdir().unwrap(),
        };
        repo.git(&["init", "-q", "-b", "main"]);
        repo
    }

    fn root(&self) -> PathBuf {
        self.dir.path().to_path_buf()
    }

    fn path(&self, rel: &str) -> PathBuf {
        rel.split('/').fold(self.root(), |p, s| p.join(s))
    }

    fn git(&self, args: &[&str]) -> String {
        let out = Command::new("git")
            .current_dir(self.dir.path())
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.com",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "core.autocrlf=false",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap().trim().to_owned()
    }

    fn write(&self, rel: &str, text: &str) {
        let path = self.path(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    fn commit(&self, message: &str) -> String {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "--allow-empty", "-m", message]);
        self.git(&["rev-parse", "HEAD"])
    }
}

fn rel(s: &str) -> RelPath {
    RelPath::parse(s).unwrap()
}

fn sources(revision: &Revision) -> Vec<String> {
    revision
        .fs
        .sources()
        .paths
        .iter()
        .map(ToString::to_string)
        .collect()
}

fn read_at(dir: &Path, commit: &str) -> Revision {
    let repo = Repository::discover(dir).unwrap();
    Revision::read(&repo, commit).unwrap().expect("a project")
}

#[test]
fn a_project_at_the_root_of_the_repository() {
    let repo = Repo::new();
    repo.write("ascribe.toml", MODEL);
    repo.write("docs/index.md", "# Home\n");
    repo.write("docs/guides/install.md", "# Install\n");
    repo.write("docs/.hidden/skip.md", "# Hidden\n");
    repo.write("docs/notes.txt", "not a source\n");
    let commit = repo.commit("first");
    // The working tree moves on; the revision doesn't.
    repo.write("docs/index.md", "# Changed\n");

    let found = Repository::discover(&repo.root()).unwrap();
    assert_eq!(found.prefix, "");
    let revision = read_at(&repo.root(), &commit);
    assert_eq!(revision.model_text, MODEL);
    assert_eq!(sources(&revision), ["guides/install.md", "index.md"]);
    assert_eq!(revision.fs.read(&rel("index.md")).unwrap(), "# Home\n");
    let project = revision.project();
    assert_eq!(project.pages().count(), 2);
}

#[test]
fn a_project_in_a_subfolder() {
    let repo = Repo::new();
    repo.write("README.md", "# Not docs\n");
    repo.write("site/ascribe.toml", MODEL);
    repo.write("site/docs/index.md", "# Home\n");
    repo.write("site/docs/img/Logo.png", "");
    let commit = repo.commit("first");

    let dir = repo.path("site");
    let found = Repository::discover(&dir).unwrap();
    assert_eq!(found.prefix, "site/");
    let revision = read_at(&dir, &commit);
    assert_eq!(sources(&revision), ["index.md"]);
    // Probes are relative to the project root, with exact names.
    assert_eq!(revision.fs.probe(&rel("docs/img/Logo.png")), Probe::File);
    assert_eq!(
        revision.fs.probe(&rel("docs/img/logo.png")),
        Probe::CaseMismatch(rel("docs/img/Logo.png"))
    );
    assert_eq!(revision.fs.probe(&rel("docs/img/none.png")), Probe::Missing);
    assert_eq!(revision.fs.probe(&rel("../README.md")), Probe::File);
    assert_eq!(revision.fs.probe(&rel("../../outside.md")), Probe::Missing);
}

#[test]
fn a_content_root_above_the_project_folder() {
    let repo = Repo::new();
    repo.write(
        "docs/config/ascribe.toml",
        "spec = \"0.1\"\n[project]\ncontent-root = \"..\"\n[builds.site]\n",
    );
    repo.write("docs/index.md", "# Home\n");
    repo.write("docs/config/notes.md", "# Kept: the project's own folder\n");
    let commit = repo.commit("first");

    let revision = read_at(&repo.path("docs/config"), &commit);
    assert_eq!(sources(&revision), ["config/notes.md", "index.md"]);
    assert_eq!(revision.fs.sources().own_folder, Some(rel("config")));
}

#[test]
fn a_content_root_above_the_repository_is_an_error() {
    let repo = Repo::new();
    repo.write(
        "ascribe.toml",
        "spec = \"0.1\"\n[project]\ncontent-root = \"..\"\n[builds.site]\n",
    );
    let commit = repo.commit("first");
    let found = Repository::discover(&repo.root()).unwrap();
    assert!(matches!(
        Revision::read(&found, &commit),
        Err(DiffError::OutsideRepository(_))
    ));
}

#[test]
fn a_nested_project_is_skipped() {
    let repo = Repo::new();
    repo.write("ascribe.toml", MODEL);
    repo.write("docs/index.md", "# Home\n");
    repo.write("docs/other/ascribe.toml", MODEL);
    repo.write("docs/other/docs/page.md", "# Theirs\n");
    repo.write("docs/other/top.md", "# Theirs too\n");
    let commit = repo.commit("first");

    let revision = read_at(&repo.root(), &commit);
    assert_eq!(sources(&revision), ["index.md"]);
    assert_eq!(revision.fs.sources().nested, [rel("other")]);
}

#[test]
fn paths_with_spaces_and_non_ascii_characters() {
    let repo = Repo::new();
    repo.write("ascribe.toml", MODEL);
    repo.write("docs/my guides/café crème.md", "# Café\n");
    let commit = repo.commit("first");

    let revision = read_at(&repo.root(), &commit);
    assert_eq!(sources(&revision), ["my guides/café crème.md"]);
    assert_eq!(
        revision.fs.read(&rel("my guides/café crème.md")).unwrap(),
        "# Café\n"
    );
}

#[test]
fn a_renamed_file_is_at_its_old_path_in_the_old_revision() {
    let repo = Repo::new();
    repo.write("ascribe.toml", MODEL);
    repo.write("docs/old-name.md", "# Page\n");
    let first = repo.commit("first");
    repo.git(&["mv", "docs/old-name.md", "docs/new-name.md"]);
    let second = repo.commit("rename");

    assert_eq!(sources(&read_at(&repo.root(), &first)), ["old-name.md"]);
    assert_eq!(sources(&read_at(&repo.root(), &second)), ["new-name.md"]);
}

#[test]
fn no_project_at_the_revision() {
    let repo = Repo::new();
    repo.write("README.md", "# Before the docs\n");
    let first = repo.commit("first");
    repo.write("ascribe.toml", MODEL);
    repo.commit("add docs");

    let found = Repository::discover(&repo.root()).unwrap();
    assert!(Revision::read(&found, &first).unwrap().is_none());
}

#[test]
fn a_base_model_with_errors() {
    let repo = Repo::new();
    repo.write(
        "ascribe.toml",
        "spec = \"0.1\"\n[builds.site]\nunknown-key = 3\n",
    );
    let commit = repo.commit("first");
    let found = Repository::discover(&repo.root()).unwrap();
    assert!(matches!(
        Revision::read(&found, &commit),
        Err(DiffError::BaseModel { .. })
    ));
}

#[test]
fn no_repository() {
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(
        Repository::discover(dir.path()),
        Err(DiffError::NotARepository { .. })
    ));
}

#[test]
fn the_default_base_is_the_merge_base_with_main() {
    let repo = Repo::new();
    repo.write("ascribe.toml", MODEL);
    let fork = repo.commit("first");
    repo.git(&["checkout", "-q", "-b", "feature"]);
    repo.write("docs/index.md", "# Feature\n");
    repo.commit("on the branch");
    repo.git(&["checkout", "-q", "main"]);
    repo.write("docs/later.md", "# Later on main\n");
    let main = repo.commit("on main");
    repo.git(&["checkout", "-q", "feature"]);

    let found = Repository::discover(&repo.root()).unwrap();
    let base = found.base(None, false).unwrap();
    assert_eq!(base.requested, "main");
    assert_eq!(base.commit, main);
    assert_eq!(base.merge_base.as_deref(), Some(fork.as_str()));
    assert_eq!(base.compared(), fork);

    let exact = found.base(Some("main"), true).unwrap();
    assert_eq!(exact.merge_base, None);
    assert_eq!(exact.compared(), main);
}

#[test]
fn an_unknown_revision() {
    let repo = Repo::new();
    repo.write("ascribe.toml", MODEL);
    repo.commit("first");
    let found = Repository::discover(&repo.root()).unwrap();
    assert!(matches!(
        found.base(Some("no-such-branch"), false),
        Err(DiffError::UnknownRevision(rev)) if rev == "no-such-branch"
    ));
    assert!(matches!(
        found.base(Some("--output=x"), false),
        Err(DiffError::UnknownRevision(_))
    ));
}

#[test]
fn no_default_branch() {
    let repo = Repo::new();
    repo.git(&["checkout", "-q", "-b", "trunk"]);
    repo.write("ascribe.toml", MODEL);
    repo.commit("first");
    let found = Repository::discover(&repo.root()).unwrap();
    assert!(matches!(
        found.base(None, false),
        Err(DiffError::NoDefaultBranch)
    ));
}
