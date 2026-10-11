//! A repository with a folder committed in it that's shaped like a bare
//! repository, whose configuration names a command, and a project in that
//! folder: what `git`, started by Ascribe in the project, must never take
//! for the repository. `ascribe-diff`'s tests and `ascribe-cli`'s include
//! it with `#[path]`.

#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository, the project in its planted folder, and where the
/// planted command would leave its mark.
pub struct Planted {
    /// The repository.
    pub dir: tempfile::TempDir,
    /// Outside the repository: where the command writes `ran`.
    marks: tempfile::TempDir,
}

impl Planted {
    /// A repository with `files` committed under `site/`, which is also a
    /// bare repository's folder: `HEAD`, `objects/` with a commit, `refs/`,
    /// and a `config` that makes `site/` its own working tree and names a
    /// file system monitor, a command `git diff` and `git status` run.
    pub fn new(files: &[(&str, &str)]) -> Planted {
        let dir = tempfile::tempdir().unwrap();
        let marks = tempfile::tempdir().unwrap();
        let root = dir.path();
        git(root, &["init", "-q", "-b", "main"]);
        git(root, &["init", "-q", "--bare", "-b", "main", "site"]);
        let site = root.join("site");
        // A commit in the planted repository, so its `HEAD` names one.
        let work = tempfile::tempdir().unwrap();
        write(
            work.path(),
            "ascribe.toml",
            files.first().map_or("", |f| f.1),
        );
        let git_dir = format!("--git-dir={}", site.display());
        let work_tree = format!("--work-tree={}", work.path().display());
        git(root, &[&git_dir, &work_tree, "add", "-A"]);
        git(
            root,
            &[&git_dir, &work_tree, "commit", "-q", "-m", "planted"],
        );
        let mark = marks.path().join("ran").display().to_string();
        let mark = mark.replace('\\', "/");
        std::fs::write(
            site.join("config"),
            format!(
                "[core]\n\trepositoryformatversion = 0\n\tbare = false\n\tworktree = .\n\
                 \tfsmonitor = \"touch '{mark}' #\"\n"
            ),
        )
        .unwrap();
        for (path, text) in files {
            write(&site, path, text);
        }
        git(root, &["add", "-A"]);
        git(root, &["commit", "-q", "-m", "start"]);
        Planted { dir, marks }
    }

    /// The project's folder, which is the planted one.
    pub fn project(&self) -> PathBuf {
        self.dir.path().join("site")
    }

    /// Whether the planted command ran.
    pub fn ran(&self) -> bool {
        self.marks.path().join("ran").exists()
    }
}

fn write(dir: &Path, rel: &str, text: &str) {
    let path = rel.split('/').fold(dir.to_path_buf(), |p, s| p.join(s));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// Runs `git` for the fixture, with a fixed author and none of the
/// options under test.
fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(dir)
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
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
