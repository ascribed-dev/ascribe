//! The git guard of the commands that write, by running the binary: each
//! refuses to change a file git can't put back, and names it; `--dry-run`
//! says what would change and changes nothing; and `--force` writes it
//! (decision #965). The guard's own cases, a file changed, untracked, new,
//! or in no repository, are `guard.rs`'s unit tests.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const MODEL: &str = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n";

fn write(root: &Path, rel: &str, text: &str) {
    let path: PathBuf = rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
    fs::create_dir_all(path.parent().expect("a parent")).expect("create directories");
    fs::write(path, text).expect("write a file");
}

fn read(root: &Path, rel: &str) -> Option<String> {
    fs::read_to_string(root.join(rel)).ok()
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(dir)
        .args([
            "-c",
            "user.name=Mira Okafor",
            "-c",
            "user.email=mira@example.com",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(args)
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn ascribe(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(dir)
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("run ascribe")
}

fn both(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// A project in a repository, everything committed: a home page that links
/// to two pages, one with a permalink, and a long `nav-label`, which is
/// advice.
fn repository() -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let root = dir.path();
    write(root, "ascribe.toml", MODEL);
    write(
        root,
        "docs/index.md",
        "---\ntitle: Home\n---\n\nSee [sync](sync.md) and [keys](keys.md).\n",
    );
    write(
        root,
        "docs/sync.md",
        "---\ntitle: Sync\nnav-label: Synchronize two copies\n---\n\nText.\n",
    );
    write(
        root,
        "docs/keys.md",
        "---\ntitle: Keys\n---\n\n## Rotate keys\n@id {permalink=true}: rotate-keys\n\nText.\n",
    );
    git(root, &["init", "-q"]);
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "Start"]);
    dir
}

/// A command that writes `file`, after `before` leaves that file there,
/// not committed, and gives the command something to change in it. With
/// `--dry-run`, it exits with `dry_run` and says `says`, and its JSON has
/// `written` when `json`.
struct Case {
    args: &'static [&'static str],
    file: &'static str,
    before: fn(&Path),
    dry_run: i32,
    says: &'static str,
    json: bool,
}

fn addresses(root: &Path) {
    write(
        root,
        "ascribe.addresses",
        "version = 1\n\n[moved]\n\n[retired]\n",
    );
}

/// The agent files `sync` writes, committed, then one edited by hand inside
/// its block, so `sync` would put it back.
fn agent_files(root: &Path) {
    assert!(
        ascribe(root, &["agents", "sync", "--force"])
            .status
            .success(),
        "sync"
    );
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "Sync the agent files"]);
    let text = read(root, "AGENTS.md").expect("AGENTS.md");
    // A line just after the block's start marker, the first line.
    let (start, rest) = text.split_once('\n').expect("a marker");
    let edited = format!("{start}\nEdited by hand.\n{rest}");
    write(root, "AGENTS.md", &edited);
}

/// An observation of a test that passes, written and not committed, and an
/// example changed since.
fn observed(root: &Path) {
    write(
        root,
        "ascribe.examples.toml",
        "include = [\"examples/**\"]\ntest = \"echo ok\"\n",
    );
    write(root, "examples/a.sh", "echo a\n");
    assert!(
        ascribe(root, &["observe", "--yes", "--force"])
            .status
            .success(),
        "observe"
    );
    write(root, "examples/a.sh", "echo b\n");
}

