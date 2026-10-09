//! `ascribe agents sync` and `ascribe agents skill`, by running the binary
//! on copies of projects in temporary repositories. This repository has an
//! `AGENTS.md` of its own, so nothing here runs on `examples/` in place.
//!
//! `tests/output/agents/` holds what `sync` writes for `examples/quill` and each
//! project of `examples/monorepo`, with every target, apart from the skill
//! (`src/agents/skill.rs` checks it against the npm package's copy). Each
//! file's name ends in `.snap`, so no agent working on this repository takes
//! one for its instructions. Run this test with `ASCRIBE_BLESS=1` to rewrite
//! them.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const BLESS: &str = "ASCRIBE_BLESS=1 cargo test -p ascribe-cli --test all agents::";

const EVERY_TARGET: &[&str] = &[
    "--target",
    "claude",
    "--target",
    "claude-rules",
    "--target",
    "copilot",
];

fn examples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

fn snapshots() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/output/agents")
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

fn git_init(dir: &Path) {
    let out = Command::new("git")
        .current_dir(dir)
        .args(["init", "-q"])
        .output()
        .expect("run git");
    assert!(out.status.success(), "git init failed");
}

/// A temporary git repository holding a copy of an example.
fn repository(example: &str) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    copy_dir(&examples().join(example), dir.path());
    git_init(dir.path());
    dir
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("{} is missing", path.display()))
}

fn ascribe(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(dir)
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("run ascribe")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).replace('\\', "/")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// Runs `ascribe` and expects it to exit with `code`.
fn run(dir: &Path, args: &[&str], code: i32) -> Output {
    let out = ascribe(dir, args);
    assert_eq!(
        out.status.code(),
        Some(code),
        "ascribe {args:?}\nstdout: {}\nstderr: {}",
        stdout(&out),
        stderr(&out),
    );
    out
}

fn sync(dir: &Path, config: &str, more: &[&str]) -> Output {
    let mut args = vec!["agents", "sync", "--config", config];
    args.extend_from_slice(more);
    run(dir, &args, 0)
}

/// Every file under `dir`, `/`-separated and sorted, leaving out `.git`.
fn files(dir: &Path) -> Vec<String> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            if entry.file_name() == ".git" {
                continue;
            }
            if entry.file_type().unwrap().is_dir() {
                walk(root, &entry.path(), out);
            } else {
                let rel = entry.path().strip_prefix(root).unwrap().to_owned();
                let parts: Vec<String> = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect();
                out.push(parts.join("/"));
            }
        }
    }
    let mut out = Vec::new();
    if dir.exists() {
        walk(dir, dir, &mut out);
    }
    out.sort();
    out
}

/// The files `sync` added to a copy of `example`, apart from the skill.
fn written(example: &str, repo: &Path) -> Vec<String> {
    let before = files(&examples().join(example));
    files(repo)
        .into_iter()
        .filter(|f| !before.contains(f))
        .filter(|f| !f.contains("/skills/ascribe/"))
        .collect()
}

/// Compares the files `sync` wrote into a copy of `example` with
/// `tests/agents/<example>/`, or writes them there when blessing.
fn expect_written(example: &str, repo: &Path) {
    let written = written(example, repo);
    let dir = snapshots().join(example);
    if std::env::var_os("ASCRIBE_BLESS").is_some() {
        let _ = fs::remove_dir_all(&dir);
        for file in &written {
            write(&dir.join(format!("{file}.snap")), &read(&repo.join(file)));
        }
        return;
    }
    let snapshots: Vec<String> = written.iter().map(|f| format!("{f}.snap")).collect();
    assert_eq!(
        files(&dir),
        snapshots,
        "tests/output/agents/{example}/ doesn't list the files sync writes; run `{BLESS}`"
    );
    for file in &written {
        let committed = read(&dir.join(format!("{file}.snap"))).replace("\r\n", "\n");
        assert!(
            committed == read(&repo.join(file)),
            "tests/output/agents/{example}/{file} isn't what sync writes; run `{BLESS}` and check the diff\n--- ascribe\n{}",
            read(&repo.join(file))
        );
    }
}

#[test]
fn quill_with_every_target() {
    let repo = repository("quill");
    sync(repo.path(), ".", EVERY_TARGET);
    expect_written("quill", repo.path());
}

