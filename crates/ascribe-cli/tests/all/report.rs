//! `ascribe report`, by running the binary: its sections, its formats, and
//! its exit codes. The link checker and the site checker are scripts that
//! write what lychee and afdocs write, so nothing reaches the network.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const MODEL: &str = r#"spec = "0.1"

[consumer]
site = "https://docs.example.com"

[types.page]
default = true

[types.page.frontmatter]
title = "string"

[types.guide]
files = ["guides/**"]

[types.guide.frontmatter]
title = "string"
team = { type = "string?", role = "owner" }

[dimensions.deployment]
values = ["cloud", "server"]
versionless = ["cloud", "server"]

[builds.cloud]
availability = { filter = "cloud" }

[builds.server]
availability = { filter = "server" }

[editor]
build = "cloud"
"#;

const INSTALL: &str = "---\ntitle: Install\nteam: platform\n---\n\nSee the [old docs](https://gone.example/) first.\n";

const CLOUD: &str =
    "---\ntitle: Cloud\nteam: platform\navailable: cloud\n---\n\nRead [install](install.md).\n";

fn project(model: &str) -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    write(dir.path(), "ascribe.toml", model);
    write(dir.path(), "docs/guides/install.md", INSTALL);
    write(dir.path(), "docs/guides/cloud.md", CLOUD);
    write(
        dir.path(),
        "docs/index.md",
        "---\ntitle: Home\n---\n\nRead [install](guides/install.md) and [cloud](guides/cloud.md).\n",
    );
    dir
}

fn write(root: &Path, rel: &str, text: &str) {
    let path: PathBuf = rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
    fs::create_dir_all(path.parent().expect("a parent")).expect("create directories");
    fs::write(path, text).expect("write a file");
}

/// Runs the binary in `dir`, with only `path` on the `PATH`, so neither
/// lychee nor afdocs is found unless a test puts it there.
fn ascribe_with_path(dir: &Path, path: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(dir)
        .args(args)
        .env("NO_COLOR", "1")
        .env("PATH", path)
        .output()
        .expect("run ascribe")
}

fn ascribe(dir: &Path, args: &[&str]) -> Output {
    ascribe_with_path(dir, &dir.join("no-tools"), args)
}

fn code_of(output: &Output) -> i32 {
    output.status.code().expect("exited with a code")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).expect("the report's JSON")
}