const CASES: &[Case] = &[
    Case {
        args: &["baseline", "write"],
        file: "ascribe.baseline",
        dry_run: 0,
        says: "ould ",
        json: true,
        before: |root| {
            assert!(ascribe(root, &["baseline", "write"]).status.success());
            write(
                root,
                "docs/more.md",
                "---\ntitle: More\nnav-label: A label that's long\n---\n\nText.\n",
            );
        },
    },
    Case {
        args: &["baseline", "prune"],
        file: "ascribe.baseline",
        dry_run: 0,
        says: "ould ",
        json: true,
        before: |root| {
            assert!(ascribe(root, &["baseline", "write"]).status.success());
            write(root, "docs/sync.md", "---\ntitle: Sync\n---\n\nText.\n");
        },
    },
    Case {
        args: &["permalinks", "sync"],
        file: "ascribe.permalinks",
        dry_run: 0,
        says: "ould ",
        json: true,
        before: |root| {
            assert!(ascribe(root, &["permalinks", "sync"]).status.success());
            write(
                root,
                "docs/sync.md",
                "---\ntitle: Sync\npermalink: sync\nnav-label: Synchronize two copies\n---\n\nText.\n",
            );
        },
    },
    Case {
        args: &[
            "permalinks",
            "retire",
            "--page",
            "docs/keys.md",
            "--reason",
            "Gone",
        ],
        file: "ascribe.addresses",
        before: addresses,
        dry_run: 0,
        says: "ould ",
        json: true,
    },
    Case {
        args: &["addresses", "move", "docs/old.md", "docs/keys.md"],
        file: "ascribe.addresses",
        before: addresses,
        dry_run: 0,
        says: "ould ",
        json: true,
    },
    Case {
        args: &["addresses", "retire", "docs/old.md", "--reason", "Gone"],
        file: "ascribe.addresses",
        before: addresses,
        dry_run: 0,
        says: "ould ",
        json: true,
    },
    Case {
        args: &["agents", "sync"],
        file: "AGENTS.md",
        before: agent_files,
        dry_run: 0,
        says: "would write AGENTS.md",
        json: true,
    },
    Case {
        args: &["agents", "index", "--into", "INDEX.md"],
        file: "INDEX.md",
        before: |root| write(root, "INDEX.md", "# The index\n"),
        dry_run: 0,
        says: "would write INDEX.md",
        json: true,
    },
    Case {
        args: &["observe", "--yes"],
        file: "examples.observations/default.toml",
        before: observed,
        dry_run: 0,
        says: "Would write examples.observations/default.toml",
        json: true,
    },
];

#[test]
fn each_command_refuses_a_file_git_cant_put_back_and_dry_run_and_force_get_past() {
    for case in CASES {
        let dir = repository();
        let root = dir.path();
        (case.before)(root);
        let before = read(root, case.file);

        let out = ascribe(root, case.args);
        assert_eq!(
            out.status.code(),
            Some(2),
            "{:?}: {}",
            case.args,
            both(&out)
        );
        let said = both(&out);
        assert!(
            said.contains(&format!(
                "error: this would change files git can't put back as they are: {}; commit or stash them, or pass --force",
                case.file
            )),
            "{:?}: {said}",
            case.args
        );
        assert_eq!(read(root, case.file), before, "{:?}", case.args);

        let dry_run = ascribe(root, &[case.args, &["--dry-run"]].concat());
        assert_eq!(
            dry_run.status.code(),
            Some(case.dry_run),
            "{:?}: {}",
            case.args,
            both(&dry_run)
        );
        assert!(
            both(&dry_run).contains(case.says),
            "{:?}: {}",
            case.args,
            both(&dry_run)
        );
        assert_eq!(read(root, case.file), before, "{:?}", case.args);
        let json = ascribe(
            root,
            &[case.args, &["--dry-run", "--format", "json"]].concat(),
        );
        let json: serde_json::Value = serde_json::from_slice(&json.stdout).expect("JSON");
        if case.json {
            assert_eq!(json["written"], false, "{:?}: {json}", case.args);
        }

        let force = ascribe(root, &[case.args, &["--force"]].concat());
        assert_eq!(
            force.status.code(),
            Some(0),
            "{:?}: {}",
            case.args,
            both(&force)
        );
        assert_ne!(read(root, case.file), before, "{:?}", case.args);
    }
}

#[test]
fn a_file_committed_and_unchanged_is_written_without_force() {
    let dir = repository();
    let root = dir.path();
    addresses(root);
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "Record addresses"]);
    let out = ascribe(
        root,
        &["addresses", "retire", "docs/old.md", "--reason", "Gone"],
    );
    assert_eq!(out.status.code(), Some(0), "{}", both(&out));
    assert!(
        read(root, "ascribe.addresses")
            .unwrap()
            .contains("\"old.md\" = \"Gone\""),
        "{}",
        read(root, "ascribe.addresses").unwrap()
    );
}
