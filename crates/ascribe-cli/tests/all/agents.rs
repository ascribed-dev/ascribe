//! `ascribe agents sync`, `ascribe agents index`, and `ascribe agents skill`, by running the binary
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

use crate::links;

const BLESS: &str = "ASCRIBE_BLESS=1 cargo test -p ascribe-cli --test all agents::";

const EVERY_TARGET: &[&str] = &[
    "--target",
    "claude",
    "--target",
    "claude-rules",
    "--target",
    "copilot",
    "--cloud",
    "--agent",
];

/// The setup steps install the version of `@ascribed/cli` that wrote them;
/// the snapshots say `VERSION`, so a release doesn't change them.
fn without_version(text: &str) -> String {
    text.replace(
        &format!("@ascribed/cli@{}", env!("CARGO_PKG_VERSION")),
        "@ascribed/cli@VERSION",
    )
}

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
            write(
                &dir.join(format!("{file}.snap")),
                &without_version(&read(&repo.join(file))),
            );
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
            committed == without_version(&read(&repo.join(file))),
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
    assert!(security.contains("- `policy`: frontmatter"), "{security}");
    assert!(!handbook.contains("- `policy`:"), "{handbook}");
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
fn at_a_root_with_several_projects_each_is_synced() {
    // Where there's no project, the error names those below.
    let repo = repository("monorepo");
    let out = run(repo.path(), &["check"], 2);
    assert!(
        stderr(&out).contains(
            "the projects below it are in docs, handbook, handbook/pages/security: run ascribe \
             in one of them, or pass --config with its folder"
        ),
        "{}",
        stderr(&out)
    );

    // `sync` writes what syncing each in turn writes, and lists each file
    // once.
    let out = run(repo.path(), &["agents", "sync", "--check"], 1);
    assert!(
        stdout(&out).contains("stale      AGENTS.md\n"),
        "{}",
        stdout(&out)
    );
    let mut args = vec!["agents", "sync"];
    args.extend_from_slice(EVERY_TARGET);
    let out = run(repo.path(), &args, 0);
    assert!(
        stderr(&out).contains("syncs each project below: docs, handbook, handbook/pages/security"),
        "{}",
        stderr(&out)
    );
    let text = stdout(&out);
    let root_agents_md = text
        .lines()
        .filter(|l| l.split_whitespace().nth(1) == Some("AGENTS.md"))
        .count();
    assert_eq!(root_agents_md, 1, "{text}");
    assert!(
        text.contains("wrote     handbook/pages/security/AGENTS.md"),
        "{text}"
    );
    expect_written("monorepo", repo.path());
    let out = run(repo.path(), &["agents", "sync", "--check"], 0);
    assert!(!stdout(&out).contains("stale"), "{}", stdout(&out));
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
fn each_file_says_who_reads_it() {
    let repo = repository("quill");
    let root = repo.path();
    let text = stdout(&sync(root, ".", &[]));
    let line = |path: &str| line_of(&text, path);
    assert!(
        line("AGENTS.md").ends_with("  read by Claude Code, Codex, Copilot, and Cursor"),
        "{text}"
    );
    assert!(
        line(".agents/skills/ascribe/SKILL.md")
            .ends_with("  read by Codex, Copilot, Cursor, and Gemini CLI"),
        "{text}"
    );
    // The skill Claude Code doesn't read, and the target that writes the
    // one it does.
    let missing = "Claude Code doesn't read the skill in .agents/skills/; `--target claude`";
    assert!(text.contains(missing), "{text}");
    let text = stdout(&sync(root, ".", &["--target", "claude"]));
    assert!(
        line_of(&text, ".claude/skills/ascribe/SKILL.md").ends_with("  read by Claude Code"),
        "{text}"
    );
    assert!(!text.contains(missing), "{text}");
    // `--check` says only whether each file is current.
    let check = stdout(&run(root, &["agents", "sync", "--check"], 0));
    assert!(!check.contains("read by"), "{check}");
}

/// The line `sync` printed about `path`.
fn line_of(text: &str, path: &str) -> String {
    text.lines()
        .find(|l| l.split_whitespace().nth(1) == Some(path))
        .unwrap_or_else(|| panic!("no {path}: {text}"))
        .to_owned()
}

#[test]
fn check_fails_when_the_content_model_changes() {
    let repo = repository("quill");
    sync(repo.path(), ".", &[]);
    let model = repo.path().join("ascribe.toml");
    write(
        &model,
        &read(&model).replace("type = \"string?\", role", "type = \"string\", role"),
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
    assert!(root.len() < 1_700, "the root's blocks are short: {root}");
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

    for args in [["--target", "copilot"].as_slice(), &["--cloud"]] {
        let mut all = vec!["agents", "sync"];
        all.extend_from_slice(args);
        let out = run(dir.path(), &all, 2);
        assert!(
            stderr(&out).contains("--target copilot and --cloud need a git repository"),
            "{}",
            stderr(&out)
        );
    }
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

#[test]
fn with_hook_writes_each_agents_hooks() {
    let repo = repository("quill");
    let root = repo.path();
    write(
        &root.join(".claude/settings.json"),
        "{\n  \"model\": \"opus\",\n  \"hooks\": {\n    \"PostToolUse\": [\n      { \"matcher\": \"Write\", \"hooks\": [ { \"type\": \"command\", \"command\": \"prettier --write\" } ] }\n    ]\n  }\n}\n",
    );
    write(
        &root.join(".mcp.json"),
        "{\"mcpServers\": {\"other\": {\"command\": \"other\"}}}\n",
    );
    let mut args = EVERY_TARGET.to_vec();
    args.extend(["--target", "codex", "--with-hook"]);
    let out = sync(root, ".", &args);
    let text = stdout(&out);
    for file in [
        ".claude/settings.json",
        ".mcp.json",
        ".codex/hooks.json",
        ".github/hooks/ascribe.json",
    ] {
        assert!(text.contains(&format!("wrote     {file}")), "{text}");
    }
    let json = |file: &str| -> serde_json::Value {
        serde_json::from_str(&read(&root.join(file))).unwrap()
    };
    let claude = json(".claude/settings.json");
    assert_eq!(claude["model"], "opus");
    let post = &claude["hooks"]["PostToolUse"];
    assert_eq!(post[0]["hooks"][0]["command"], "prettier --write");
    assert_eq!(post[1]["matcher"], "Write|Edit|MultiEdit");
    assert_eq!(
        post[1]["hooks"][0]["command"],
        "ascribe agents hook claude-code --event edit"
    );
    assert_eq!(
        claude["hooks"]["Stop"][0]["hooks"][0]["command"],
        "ascribe agents hook claude-code --event stop"
    );
    // The team's keys keep their order.
    let settings = read(&root.join(".claude/settings.json"));
    assert!(
        settings.find("\"model\"") < settings.find("\"hooks\""),
        "{settings}"
    );
    let mcp = json(".mcp.json");
    assert_eq!(mcp["mcpServers"]["other"]["command"], "other");
    assert_eq!(mcp["mcpServers"]["ascribe"]["args"][0], "mcp");
    let codex = json(".codex/hooks.json");
    assert_eq!(
        codex["hooks"]["Stop"][0]["hooks"][0]["command"],
        "ascribe agents hook codex --event stop"
    );
    let copilot = json(".github/hooks/ascribe.json");
    assert_eq!(copilot["version"], 1);
    assert_eq!(
        copilot["hooks"]["postToolUse"][0]["bash"],
        "ascribe agents hook copilot --event edit"
    );
    assert_eq!(
        copilot["hooks"]["agentStop"][0]["powershell"],
        "ascribe agents hook copilot --event stop"
    );

    // Kept up to date without --with-hook, and covered by --check.
    let out = sync(root, ".", &[]);
    assert!(!stdout(&out).contains("wrote"), "{}", stdout(&out));
    assert!(
        stdout(&out).contains("unchanged .codex/hooks.json"),
        "{}",
        stdout(&out)
    );
    run(root, &["agents", "sync", "--check"], 0);
    fs::remove_file(root.join(".github/hooks/ascribe.json")).unwrap();
    write(&root.join(".github/hooks/ascribe.json"), "{}\n");
    let out = run(root, &["agents", "sync", "--check"], 1);
    assert!(
        stdout(&out).contains("stale      .github/hooks/ascribe.json"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn with_hook_keeps_what_the_user_changed() {
    let repo = repository("quill");
    let root = repo.path();
    sync(root, ".", &["--target", "claude", "--with-hook"]);
    // Laid out by another formatter, with a longer timeout on Ascribe's
    // stop hook, an `env` on its MCP server, and nothing else changed.
    let settings = root.join(".claude/settings.json");
    let mut value: serde_json::Value = serde_json::from_str(&read(&settings)).unwrap();
    value["hooks"]["Stop"][0]["hooks"][0]["timeout"] = 300.into();
    let mut four = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut serializer = serde_json::Serializer::with_formatter(&mut four, formatter);
    serde::Serialize::serialize(&value, &mut serializer).unwrap();
    let four = String::from_utf8(four).unwrap();
    write(&settings, &four);
    let mcp = root.join(".mcp.json");
    let mut servers: serde_json::Value = serde_json::from_str(&read(&mcp)).unwrap();
    servers["mcpServers"]["ascribe"]["env"] = serde_json::json!({ "A": "1" });
    write(&mcp, &serde_json::to_string(&servers).unwrap());
    let before = read(&mcp);
    run(root, &["agents", "sync", "--check"], 0);
    sync(root, ".", &[]);
    assert_eq!(read(&settings), four);
    assert_eq!(read(&mcp), before);
    // A server the user removed stays removed.
    write(&mcp, "{\"mcpServers\": {}}\n");
    sync(root, ".", &[]);
    assert_eq!(read(&mcp), "{\"mcpServers\": {}}\n");
    run(root, &["agents", "sync", "--check"], 0);
}

#[test]
fn with_hook_runs_the_pinned_ascribe() {
    let repo = repository("quill");
    let root = repo.path();
    fs::create_dir_all(root.join("node_modules/@ascribed/cli")).unwrap();
    sync(root, ".", &["--target", "claude", "--with-hook"]);
    let settings = read(&root.join(".claude/settings.json"));
    assert!(
        settings.contains(
            r#""command": "\"$CLAUDE_PROJECT_DIR\"/node_modules/.bin/ascribe agents hook claude-code --event edit""#
        ),
        "{settings}"
    );
    // `.mcp.json` has no way to name the repository's root on every
    // system, so it runs `ascribe` from the path.
    let mcp = read(&root.join(".mcp.json"));
    assert!(mcp.contains(r#""command": "ascribe""#), "{mcp}");
}

#[test]
fn a_settings_file_that_isnt_json_stops_everything() {
    let repo = repository("quill");
    let root = repo.path();
    write(&root.join(".claude/settings.json"), "{ not json\n");
    let out = run(
        root,
        &["agents", "sync", "--target", "claude", "--with-hook"],
        2,
    );
    assert!(
        stderr(&out).contains("settings.json: it isn't JSON"),
        "{}",
        stderr(&out)
    );
    assert!(!root.join("AGENTS.md").exists());
}

const SETUP_STEPS: &str = ".github/workflows/copilot-setup-steps.yml";

#[test]
fn cloud_prints_the_mcp_settings_unless_it_writes_the_agent() {
    let repo = repository("quill");
    let root = repo.path();
    let out = sync(root, ".", &["--cloud"]);
    let text = stdout(&out);
    assert!(text.contains(&format!("wrote     {SETUP_STEPS}")), "{text}");
    // --cloud implies --target copilot.
    assert!(
        text.contains("wrote     .github/instructions/ascribe.instructions.md"),
        "{text}"
    );
    let json = text
        .split_once("\n\n{")
        .map(|(_, json)| format!("{{{json}"))
        .unwrap_or_default();
    let settings: serde_json::Value = serde_json::from_str(&json).unwrap();
    let server = &settings["mcpServers"]["ascribe"];
    assert_eq!(server["type"], "local");
    assert_eq!(server["command"], "ascribe");
    assert_eq!(server["args"][0], "mcp");
    assert_eq!(server["tools"][0], "ascribe_check");
    assert!(!root.join(".github/agents").exists());

    // Kept up to date without --cloud, which alone prints the settings.
    let out = sync(root, ".", &[]);
    assert!(
        stdout(&out).contains(&format!("unchanged {SETUP_STEPS}")),
        "{}",
        stdout(&out)
    );
    assert!(!stdout(&out).contains("mcpServers"), "{}", stdout(&out));
    run(root, &["agents", "sync", "--check"], 0);

    // With --agent, the custom agent carries the server instead.
    let out = sync(root, ".", &["--cloud", "--agent"]);
    assert!(!stdout(&out).contains("mcpServers"), "{}", stdout(&out));
    let agent = read(&root.join(".github/agents/ascribe-docs.md"));
    assert!(
        agent.contains("mcp-servers:\n  ascribe:\n    type: local\n"),
        "{agent}"
    );
    let out = sync(root, ".", &["--cloud"]);
    assert!(
        stdout(&out).contains("unchanged .github/agents/ascribe-docs.md"),
        "{}",
        stdout(&out)
    );
    assert!(!stdout(&out).contains("mcpServers"), "{}", stdout(&out));

    // --check fails when either is stale.
    for file in [SETUP_STEPS, ".github/agents/ascribe-docs.md"] {
        let path = root.join(file);
        let before = read(&path);
        write(
            &path,
            &before
                .replace("ascribe --version", "ascribe -V")
                .replace("ascribe_check", "x"),
        );
        let out = run(root, &["agents", "sync", "--check"], 1);
        assert!(
            stdout(&out).contains(&format!("stale      {file}")),
            "{}",
            stdout(&out)
        );
        sync(root, ".", &[]);
        assert_eq!(read(&path), before);
    }
}

#[test]
fn agent_needs_cloud() {
    let repo = repository("quill");
    let out = run(repo.path(), &["agents", "sync", "--agent"], 2);
    assert!(stderr(&out).contains("--cloud"), "{}", stderr(&out));
    assert!(!repo.path().join("AGENTS.md").exists());
}

#[test]
fn cloud_adds_its_steps_to_a_workflow_that_has_others() {
    let repo = repository("quill");
    let root = repo.path();
    let team = "\
name: Copilot setup steps
on: workflow_dispatch
jobs:
  copilot-setup-steps:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - name: Install Python
        uses: actions/setup-python@v6

    timeout-minutes: 20
";
    write(&root.join(SETUP_STEPS), team);
    sync(root, ".", &["--cloud"]);
    let text = read(&root.join(SETUP_STEPS));
    assert!(
        text.starts_with(
            "name: Copilot setup steps\non: workflow_dispatch\njobs:\n  copilot-setup-steps:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v7\n      - name: Install Python\n        uses: actions/setup-python@v6\n      # ascribe:copilot start ("
        ),
        "{text}"
    );
    assert!(
        text.ends_with("        run: ascribe --version\n      # ascribe:copilot end\n\n    timeout-minutes: 20\n"),
        "{text}"
    );
    // The team's steps outside the block are theirs to change.
    let changed = text.replace("setup-python@v6", "setup-python@v7");
    write(&root.join(SETUP_STEPS), &changed);
    run(root, &["agents", "sync", "--check"], 0);
    sync(root, ".", &[]);
    assert_eq!(read(&root.join(SETUP_STEPS)), changed);

    // A workflow it can't add to stops everything.
    write(&root.join(SETUP_STEPS), "jobs:\n  build:\n    steps: []\n");
    fs::remove_file(root.join("AGENTS.md")).unwrap();
    let out = run(root, &["agents", "sync", "--cloud"], 2);
    assert!(
        stderr(&out).contains("copilot-setup-steps.yml: it has no `copilot-setup-steps` job"),
        "{}",
        stderr(&out)
    );
    assert!(!root.join("AGENTS.md").exists());
}

#[test]
fn cloud_installs_the_pinned_ascribe() {
    let repo = repository("monorepo");
    let root = repo.path();
    write(
        &root.join("docs/package.json"),
        "{\"devDependencies\": {\"@ascribed/cli\": \"1.0.0\"}}\n",
    );
    write(&root.join("pnpm-lock.yaml"), "lockfileVersion: '9.0'\n");
    sync(root, "docs", &["--cloud"]);
    sync(root, "handbook", &["--cloud"]);
    let text = read(&root.join(SETUP_STEPS));
    // docs pins it, installed from the lockfile at the root; handbook
    // doesn't, so it installs this version, in the block every such
    // project shares.
    assert!(
        text.contains("      # ascribe:copilot:docs start ("),
        "{text}"
    );
    assert!(
        text.contains("        run: |\n          corepack enable && pnpm install --frozen-lockfile\n          echo \"$GITHUB_WORKSPACE/docs/node_modules/.bin\" >> \"$GITHUB_PATH\"\n      - name: Show"),
        "{text}"
    );
    assert!(text.contains("      # ascribe:copilot start ("), "{text}");
    assert!(
        text.contains(&format!(
            "run: npm install --global '@ascribed/cli@{}'\n",
            env!("CARGO_PKG_VERSION")
        )),
        "{text}"
    );
    for project in ["docs", "handbook"] {
        run(root, &["agents", "sync", "--config", project, "--check"], 0);
    }
}

#[test]
fn the_setup_steps_match_this_repositorys_versions() {
    let repo = repository("quill");
    sync(repo.path(), ".", &["--cloud"]);
    let text = read(&repo.path().join(SETUP_STEPS));
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let ci = read(&workspace.join(".github/workflows/ci.yml"))
        + &read(&workspace.join(".github/workflows/js.yml"));
    for action in ["actions/checkout", "actions/setup-node"] {
        let major = text
            .split_once(&format!("uses: {action}@v"))
            .and_then(|(_, rest)| rest.lines().next())
            .unwrap_or_else(|| panic!("no {action} in\n{text}"));
        let ours = ci
            .lines()
            .find(|line| line.contains(&format!("uses: {action}@")))
            .and_then(|line| line.split_once(" # v"))
            .map(|(_, version)| version.split('.').next().unwrap_or_default().to_owned())
            .unwrap_or_else(|| panic!("no {action} in this repository's workflows"));
        assert_eq!(
            major, ours,
            "the setup steps use {action}@v{major}, this repository v{ours}; change src/agents/copilot.rs"
        );
    }
    let package = read(&workspace.join("packages/cli/package.json"));
    let package: serde_json::Value = serde_json::from_str(&package).unwrap();
    let engines = package["engines"]["node"].as_str().unwrap();
    let node = text
        .split_once("node-version: ")
        .and_then(|(_, rest)| rest.lines().next())
        .unwrap();
    assert_eq!(
        format!(">={node}"),
        engines,
        "the setup steps install Node.js {node}; @ascribed/cli needs {engines}"
    );
}

#[test]
fn a_pinned_project_runs_its_own_mcp_server() {
    let repo = repository("monorepo");
    let root = repo.path();
    write(
        &root.join("docs/package.json"),
        "{\"devDependencies\": {\"@ascribed/cli\": \"1.0.0\"}}\n",
    );
    write(&root.join("docs/package-lock.json"), "{}\n");
    let out = sync(root, "docs", &["--cloud"]);
    // Raw: `stdout` turns the JSON's escapes into slashes.
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let json = text
        .split_once("\n\n{")
        .map(|(_, json)| format!("{{{json}"))
        .unwrap_or_default();
    let settings: serde_json::Value = serde_json::from_str(&json).unwrap();
    let server = &settings["mcpServers"]["ascribe"];
    assert_eq!(server["command"], "bash");
    assert_eq!(
        server["args"][1],
        "exec \"$(git rev-parse --show-toplevel)/docs/node_modules/.bin/ascribe\" mcp"
    );
    let steps = read(&root.join(SETUP_STEPS));
    assert!(steps.contains("          npm ci\n"), "{steps}");
    assert!(
        steps.contains("        working-directory: docs\n"),
        "{steps}"
    );
    sync(root, "docs", &["--cloud", "--agent"]);
    let agent = read(&root.join(".github/agents/ascribe-docs.md"));
    assert!(
        agent.contains("    command: \"bash\"\n    args: [\"-c\", \"exec \\\"$(git rev-parse --show-toplevel)/docs/node_modules/.bin/ascribe\\\" mcp\"]\n"),
        "{agent}"
    );
}

#[test]
fn projects_that_install_differently_get_a_server_each() {
    let repo = repository("monorepo");
    let root = repo.path();
    write(
        &root.join("docs/package.json"),
        "{\"devDependencies\": {\"@ascribed/cli\": \"1.0.0\"}}\n",
    );
    write(&root.join("docs/package-lock.json"), "{}\n");
    // docs installs its pinned version, and handbook (with the project
    // nested in it) this one, globally: one JSON document, a server each,
    // named as the steps' blocks are.
    let out = run(root, &["agents", "sync", "--cloud"], 0);
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert_eq!(text.matches("mcpServers").count(), 1, "{text}");
    let json = text
        .split_once("\n\n{")
        .map(|(_, json)| format!("{{{json}"))
        .unwrap_or_default();
    let settings: serde_json::Value = serde_json::from_str(&json).unwrap();
    let servers = settings["mcpServers"].as_object().unwrap();
    let names: Vec<&str> = servers.keys().map(String::as_str).collect();
    assert_eq!(names, ["ascribe", "ascribe-docs"], "{json}");
    assert_eq!(
        servers["ascribe-docs"]["args"][1],
        "exec \"$(git rev-parse --show-toplevel)/docs/node_modules/.bin/ascribe\" mcp"
    );
    assert_eq!(servers["ascribe"]["command"], "ascribe");
    let steps = read(&root.join(SETUP_STEPS));
    assert!(steps.contains("# ascribe:copilot:docs start ("), "{steps}");
    assert!(steps.contains("# ascribe:copilot start ("), "{steps}");

    // One project synced alone still prints every server.
    let out = sync(root, "handbook", &["--cloud"]);
    let alone = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(alone.ends_with(&json), "{alone}");

    // The custom agent carries every server too, under the same names.
    run(root, &["agents", "sync", "--cloud", "--agent"], 0);
    let agent = read(&root.join(".github/agents/ascribe-docs.md"));
    let front = agent.split("---\n").nth(1).unwrap_or_default();
    let value: serde_yaml_ng::Value = serde_yaml_ng::from_str(front).unwrap();
    let ours = value["mcp-servers"].as_mapping().unwrap();
    let names: Vec<&str> = ours.keys().filter_map(|k| k.as_str()).collect();
    assert_eq!(names, ["ascribe-docs", "ascribe"], "{agent}");
    assert!(
        agent.contains("`ascribe-docs` for a project that installs it from `docs/package.json`"),
        "{agent}"
    );
    sync(root, "docs", &[]);
    assert_eq!(read(&root.join(".github/agents/ascribe-docs.md")), agent);
    run(root, &["agents", "sync", "--check"], 0);
}

#[test]
fn each_yarn_is_installed_its_own_way() {
    const BERRY: &str = "__metadata:\n  version: 8\n";
    let case = |lock: &str, yarnrc: Option<&str>, spec: &str| {
        let repo = repository("quill");
        let root = repo.path();
        write(
            &root.join("package.json"),
            &format!("{{\"devDependencies\": {{\"@ascribed/cli\": \"{spec}\"}}}}\n"),
        );
        write(&root.join("yarn.lock"), lock);
        if let Some(yarnrc) = yarnrc {
            write(&root.join(".yarnrc.yml"), yarnrc);
        }
        let out = ascribe(root, &["agents", "sync", "--cloud"]);
        let steps = fs::read_to_string(root.join(SETUP_STEPS)).unwrap_or_default();
        (out, steps, repo)
    };

    let (_, steps, _) = case("# yarn lockfile v1\n", None, "1.0.0");
    assert!(
        steps.contains("          corepack enable && yarn install --frozen-lockfile\n          echo \"$GITHUB_WORKSPACE/node_modules/.bin\""),
        "{steps}"
    );
    let (_, steps, _) = case(BERRY, Some("nodeLinker: node-modules\n"), "1.0.0");
    assert!(
        steps.contains("          corepack enable && yarn install --immutable\n"),
        "{steps}"
    );
    // Plug'n'Play writes no node_modules/.bin: the pinned version is
    // installed globally instead, and the MCP server runs from the path.
    let (out, steps, _) = case(BERRY, None, "^1.2.0");
    assert!(
        steps.contains("run: npm install --global '@ascribed/cli@^1.2.0'\n"),
        "{steps}"
    );
    assert!(!steps.contains("yarn install"), "{steps}");
    assert!(
        stdout(&out).contains("\"command\": \"ascribe\""),
        "{}",
        stdout(&out)
    );
    // A version npm can't install stops everything.
    let (out, _, repo) = case(BERRY, Some("nodeLinker: pnp\n"), "workspace:*");
    assert_eq!(out.status.code(), Some(2), "{}", stdout(&out));
    assert!(
        stderr(&out).contains("npm can't install `workspace:*`; pin a version in package.json, or set `nodeLinker: node-modules` in .yarnrc.yml"),
        "{}",
        stderr(&out)
    );
    assert!(!repo.path().join("AGENTS.md").exists());
}

#[test]
fn a_committed_link_on_the_way_stops_everything() {
    let repo = repository("quill");
    let outside = tempfile::tempdir().unwrap();
    links::dir(outside.path(), repo.path().join(".claude"));
    let out = run(repo.path(), &["agents", "sync", "--target", "claude"], 2);
    assert!(stderr(&out).contains("is a link"), "{}", stderr(&out));
    assert_eq!(
        fs::read_dir(outside.path()).unwrap().count(),
        0,
        "sync wrote through the link"
    );
    assert!(
        !repo.path().join("AGENTS.md").exists(),
        "it wrote AGENTS.md"
    );
}

/// `ascribe agents index --format json` in `dir`, with `args`.
fn index_json(dir: &Path, args: &[&str]) -> serde_json::Value {
    let mut all = vec!["agents", "index", "--format", "json"];
    all.extend_from_slice(args);
    let out = run(dir, &all, 0);
    serde_json::from_slice(&out.stdout).expect("index --format json is JSON")
}

#[test]
fn the_index_of_ascribes_docs_lists_every_offered_page_under_its_limit() {
    let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
    let index = index_json(&docs, &[]);
    let limit = index["limit"].as_u64().unwrap();
    let bytes = index["bytes"].as_u64().unwrap();
    let text = index["text"].as_str().unwrap();
    assert!(bytes <= limit, "{bytes} bytes, over {limit}:\n{text}");
    assert_eq!(bytes, text.len() as u64);
    let listed = index["listed"].as_array().unwrap();
    let left_out = index["left_out"].as_array().unwrap();
    assert_eq!(
        (listed.len() + left_out.len()) as u64,
        index["offered"].as_u64().unwrap(),
        "every offered page is listed or left out"
    );
    if !left_out.is_empty() {
        assert!(
            text.contains(&format!("leaves out {} page", left_out.len())),
            "{text}"
        );
    }
    for page in listed {
        let page = page.as_str().unwrap();
        let source = read(&docs.join("content").join(page));
        let frontmatter = source.split("\n---").next().unwrap_or_default();
        assert!(
            !frontmatter.contains("\nlisting: unlisted"),
            "{page} isn't offered, but the index lists it"
        );
    }
}

#[test]
fn a_second_index_into_a_file_changes_nothing() {
    let repo = repository("quill");
    let dir = repo.path();
    write(
        &dir.join("pkg/AGENTS.md"),
        "# Quill's SDK\n\nUse the SDK.\n",
    );
    let args = [
        "agents",
        "index",
        "--build",
        "site",
        "--into",
        "pkg/AGENTS.md",
        "--pages",
        "pkg/docs",
    ];
    let first = run(dir, &args, 0);
    assert!(
        stdout(&first).starts_with("wrote     pkg/AGENTS.md\nwrote     pkg/docs"),
        "{}",
        stdout(&first)
    );
    let agents_md = read(&dir.join("pkg/AGENTS.md"));
    assert!(
        agents_md.starts_with("# Quill's SDK\n\nUse the SDK.\n\n<!-- ascribe:agents:index start"),
        "{agents_md}"
    );
    // The addresses are from the file's folder.
    assert!(agents_md.contains("](docs/"), "{agents_md}");
    let pages = files(&dir.join("pkg/docs"));
    assert!(!pages.is_empty());
    for page in &pages {
        assert!(
            agents_md.contains(&format!("](docs/{page})")),
            "{page} isn't in:\n{agents_md}"
        );
    }

    let second = run(dir, &args, 0);
    assert!(
        stdout(&second).starts_with("unchanged pkg/AGENTS.md\nunchanged pkg/docs"),
        "{}",
        stdout(&second)
    );
    assert_eq!(read(&dir.join("pkg/AGENTS.md")), agents_md);
}

#[test]
fn an_index_in_its_own_pages_folder_isnt_a_stray_page() {
    let repo = repository("quill");
    let dir = repo.path();
    let args = [
        "agents",
        "index",
        "--build",
        "site",
        "--into",
        "out/AGENTS.md",
        "--pages",
        "out",
    ];
    for _ in 0..2 {
        let out = run(dir, &args, 0);
        assert!(!stderr(&out).contains("AGENTS.md"), "{}", stderr(&out));
    }
    // A Markdown file of the folder's that isn't a page is still named.
    write(&dir.join("out/old.md"), "# Old\n");
    let out = run(dir, &args, 0);
    assert!(stderr(&out).contains(": old.md"), "{}", stderr(&out));
    assert!(!stderr(&out).contains("AGENTS.md"), "{}", stderr(&out));
}

#[test]
fn pages_in_the_current_folder_are_shown_as_a_dot() {
    let repo = repository("quill");
    let out_dir = repo.path().join("out");
    fs::create_dir_all(&out_dir).unwrap();
    let out = run(
        &out_dir,
        &[
            "--config",
            "..",
            "agents",
            "index",
            "--build",
            "site",
            "--into",
            "AGENTS.md",
            "--pages",
            ".",
        ],
        0,
    );
    assert!(stdout(&out).contains("\nwrote     .  "), "{}", stdout(&out));
}

#[test]
fn an_index_with_its_pages_in_the_content_root_is_refused() {
    let repo = repository("quill");
    let dir = repo.path();
    let out = run(
        dir,
        &[
            "agents",
            "index",
            "--build",
            "site",
            "--pages",
            "docs/copies",
        ],
        2,
    );
    assert!(
        stderr(&out).contains("under the content root"),
        "{}",
        stderr(&out)
    );
    assert!(!dir.join("docs/copies").exists());
}

/// Whether `file`, from the repository's root, is one `pattern` from the
/// page's table names: a folder ending `/`, a name with one `*`, or a path.
fn names(pattern: &str, file: &str) -> bool {
    if pattern.ends_with('/') {
        file.starts_with(pattern)
    } else if let Some((before, after)) = pattern.split_once('*') {
        file.len() >= before.len() + after.len()
            && file.starts_with(before)
            && file.ends_with(after)
            && !file[before.len()..file.len() - after.len()].contains('/')
    } else {
        file == pattern
    }
}

/// Files Ascribe writes, `docs/content/reference/project-files.md`, lists
/// every file `sync` writes with every option, in its rows that name
/// `ascribe agents sync`, and nothing it doesn't.
#[test]
fn the_project_files_page_lists_what_sync_writes() {
    let page =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/content/reference/project-files.md");
    let page = read(&page);
    let mut listed = Vec::new();
    for row in page.lines().filter(|l| l.starts_with("| ")) {
        let cells: Vec<&str> = row.split(" | ").collect();
        if cells.len() < 2 || !cells[1].contains("`ascribe agents sync") {
            continue;
        }
        let paths = cells[0].split('`').skip(1).step_by(2);
        listed.extend(paths.map(str::to_owned));
    }
    assert!(
        !listed.is_empty(),
        "no row of the page names `ascribe agents sync`"
    );

    let repo = repository("quill");
    let before = files(repo.path());
    let mut args = EVERY_TARGET.to_vec();
    args.extend(["--target", "codex", "--with-hook"]);
    sync(repo.path(), ".", &args);
    let written: Vec<String> = files(repo.path())
        .into_iter()
        .filter(|f| !before.contains(f))
        .collect();

    let unlisted: Vec<&String> = written
        .iter()
        .filter(|file| !listed.iter().any(|pattern| names(pattern, file)))
        .collect();
    assert!(
        unlisted.is_empty(),
        "sync writes {unlisted:?}, which the project files page doesn't list"
    );
    let unwritten: Vec<&String> = listed
        .iter()
        .filter(|pattern| !written.iter().any(|file| names(pattern, file)))
        .collect();
    assert!(
        unwritten.is_empty(),
        "the project files page lists {unwritten:?}, which sync doesn't write"
    );
}
