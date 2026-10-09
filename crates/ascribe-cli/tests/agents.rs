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

const BLESS: &str = "ASCRIBE_BLESS=1 cargo test -p ascribe-cli --test agents";

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
            "run: npm install --global @ascribed/cli@{}\n",
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