#[test]
fn monorepo_with_every_target() {
    let repo = repository("monorepo");
    for project in ["docs", "handbook", "handbook/pages/security"] {
        sync(repo.path(), project, EVERY_TARGET);
    }
    expect_written("monorepo", repo.path());

    // Each project's block names its own frontmatter, not another's.
    let security = read(&repo.path().join("handbook/pages/security/AGENTS.md"));
    let handbook = read(&repo.path().join("handbook/AGENTS.md"));
    assert!(
        handbook.contains("`owner` (string, required)"),
        "{handbook}"
    );
    assert!(
        security.contains("`policy` (files `policies/**`)"),
        "{security}"
    );
    assert!(!handbook.contains("`policy` (files"), "{handbook}");
    assert!(handbook.contains("`{company}`") && !security.contains("`{company}`"));
    // The root holds a short block for each, and Claude Code's skill is
    // loaded for the pages of all three.
    let root = read(&repo.path().join("AGENTS.md"));
    for project in ["docs", "handbook", "handbook/pages/security"] {
        assert!(
            root.contains(&format!("<!-- ascribe:agents:{project} start")),
            "{root}"
        );
        assert!(
            root.contains(&format!("`ascribe check --config {project}`")),
            "{root}"
        );
    }
    let skill = read(&repo.path().join(".claude/skills/ascribe/SKILL.md"));
    assert!(
        skill.contains(
            "paths:\n  - \"docs/content/**/*.md\"\n  - \"handbook/pages/**/*.md\"\n  - \"handbook/pages/security/content/**/*.md\"\n"
        ),
        "{skill}"
    );
}

#[test]
fn a_second_sync_changes_nothing() {
    let repo = repository("quill");
    sync(repo.path(), ".", EVERY_TARGET);
    let before: Vec<(Vec<u8>, String)> = files(repo.path())
        .into_iter()
        .map(|f| (fs::read(repo.path().join(&f)).unwrap(), f))
        .collect();
    let out = sync(repo.path(), ".", &[]);
    let text = stdout(&out);
    assert!(!text.contains("wrote"), "{text}");
    assert!(text.contains("unchanged CLAUDE.md"), "{text}");
    assert!(
        text.contains("unchanged .github/instructions/ascribe.instructions.md"),
        "{text}"
    );
    let after: Vec<(Vec<u8>, String)> = files(repo.path())
        .into_iter()
        .map(|f| (fs::read(repo.path().join(&f)).unwrap(), f))
        .collect();
    assert_eq!(before, after);
    run(repo.path(), &["agents", "sync", "--check"], 0);
}

#[test]
fn check_fails_when_the_content_model_changes() {
    let repo = repository("quill");
    sync(repo.path(), ".", &[]);
    let model = repo.path().join("ascribe.toml");
    write(
        &model,
        &read(&model).replace("description = \"string?\"", "description = \"string\""),
    );
    let out = run(repo.path(), &["agents", "sync", "--check"], 1);
    assert!(
        stdout(&out).contains("stale      AGENTS.md"),
        "{}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("1 file is out of date"),
        "{}",
        stderr(&out)
    );
    assert!(
        !read(&repo.path().join("AGENTS.md")).contains("`description` (string, required)"),
        "--check wrote the file"
    );
    sync(repo.path(), ".", &[]);
    run(repo.path(), &["agents", "sync", "--check"], 0);
}

#[test]
fn the_teams_text_survives() {
    let repo = repository("quill");
    let team = "# Our rules\r\n\r\nUse British spelling.  \r\n";
    let after = "\r\n## More\r\n\r\nNo tables.";
    write(
        &repo.path().join("AGENTS.md"),
        &format!(
            "{team}<!-- ascribe:agents start (old) -->\r\nOld.\r\n<!-- ascribe:agents end -->\r\n{after}"
        ),
    );
    sync(repo.path(), ".", &[]);
    let text = read(&repo.path().join("AGENTS.md"));
    assert!(text.starts_with(team), "{text}");
    assert!(text.ends_with(after), "{text}");
    assert!(text.contains("## Ascribe documentation\r\n"), "{text}");
    assert!(!text.contains("Old."), "{text}");
}

#[test]
fn damaged_markers_stop_everything() {
    let repo = repository("quill");
    let damaged = "# Rules\n\n<!-- ascribe:agents start (x) -->\nOld.\n";
    write(&repo.path().join("AGENTS.md"), damaged);
    let out = run(repo.path(), &["agents", "sync"], 2);
    assert!(
        stderr(&out).contains("its start marker has no end marker after it"),
        "{}",
        stderr(&out)
    );
    assert_eq!(read(&repo.path().join("AGENTS.md")), damaged);
    assert!(!repo.path().join(".agents").exists(), "it wrote the skill");
}