#[test]
fn with_no_section_it_reports_what_needs_nothing_outside_the_project() {
    let dir = project(MODEL);
    let out = ascribe(dir.path(), &["report"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.starts_with("Problems\n"), "{text}");
    assert!(text.contains("\nInventory\n  3 pages.\n"), "{text}");
    assert!(text.contains("By type: guide 2, page 1."), "{text}");
    assert!(text.contains("By owner: platform 2, no owner 1."), "{text}");
    assert!(
        text.contains("server leaves out 1 page and 0 pieces of content that another build keeps:\n    guides/cloud.md: "),
        "{text}"
    );
    assert!(!text.contains("External links"), "{text}");

    let out = ascribe(dir.path(), &["report", "--format", "json"]);
    let report = json(&out);
    assert_eq!(report["schema_version"], 1);
    assert_eq!(
        report["sections"],
        serde_json::json!(["problems", "inventory", "builds"])
    );
    assert_eq!(report["inventory"]["pages"], 3);
    assert_eq!(report["builds"][1]["build"], "server");
    assert_eq!(
        report["builds"][1]["pages"]["items"][0]["file"],
        "guides/cloud.md"
    );
    assert_eq!(
        report["builds"][1]["pages"]["items"][0]["kept_by"],
        serde_json::json!(["cloud"])
    );
}

#[test]
fn a_section_that_can_t_run_says_why_and_fails_only_with_exit_code() {
    let dir = project(MODEL);
    let out = ascribe(dir.path(), &["report", "links", "agents"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains("External links\n  Not run: `lychee` couldn't be run: it isn't installed, or isn't on the PATH.\n  Install lychee"),
        "{text}"
    );
    assert!(
        text.contains("Not run: it checks a built site, and no `--site` was given.\n  Give the address of a built site with `--site`, such as the published one, `--site https://docs.example.com`."),
        "{text}"
    );

    let out = ascribe(dir.path(), &["report", "links", "--exit-code"]);
    assert_eq!(code_of(&out), 2, "a section that didn't run isn't a pass");

    let out = ascribe(
        dir.path(),
        &["report", "links", "agents", "--format", "json"],
    );
    let report = json(&out);
    assert_eq!(report["not_run"], serde_json::json!(["links", "agents"]));
    assert!(
        report["links"]["not_run"]["how"]
            .as_str()
            .unwrap()
            .starts_with("Install lychee")
    );
}

#[test]
fn prompt_takes_one_section() {
    let dir = project(MODEL);
    for args in [
        &["report", "--format", "prompt"][..],
        &["report", "problems", "links", "--format", "prompt"],
        &["report", "inventory", "--format", "prompt"],
    ] {
        let out = ascribe(dir.path(), args);
        assert_eq!(code_of(&out), 2, "{args:?}");
        assert!(stderr(&out).contains("--format prompt writes one section's prompt"));
    }
}

#[test]
fn an_unknown_build_is_a_failure() {
    let dir = project(MODEL);
    let out = ascribe(dir.path(), &["report", "--build", "nope"]);
    assert_eq!(code_of(&out), 2);
    assert!(stderr(&out).contains("nope"), "{}", stderr(&out));
}

#[test]
fn lists_are_capped_with_the_command_that_lists_them_all() {
    let dir = project(MODEL);
    let out = ascribe(
        dir.path(),
        &["report", "builds", "--limit", "0", "--format", "json"],
    );
    let report = json(&out);
    let pages = &report["builds"][1]["pages"];
    assert_eq!(
        (&pages["shown"], &pages["total"], &pages["truncated"]),
        (&0.into(), &1.into(), &true.into())
    );
    assert_eq!(
        report["next_command"],
        "ascribe report builds --limit 1 --format json"
    );

    let out = ascribe(dir.path(), &["report", "builds", "--limit", "0"]);
    assert!(
        stdout(&out).contains("    and 1 more: ascribe report builds --limit 1\n"),
        "{}",
        stdout(&out)
    );
}

#[cfg(unix)]
fn script(dir: &Path, name: &str, body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join(name);
    fs::create_dir_all(dir).unwrap();
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
}

/// What lychee writes when the first address it's given is a 404.
#[cfg(unix)]
const LYCHEE_404: &str = r#"cat > /dev/null
printf '%s' '{"total":1,"successful":0,"errors":1,"error_map":{"stdin":[{"url":"https://gone.example/","status":{"text":"Rejected status code: 404 Not Found","code":404},"span":{"line":1,"column":1}}]},"success_map":{},"timeout_map":{},"redirect_map":{}}'
exit 2"#;

#[cfg(unix)]
#[test]
fn links_are_reported_at_their_place_and_fail_with_exit_code() {
    let dir = project(&format!(
        "{MODEL}\n[checks.links]\ncommand = \"bin/lychee\"\n"
    ));
    script(&dir.path().join("bin"), "lychee", LYCHEE_404);
    let out = ascribe(dir.path(), &["report", "links"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains("docs/guides/install.md:6: [ASC168] `https://gone.example/` answered 404 Not Found, so the link is broken for readers"),
        "{text}"
    );

    let out = ascribe(dir.path(), &["report", "links", "--exit-code"]);
    assert_eq!(code_of(&out), 1);
    let out = ascribe(dir.path(), &["report", "links", "--exit-code=warning"]);
    assert_eq!(code_of(&out), 0, "the finding is advice");

    let out = ascribe(dir.path(), &["report", "links", "--format", "summary"]);
    let summary = stdout(&out);
    assert!(
        summary.starts_with(
            "## Ascribe report\n\n0 errors, 0 warnings, 1 advice.\n\n### External links\n"
        ),
        "{summary}"
    );

    let out = ascribe(dir.path(), &["report", "links", "--format", "prompt"]);
    let prompt = stdout(&out);
    assert!(
        prompt.starts_with("Fix the external link `ascribe report links` finds broken or moved."),
        "{prompt}"
    );
    assert!(
        prompt.contains("The sentence around it: See the [old docs](https://gone.example/) first."),
        "{prompt}"
    );
    assert!(prompt.contains("run `ascribe report links`"), "{prompt}");

    let out = ascribe(dir.path(), &["report", "links", "--format", "json"]);
    let report = json(&out);
    let entry = &report["links"]["diagnostics"]["items"][0];
    assert_eq!(entry["slug"], "link-external-broken");
    assert_eq!(entry["next"], "review");
    assert_eq!(entry["file"], "docs/guides/install.md");
    assert_eq!(report["findings"]["by_next"][0]["next"], "review");
}

/// What afdocs writes when content negotiation fails and the rest passes.
#[cfg(unix)]
const AFDOCS: &str = r#"printf '%s' '{"url":"https://docs.example.com","results":[{"id":"llms-txt-exists","category":"content-discoverability","status":"pass","message":"found"},{"id":"content-negotiation","category":"markdown-availability","status":"fail","message":"Markdown is never served for Accept: text/markdown"}],"summary":{"total":2,"pass":1,"fail":1}}'
exit 1"#;

#[cfg(unix)]
#[test]
fn the_site_s_checks_say_who_changes_what() {
    let dir = project(MODEL);
    let bin = dir.path().join("tools");
    script(&bin, "afdocs", AFDOCS);
    let out = ascribe_with_path(
        dir.path(),
        &bin,
        &[
            "report",
            "agents",
            "--site",
            "https://docs.example.com",
            "--format",
            "json",
        ],
    );
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let report = json(&out);
    let agents = &report["agents"];
    assert_eq!(agents["results"][1]["owner"], "hosting");
    let entry = &agents["diagnostics"]["items"][0];
    assert_eq!(entry["slug"], "delivery-hosting");
    assert_eq!(entry["next"], "outside");
    assert!(agents["diagnostics"]["items"].as_array().unwrap().len() == 1);
}