#[test]
fn claude_md_is_updated_only_when_there_is_one() {
    // None: none is made, since one would stop Claude Code reading
    // AGENTS.md by itself.
    let repo = repository("quill");
    sync(repo.path(), ".", &[]);
    assert!(!repo.path().join("CLAUDE.md").exists());
    assert!(!repo.path().join(".claude").exists());

    // One: it imports AGENTS.md, and the skill is written where Claude Code
    // reads skills.
    let repo = repository("quill");
    write(&repo.path().join("CLAUDE.md"), "# Ours\n");
    sync(repo.path(), ".", &[]);
    assert_eq!(
        read(&repo.path().join("CLAUDE.md")),
        "# Ours\n\n<!-- ascribe:agents start (generated by `ascribe agents sync`; it loads AGENTS.md) -->\n@AGENTS.md\n<!-- ascribe:agents end -->\n"
    );
    assert!(repo.path().join(".claude/skills/ascribe/SKILL.md").exists());

    // One in `.claude/`: the import is relative to it.
    let repo = repository("quill");
    write(&repo.path().join(".claude/CLAUDE.md"), "# Ours\n");
    sync(repo.path(), ".", &[]);
    assert!(read(&repo.path().join(".claude/CLAUDE.md")).contains("\n@../AGENTS.md\n"));
    assert!(!repo.path().join("CLAUDE.md").exists());

    // A personal CLAUDE.local.md is left alone, with a note.
    let repo = repository("quill");
    write(&repo.path().join("CLAUDE.local.md"), "Mine.\n");
    let out = sync(repo.path(), ".", &[]);
    assert!(
        stderr(&out).contains("CLAUDE.local.md exists, so Claude Code won't load AGENTS.md"),
        "{}",
        stderr(&out)
    );
    assert_eq!(read(&repo.path().join("CLAUDE.local.md")), "Mine.\n");
}

#[test]
fn two_projects_share_the_root_agents_md() {
    let repo = repository("monorepo");
    write(
        &repo.path().join("AGENTS.md"),
        "# Lantern\n\nRun `go test ./...`.\n",
    );
    sync(repo.path(), "docs", &[]);
    sync(repo.path(), "handbook", &[]);
    let root = read(&repo.path().join("AGENTS.md"));
    assert!(
        root.starts_with("# Lantern\n\nRun `go test ./...`.\n\n<!-- ascribe:agents:docs start")
    );
    assert!(
        root.contains("\n\n<!-- ascribe:agents:handbook start"),
        "{root}"
    );
    assert!(root.len() < 1_500, "the root's blocks are short: {root}");
    // Syncing one again leaves the other alone.
    sync(repo.path(), "docs", &[]);
    assert_eq!(read(&repo.path().join("AGENTS.md")), root);
}

#[test]
fn the_rules_fit_their_budget() {
    let repo = repository("quill");
    sync(repo.path(), ".", &[]);
    let text = read(&repo.path().join("AGENTS.md"));
    assert!(text.chars().count() <= 4_200, "{}", text.chars().count());
    assert!(text.contains(
        "Programmatic check: after editing a page, run `ascribe check <file> --format concise`"
    ));
    // Quill writes `@variant`, so it's told about it, with its dimensions.
    assert!(text.contains("`@variant {<dimension>=<value>}:`"), "{text}");
    assert!(text.contains("dimensions `pm`, `deployment`"), "{text}");

    // A project with no variants isn't told about them.
    let repo = repository("monorepo");
    sync(repo.path(), "handbook", &[]);
    let text = read(&repo.path().join("handbook/AGENTS.md"));
    assert!(!text.contains("@variant"), "{text}");
}

#[test]
fn outside_a_repository_the_projects_folder_is_the_root() {
    let dir = tempfile::tempdir().unwrap();
    copy_dir(&examples().join("monorepo/handbook"), dir.path());
    if Command::new("git")
        .current_dir(dir.path())
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .is_ok_and(|out| out.status.success())
    {
        // The temporary directory is inside a repository on this machine.
        return;
    }
    let out = sync(dir.path(), ".", &[]);
    assert!(
        stderr(&out).contains(
            "isn't in a git repository, so its folder is treated as the repository's root"
        ),
        "{}",
        stderr(&out)
    );
    assert!(dir.path().join("AGENTS.md").exists());
    assert!(dir.path().join(".agents/skills/ascribe/SKILL.md").exists());

    let out = run(dir.path(), &["agents", "sync", "--target", "copilot"], 2);
    assert!(
        stderr(&out).contains("--target copilot needs a git repository"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn a_file_that_would_be_a_page_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    write(
        &dir.path().join("ascribe.toml"),
        "spec = \"0.1\"\n\n[project]\ncontent-root = \".\"\noutput-dir = \"../build\"\n",
    );
    write(&dir.path().join("index.md"), "# Home\n");
    git_init(dir.path());
    let out = run(dir.path(), &["agents", "sync"], 2);
    assert!(
        stderr(&out)
            .contains("it's under the content root, so it would be one of the project's pages"),
        "{}",
        stderr(&out)
    );
    assert!(!dir.path().join("AGENTS.md").exists());
}

#[test]
fn skill_prints_the_packaged_skill() {
    let out = run(
        Path::new(env!("CARGO_MANIFEST_DIR")),
        &["agents", "skill"],
        0,
    );
    let packaged = read(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/cli/skills/ascribe/SKILL.md"),
    )
    .replace("\r\n", "\n");
    assert_eq!(String::from_utf8_lossy(&out.stdout), packaged);
}
