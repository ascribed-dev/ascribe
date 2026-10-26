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

[types.page.frontmatter]
title = "string"

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

const INSTALL: &str = "---\ntype: guide\ntitle: Install\nteam: platform\n---\n\nSee the [old docs](https://gone.example/) first.\n";

const CLOUD: &str = "---\ntype: guide\ntitle: Cloud\nteam: platform\navailable: cloud\n---\n\nRead [install](install.md).\n";

fn project(model: &str) -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    write(dir.path(), "ascribe.toml", model);
    write(dir.path(), "docs/guides/install.md", INSTALL);
    write(dir.path(), "docs/guides/cloud.md", CLOUD);
    write(
        dir.path(),
        "docs/index.md",
        "---\ntype: page\ntitle: Home\n---\n\nRead [install](guides/install.md) and [cloud](guides/cloud.md).\n",
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
        .env("ASCRIBE_LINKS_RETRY_WAIT", "0")
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
        text.contains("docs/guides/install.md:7: [ASC168] `https://gone.example/` answered 404 Not Found, so the link is broken for readers"),
        "{text}"
    );

    let out = ascribe(dir.path(), &["report", "links", "--exit-code"]);
    assert_eq!(code_of(&out), 1);
    let out = ascribe(dir.path(), &["report", "links", "--exit-code=warning"]);
    assert_eq!(code_of(&out), 0, "the problem is advice");

    let out = ascribe(dir.path(), &["report", "links", "--format", "markdown"]);
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
    assert_eq!(report["summary"]["by_next"][0]["next"], "review");
}

/// What lychee writes when every address answers, counting its runs in
/// `runs`.
#[cfg(unix)]
const LYCHEE_FINE: &str = r#"cat > /dev/null
echo run >> runs
printf '%s' '{"total":1,"successful":1,"errors":0,"error_map":{},"success_map":{},"timeout_map":{},"redirect_map":{}}'"#;

/// `ascribe report links` on `today`, with the arguments after.
#[cfg(unix)]
fn links_on(dir: &Path, today: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(dir)
        .args(["report", "links"])
        .args(args)
        .env("NO_COLOR", "1")
        .env("PATH", dir.join("no-tools"))
        .env("ASCRIBE_TODAY", today)
        .env("ASCRIBE_LINKS_RETRY_WAIT", "0")
        .output()
        .expect("run ascribe")
}

#[cfg(unix)]
#[test]
fn an_address_that_answered_isn_t_asked_about_again() {
    let dir = project(&format!(
        "{MODEL}\n[checks.links]\ncommand = \"bin/lychee\"\n"
    ));
    script(&dir.path().join("bin"), "lychee", LYCHEE_FINE);
    let runs = || fs::read_to_string(dir.path().join("runs")).map_or(0, |r| r.lines().count());

    let first = stdout(&links_on(dir.path(), "2026-10-01", &[]));
    assert!(
        first.contains(
            "Checked 1 address, asking about each. Nothing is remembered from an earlier run, so every failure is reported.\n"
        ),
        "{first}"
    );
    assert!(dir.path().join(".ascribe/cache/links").is_dir());

    let second = stdout(&links_on(dir.path(), "2026-10-03", &[]));
    assert!(
        second.contains(
            "Checked 1 address: asked about none, and reused 1 answer, the oldest from 1 October.\n"
        ),
        "{second}"
    );
    assert_eq!(runs(), 1, "nothing was left to ask about");

    let report = json(&links_on(dir.path(), "2026-10-03", &["--format", "json"]));
    let links = &report["links"];
    assert_eq!(
        (&links["checked"], &links["asked"], &links["reused"]),
        (&1.into(), &0.into(), &1.into())
    );
    assert_eq!(links["oldest_answer"], "2026-10-01");

    let rechecked = json(&links_on(
        dir.path(),
        "2026-10-03",
        &["--recheck", "--format", "json"],
    ));
    assert_eq!(rechecked["links"]["asked"], 1);
    assert_eq!(rechecked["links"]["oldest_answer"], serde_json::Value::Null);
    assert_eq!(runs(), 2);

    // Past every limit, it's asked about again.
    let later = json(&links_on(dir.path(), "2026-11-01", &["--format", "json"]));
    assert_eq!(later["links"]["asked"], 1);
}

/// What lychee writes: `answer.json` in the project's folder, one line,
/// which a test changes between runs. Only the shell's own commands run.
#[cfg(unix)]
const LYCHEE_ANSWER: &str =
    "while read -r _; do :; done\nread -r answer < answer.json\nprintf '%s' \"$answer\"";

/// lychee's JSON when every address answers.
#[cfg(unix)]
const ANSWERED: &str = r#"{"error_map":{},"success_map":{},"timeout_map":{}}"#;

/// lychee's JSON when the first address answers 503.
#[cfg(unix)]
const UNAVAILABLE: &str = r#"{"error_map":{"stdin":[{"url":"https://gone.example/","status":{"text":"Rejected status code: 503 Service Unavailable","code":503},"span":{"line":1,"column":1}}]},"success_map":{},"timeout_map":{}}"#;

#[cfg(unix)]
#[test]
fn a_link_that_answered_before_is_held_until_it_keeps_failing() {
    let dir = project(&format!(
        "{MODEL}\n[checks.links]\ncommand = \"bin/lychee\"\nreuse-days = 0\n"
    ));
    script(&dir.path().join("bin"), "lychee", LYCHEE_ANSWER);
    let answer = |json: &str| fs::write(dir.path().join("answer.json"), json).unwrap();

    answer(ANSWERED);
    let first = stdout(&links_on(dir.path(), "2026-10-08", &[]));
    assert!(
        first.contains("Checked 1 address, asking about each. Nothing is remembered from an earlier run, so every failure is reported.\n"),
        "{first}"
    );
    assert!(dir.path().join(".ascribe/history/links.json").is_file());

    // It answered on the 8th, so on the 10th it isn't reported yet.
    answer(UNAVAILABLE);
    let out = links_on(dir.path(), "2026-10-10", &["--exit-code"]);
    assert_eq!(code_of(&out), 0, "an unconfirmed link isn't a problem");
    assert_eq!(
        stdout(&out),
        "External links\n  \
         Checked 1 address, asking about each: no answer from an earlier run was reused.\n  \
         None is broken or moved.\n  \
         1 link not confirmed. It answered before and fails now, and is reported if it's still failing on its day:\n    \
         docs/guides/install.md:7: https://gone.example/ answered 503 Service Unavailable. Last answered 8 October; reported from 13 October\n"
    );
    let prompt = stdout(&links_on(dir.path(), "2026-10-10", &["--format", "prompt"]));
    assert!(!prompt.contains("gone.example"), "{prompt}");
    let report = json(&links_on(dir.path(), "2026-10-10", &["--format", "json"]));
    let links = &report["links"];
    assert_eq!(links["diagnostics"]["total"], 0);
    assert_eq!(links["previous_run"], "2026-10-10");
    assert_eq!((&links["new"], &links["resolved"]), (&0.into(), &0.into()));
    let held = &links["unconfirmed"]["items"][0];
    assert_eq!(held["file"], "docs/guides/install.md");
    assert_eq!(held["range"]["start"]["line"], 7);
    assert_eq!(held["url"], "https://gone.example/");
    assert_eq!(held["status"], "503 Service Unavailable");
    assert_eq!(
        (
            &held["last_answered"],
            &held["failing_since"],
            &held["reported_from"]
        ),
        (
            &"2026-10-08".into(),
            &"2026-10-10".into(),
            &"2026-10-13".into()
        )
    );
    assert_eq!(report["summary"]["advice"], 0);

    // Still failing on the 13th: reported, and new since the last run.
    let text = stdout(&links_on(dir.path(), "2026-10-13", &[]));
    assert!(
        text.contains(
            "  1 link to look at, new since 10 October:\n    docs/guides/install.md:7: [ASC168] `https://gone.example/` answered 503 Service Unavailable when asked twice: the server failed, which may be temporary"
        ),
        "{text}"
    );

    // It answers again: one link fewer than the last run reported.
    answer(ANSWERED);
    let text = stdout(&links_on(dir.path(), "2026-10-14", &[]));
    assert!(
        text.ends_with("  None is broken or moved.\n  1 link the last run reported is fine now, or no longer in the pages.\n"),
        "{text}"
    );

    // `--no-history` reports it at once, as a first run would.
    answer(UNAVAILABLE);
    let report = json(&links_on(
        dir.path(),
        "2026-10-15",
        &["--no-history", "--format", "json"],
    ));
    let links = &report["links"];
    assert_eq!(links["diagnostics"]["total"], 1);
    assert_eq!(links["unconfirmed"]["total"], 0);
    assert_eq!(links["previous_run"], serde_json::Value::Null);
    assert_eq!(links["new"], serde_json::Value::Null);
}

/// What lychee writes when the first address answers 503 the first time
/// it's asked, and answers after.
#[cfg(unix)]
const LYCHEE_503_ONCE: &str = r#"while read -r _; do :; done
if [ -f asked ]; then
  printf '%s' '{"error_map":{},"success_map":{},"timeout_map":{}}'
else
  : > asked
  printf '%s' '{"error_map":{"stdin":[{"url":"https://gone.example/","status":{"text":"Rejected status code: 503 Service Unavailable","code":503},"span":{"line":1,"column":1}}]},"success_map":{},"timeout_map":{}}'
fi"#;

#[cfg(unix)]
#[test]
fn a_server_error_is_asked_about_again_before_it_s_reported() {
    let dir = project(&format!(
        "{MODEL}\n[checks.links]\ncommand = \"bin/lychee\"\n"
    ));
    script(&dir.path().join("bin"), "lychee", LYCHEE_503_ONCE);
    let out = links_on(dir.path(), "2026-10-08", &["--exit-code"]);
    assert_eq!(code_of(&out), 0, "{}", stdout(&out));
    assert!(stdout(&out).contains("None is broken or moved."));
    assert!(dir.path().join("asked").is_file(), "it was asked twice");
}

/// What afdocs writes when content negotiation fails, the caching warns,
/// and the rest passes.
#[cfg(unix)]
const AFDOCS: &str = r#"printf '%s' '{"url":"https://docs.example.com","results":[{"id":"llms-txt-exists","category":"content-discoverability","status":"pass","message":"found"},{"id":"content-negotiation","category":"markdown-availability","status":"fail","message":"Markdown is never served for Accept: text/markdown"},{"id":"cache-header-hygiene","category":"observability","status":"warn","message":"llms.txt has no Cache-Control"}],"summary":{"total":3,"pass":1,"fail":1,"warn":1}}'
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
    let warning = &agents["diagnostics"]["items"][1];
    assert_eq!(warning["slug"], "delivery-hosting");
    assert!(
        warning["message"]
            .as_str()
            .unwrap()
            .starts_with("the site passes the delivery spec's `cache-header-hygiene` check with a warning: llms.txt has no Cache-Control. Set `Cache-Control`"),
        "{warning}"
    );
    assert!(agents["diagnostics"]["items"].as_array().unwrap().len() == 2);
}

const PAGES_MODEL: &str = r#"spec = "0.1"

[consumer]
site = "https://docs.example.com"
base-path = "/docs/"
agents = true

[types.page.frontmatter]
title = "string"
description = { type = "string?", role = "description" }

[listing.folders]
"internal/" = "unlisted"
"#;

/// A project whose records have every field: a description, a canonical
/// address, a Markdown version, breadcrumbs, and listings from a page and
/// from a folder's rule.
fn pages_project() -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    write(dir.path(), "ascribe.toml", PAGES_MODEL);
    write(
        dir.path(),
        "docs/index.md",
        "---\ntype: page\ntitle: Home\ndescription: Where to start.\n---\n\nRead [install](guides/install.md), [roadmap](roadmap.md), and [notes](internal/notes.md).\n",
    );
    write(
        dir.path(),
        "docs/guides/install.md",
        "---\ntype: page\ntitle: Install\ndescription: Install the product.\n---\n\nDownload it.\n",
    );
    write(
        dir.path(),
        "docs/roadmap.md",
        "---\ntype: page\ntitle: Roadmap\nlisting: unquoted\n---\n\nWhat's next.\n",
    );
    write(
        dir.path(),
        "docs/internal/notes.md",
        "---\ntype: page\ntitle: Notes\n---\n\nFor the team.\n",
    );
    dir
}

#[test]
fn pages_lists_every_record_by_address() {
    let dir = pages_project();
    let out = ascribe(dir.path(), &["report", "pages", "--limit", "3"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.starts_with(
            "Pages\n  site publishes 4 pages on https://docs.example.com, 2 of them not listed:\n"
        ),
        "{text}"
    );
    let rows: Vec<&str> = text.lines().skip(2).collect();
    assert!(rows[0].trim_start().starts_with("ADDRESS"), "{text}");
    assert!(rows[1].trim_start().starts_with("/docs/ "), "{text}");
    assert!(rows[2].contains("/docs/guides/install/"), "{text}");
    assert!(
        rows[3].contains("unlisted") && rows[3].contains("folder internal/, line 13"),
        "{text}"
    );
    assert_eq!(
        rows[4].trim(),
        "and 1 more: ascribe report pages --limit 4",
        "{text}"
    );

    let out = ascribe(dir.path(), &["report", "pages", "--not-listed"]);
    let text = stdout(&out);
    assert!(
        text.contains("site publishes 4 pages on https://docs.example.com, and 2 aren't listed:"),
        "{text}"
    );
    assert!(text.contains("/docs/internal/notes/"), "{text}");
    assert!(
        text.lines()
            .any(|l| l.contains("/docs/roadmap/") && l.contains("unquoted  page")),
        "{text}"
    );
    assert!(!text.contains("/docs/guides/install/"), "{text}");
}

/// `pages` as JSON has each entry of the build's `_ascribe/pages.json`, by
/// the same names, with the route it's keyed by there; and the report's
/// schema names exactly those fields.
#[test]
fn pages_as_json_are_the_build_s_records() {
    let dir = pages_project();
    let built = ascribe(dir.path(), &["build", "--emit", "site"]);
    assert_eq!(code_of(&built), 0, "{}", stderr(&built));
    let file = dir
        .path()
        .join(".ascribe/build/site/site/_ascribe/pages.json");
    let list: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(file).expect("pages.json")).expect("its JSON");

    let out = ascribe(dir.path(), &["report", "pages", "--format", "json"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let report = json(&out);
    assert_eq!(report["sections"], serde_json::json!(["pages"]));
    let build = &report["pages"][0];
    assert_eq!(build["build"], "site");
    assert_eq!(build["site"], list["site"]);
    assert_eq!(
        (build["published"].clone(), build["not_listed"].clone()),
        (4.into(), 2.into())
    );
    let mut routes = Vec::new();
    let mut fields = std::collections::BTreeSet::new();
    let entries = list["pages"].as_object().expect("pages");
    for item in build["records"]["items"].as_array().expect("records") {
        let mut item = item.as_object().expect("a record").clone();
        let route = item.remove("route").expect("a route");
        let route = route.as_str().expect("a string").to_owned();
        assert_eq!(
            serde_json::Value::Object(item.clone()),
            entries[&route],
            "{route}"
        );
        fields.extend(item.keys().cloned());
        routes.push(route);
    }
    let keyed: Vec<&String> = entries.keys().collect();
    assert_eq!(
        routes.iter().collect::<Vec<_>>(),
        keyed,
        "sorted by address, every page"
    );

    let schema: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schemas/report.schema.json"),
        )
        .expect("the report's schema"),
    )
    .expect("its JSON");
    let properties = |name: &str| -> std::collections::BTreeSet<String> {
        schema["$defs"][name]["properties"]
            .as_object()
            .unwrap_or_else(|| panic!("{name} in the schema"))
            .keys()
            .cloned()
            .collect()
    };
    fields.insert("route".to_owned());
    assert_eq!(properties("PageRecordJson"), fields);
    let from: std::collections::BTreeSet<String> = entries
        .values()
        .flat_map(|e| {
            e["listing_from"]
                .as_object()
                .expect("listing_from")
                .keys()
                .cloned()
        })
        .chain(["type".to_owned()])
        .collect();
    assert_eq!(properties("ListingFromJson"), from);
}

/// A project whose page shows three examples of a folder beside the docs,
/// with two observations: one current under either, one stale under both,
/// and one neither lists.
fn examples_project() -> TempDir {
    let hash = ascribe_model::example_hash;
    let dir = tempfile::tempdir().expect("a temporary directory");
    write(
        dir.path(),
        "ascribe.toml",
        "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[sources.code]\npath = \"code\"\n",
    );
    write(
        dir.path(),
        "code/ascribe.examples.toml",
        "include = [\"examples/**\"]\n\n[environment]\nnode = { label = \"Node\", command = \"node --version\" }\n",
    );
    write(dir.path(), "code/examples/a.ts", "a();\n");
    write(dir.path(), "code/examples/b.ts", "b();\n");
    write(dir.path(), "code/examples/c.ts", "c();\n");
    for (name, node) in [("default", "v24.9.0"), ("node-22", "v22.4.0")] {
        write(
            dir.path(),
            &format!("code/examples.observations/{name}.toml"),
            &format!(
                "version = 1\nascribe = \"0.4.0\"\ncommand = \"node --test\"\n\n[environment]\nos = \"linux\"\narch = \"x64\"\nnode = \"{node}\"\n\n[files]\n\"examples/a.ts\" = \"{}\"\n\"examples/b.ts\" = \"{}\"\n",
                hash(b"a();\n"),
                hash(b"old\n"),
            ),
        );
    }
    write(
        dir.path(),
        "docs/index.md",
        "---\ntitle: Examples\n---\n\n@snippet: code:examples/a.ts\n\n@snippet: code:examples/b.ts\n\n@snippet: code:examples/c.ts\n\n@snippet: code:examples/a.ts\n",
    );
    dir
}

#[test]
fn the_inventory_counts_a_source_in_another_repository_from_its_copies() {
    let hash = ascribe_model::example_hash;
    let dir = tempfile::tempdir().expect("a temporary directory");
    write(
        dir.path(),
        "ascribe.toml",
        "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[sources.code]\ngit = \"https://github.com/acme/code.git\"\n",
    );
    write(
        dir.path(),
        "sources/code/ascribe.examples.toml",
        "include = [\"examples/**\"]\n",
    );
    write(dir.path(), "sources/code/examples/a.ts", "a();\n");
    write(dir.path(), "sources/code/examples/b.ts", "b();\n");
    // `default` has neither as it is now; `node-22` has `a`, and lists a
    // file that wasn't copied.
    let uncopied = format!("\"examples/uncopied.ts\" = \"{}\"\n", hash(b"u();\n"));
    for (name, a, extra) in [("default", "old\n", ""), ("node-22", "a();\n", &*uncopied)] {
        write(
            dir.path(),
            &format!("sources/code/examples.observations/{name}.toml"),
            &format!(
                "version = 1\nascribe = \"0.4.0\"\ncommand = \"node --test\"\n\n[environment]\nos = \"linux\"\narch = \"x64\"\n\n[files]\n\"examples/a.ts\" = \"{}\"\n\"examples/b.ts\" = \"{}\"\n{extra}",
                hash(a.as_bytes()),
                hash(b"old\n"),
            ),
        );
    }
    write(
        dir.path(),
        "docs/index.md",
        "---\ntitle: Examples\n---\n\n@snippet: code:examples/a.ts\n\n@snippet: code:examples/b.ts\n",
    );
    let report = json(&ascribe(dir.path(), &["report", "--format", "json"]));
    assert_eq!(
        report["inventory"]["examples"],
        serde_json::json!([
            { "source": "code", "snippets": 2, "current": 1, "stale": 1, "unobserved": 0 }
        ])
    );

    // `ascribe model` counts only the files that were copied.
    let model = json(&ascribe(dir.path(), &["model", "--format", "json"]));
    let observations: Vec<(&str, u64, u64)> = model["sources"][0]["examples"]["observations"]
        .as_array()
        .expect("observations")
        .iter()
        .map(|o| {
            (
                o["name"].as_str().expect("a name"),
                o["current"].as_u64().expect("a count"),
                o["listed"].as_u64().expect("a count"),
            )
        })
        .collect();
    assert_eq!(observations, [("default", 0, 2), ("node-22", 1, 2)]);
}

#[test]
fn the_inventory_counts_examples_as_the_pages_have_them() {
    let dir = examples_project();
    let out = ascribe(dir.path(), &["report", "--format", "json"]);
    let report = json(&out);
    assert_eq!(
        report["inventory"]["examples"],
        serde_json::json!([
            { "source": "code", "snippets": 4, "current": 2, "stale": 1, "unobserved": 1 }
        ])
    );
    let text = stdout(&ascribe(dir.path(), &["report"]));
    assert!(
        text.contains("Examples from code: 4 snippets, 2 current, 1 stale, 1 unobserved."),
        "{text}"
    );

    // The JSON output's pages say the same of each snippet.
    let out = ascribe(dir.path(), &["build", "--emit", "json"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let page: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(dir.path().join(".ascribe/build/site/json/index.json"))
            .expect("the page's JSON"),
    )
    .expect("JSON");
    let tested: Vec<&serde_json::Value> = page["blocks"]
        .as_array()
        .expect("blocks")
        .iter()
        .filter_map(|b| b["snippet"].get("tested"))
        .collect();
    let statuses: Vec<&str> = tested
        .iter()
        .map(|t| t["status"].as_str().expect("a status"))
        .collect();
    assert_eq!(statuses, ["current", "stale", "unobserved", "current"]);
    assert_eq!(
        tested[0]["observations"],
        serde_json::json!([
            { "name": "default", "environment": { "os": "linux", "arch": "x64", "node": "v24.9.0" } },
            { "name": "node-22", "environment": { "os": "linux", "arch": "x64", "node": "v22.4.0" } }
        ])
    );
    assert_eq!(tested[1]["observations"], serde_json::json!([]));
    let blocks = page["blocks"].as_array().expect("blocks");
    let info = blocks
        .iter()
        .find(|b| b["snippet"]["tested"]["status"] == "current")
        .map(|b| b["info"].as_str().expect("an info string"));
    assert_eq!(info, Some("ts tested=\"Node 22.4.0 and 24.9.0 on Linux\""));

    // A project without a config has no such key.
    let plain = project(MODEL);
    let report = json(&ascribe(plain.path(), &["report", "--format", "json"]));
    assert!(report["inventory"].get("examples").is_none(), "{report}");
}

/// `ascribe report problems` in `dir` on `today`.
fn problems_on(dir: &Path, today: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(dir)
        .args(["report", "problems"])
        .args(args)
        .env("NO_COLOR", "1")
        .env("PATH", dir.join("no-tools"))
        .env("ASCRIBE_TODAY", today)
        .output()
        .expect("run ascribe")
}

/// The steel thread of a finding's identity (decision #787): a line added
/// above a finding keeps it, and an edit to its text makes it a new one.
#[test]
fn problems_says_what_s_new_since_the_last_report() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    write(
        dir.path(),
        "ascribe.toml",
        "spec = \"0.1\"\n\n[builds.one]\n\n[builds.two]\n\n[editor]\nbuild = \"one\"\n",
    );
    let page = |text: &str| {
        write(
            dir.path(),
            "docs/index.md",
            &format!("---\ntitle: Home\n---\n\n{text}"),
        )
    };
    page("Read [the old page](old.md).\n");

    // A first run says nothing about what's new.
    let first = problems_on(dir.path(), "2026-10-09", &[]);
    assert_eq!(code_of(&first), 0, "{}", stderr(&first));
    let text = stdout(&first);
    assert!(
        text.starts_with("Problems\n  1 error, 0 warnings, 0 advice.\n  By next step"),
        "{text}"
    );
    assert!(!text.contains("since"), "{text}");
    assert!(dir.path().join(".ascribe/history/report.json").is_file());

    // A line added above the finding, and a new finding below it.
    page("A new first line.\n\nRead [the old page](old.md).\n\nAnd [the new page](new.md).\n");
    let text = stdout(&problems_on(dir.path(), "2026-10-10", &[]));
    assert!(
        text.contains(
            "  2 errors, 0 warnings, 0 advice.\n  2 problems, 1 of them new since 9 October:\n    docs/index.md:9: [ASC036] `new.md` doesn't exist\n"
        ),
        "{text}"
    );
    let report = json(&problems_on(
        dir.path(),
        "2026-10-10",
        &["--format", "json"],
    ));
    let problems = &report["problems"];
    assert_eq!(problems["previous_run"], "2026-10-10");
    assert_eq!(
        (&problems["new"], &problems["resolved"]),
        (&0.into(), &0.into())
    );

    // The existing finding's text is edited: one new, and one gone.
    page("A new first line.\n\nRead [the old page](older.md).\n\nAnd [the new page](new.md).\n");
    let report = json(&problems_on(
        dir.path(),
        "2026-10-11",
        &["--format", "json"],
    ));
    let problems = &report["problems"];
    assert_eq!(problems["previous_run"], "2026-10-10");
    assert_eq!(
        (&problems["new"], &problems["resolved"]),
        (&1.into(), &1.into())
    );
    assert_eq!(problems["new_items"]["total"], 1);
    assert_eq!(problems["new_items"]["items"][0]["file"], "docs/index.md");
    assert_eq!(
        problems["new_items"]["items"][0]["range"]["start"]["line"],
        7
    );
    let text = stdout(&problems_on(dir.path(), "2026-10-12", &[]));
    assert!(
        text.contains("  2 problems, none of them new since 11 October.\n"),
        "{text}"
    );

    // Both fixed: the report says they're gone.
    page("A new first line.\n");
    let text = stdout(&problems_on(dir.path(), "2026-10-13", &[]));
    assert_eq!(
        text,
        "Problems\n  0 errors, 0 warnings, 0 advice.\n  The 2 problems the report of 12 October listed are gone.\n"
    );

    // `--no-history` says nothing about what's new, and neither reads nor
    // writes the history.
    page("Read [the old page](old.md).\n");
    let history = dir.path().join(".ascribe/history/report.json");
    let before = fs::read_to_string(&history).expect("the history");
    let report = json(&problems_on(
        dir.path(),
        "2026-10-14",
        &["--no-history", "--format", "json"],
    ));
    let problems = &report["problems"];
    assert_eq!(problems["previous_run"], serde_json::Value::Null);
    assert_eq!(
        (&problems["new"], &problems["resolved"]),
        (&serde_json::Value::Null, &serde_json::Value::Null)
    );
    assert_eq!(problems["new_items"]["total"], 0);
    assert_eq!(fs::read_to_string(&history).expect("the history"), before);
    let text = stdout(&problems_on(dir.path(), "2026-10-15", &[]));
    assert!(
        text.contains("  1 problem, new since 13 October:\n    docs/index.md:5: [ASC036] `old.md` doesn't exist\n"),
        "{text}"
    );

    // A report on one build of the two says nothing about what's new...
    let on = |today: &str, builds: &[&str]| {
        let mut args = vec!["--format", "json"];
        for b in builds {
            args.extend(["--build", b]);
        }
        json(&problems_on(dir.path(), today, &args))["problems"]["previous_run"].clone()
    };
    assert_eq!(on("2026-10-16", &["one"]), serde_json::Value::Null);
    // ...and the same builds in another order compare.
    assert_eq!(on("2026-10-17", &["one", "two"]), serde_json::Value::Null);
    assert_eq!(on("2026-10-18", &["two", "one"]), "2026-10-17");
}

/// What afdocs writes: content negotiation fails, and the redirects fail
/// too once a file named `failing` is in the folder it runs in.
#[cfg(unix)]
const AFDOCS_REDIRECTS_FAIL_LATER: &str = r#"redirects=pass
[ -f failing ] && redirects=fail
printf '{"url":"%s","results":[{"id":"llms-txt-exists","category":"content-discoverability","status":"pass","message":"found"},{"id":"content-negotiation","category":"markdown-availability","status":"fail","message":"Markdown is never served for Accept: text/markdown"},{"id":"redirect-behavior","category":"url-stability","status":"%s","message":"A redirect goes to another host"}]}' "$2" "$redirects"
exit 1"#;

/// `ascribe report agents` on `site`, in `dir` on `today`, with the stand-in
/// afdocs in `dir/tools`.
#[cfg(unix)]
fn agents_on(dir: &Path, today: &str, site: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(dir)
        .args(["report", "agents", "--site", site])
        .args(args)
        .env("NO_COLOR", "1")
        .env("PATH", dir.join("tools"))
        .env("ASCRIBE_TODAY", today)
        .output()
        .expect("run ascribe")
}

/// `agents` says which checks are failing since the last report on the
/// same site (decision #788): a check's identity is the site and its id.
#[cfg(unix)]
#[test]
fn agents_says_what_s_failing_since_the_last_report() {
    let dir = project(MODEL);
    script(
        &dir.path().join("tools"),
        "afdocs",
        AFDOCS_REDIRECTS_FAIL_LATER,
    );
    let site = "https://docs.example.com";

    // A first run says nothing about what's new, and keeps what it found.
    let first = agents_on(dir.path(), "2026-10-09", site, &[]);
    assert_eq!(code_of(&first), 0, "{}", stderr(&first));
    let text = stdout(&first);
    assert!(
        text.contains("checks on https://docs.example.com: 2 pass, 1 fail.\n  To change:"),
        "{text}"
    );
    assert!(!text.contains("since"), "{text}");
    let history = dir.path().join(".ascribe/history/report.json");
    assert!(history.is_file());

    // The redirects fail the second time: the report names the check.
    fs::write(dir.path().join("failing"), "").unwrap();
    let report = json(&agents_on(
        dir.path(),
        "2026-10-10",
        site,
        &["--format", "json"],
    ));
    let agents = &report["agents"];
    assert_eq!(agents["previous_run"], "2026-10-09");
    assert_eq!(
        (&agents["new"], &agents["resolved"]),
        (&1.into(), &0.into())
    );
    assert_eq!(
        agents["new_checks"],
        serde_json::json!(["redirect-behavior"])
    );
    let text = stdout(&agents_on(dir.path(), "2026-10-11", site, &[]));
    assert!(
        text.contains("  3 checks: 1 passing, 2 failing, none of them new since 10 October.\n"),
        "{text}"
    );

    // `--no-history` says nothing about what's new, and neither reads nor
    // writes the history.
    fs::remove_file(dir.path().join("failing")).unwrap();
    let before = fs::read_to_string(&history).unwrap();
    let report = json(&agents_on(
        dir.path(),
        "2026-10-12",
        site,
        &["--no-history", "--format", "json"],
    ));
    let agents = &report["agents"];
    assert_eq!(agents["previous_run"], serde_json::Value::Null);
    assert_eq!(
        (&agents["new"], &agents["resolved"]),
        (&serde_json::Value::Null, &serde_json::Value::Null)
    );
    assert_eq!(agents["new_checks"], serde_json::json!([]));
    assert_eq!(fs::read_to_string(&history).unwrap(), before);

    // The redirects pass again: one that failed no longer does.
    let text = stdout(&agents_on(dir.path(), "2026-10-13", site, &[]));
    assert!(
        text.contains(
            "  3 checks: 2 passing, 1 failing, not new since 11 October; 1 that failed on 11 October no longer fails.\n"
        ),
        "{text}"
    );

    // Another site has nothing to go by; and the next run on it compares
    // with it.
    fs::write(dir.path().join("failing"), "").unwrap();
    let preview = "http://localhost:4321";
    let text = stdout(&agents_on(dir.path(), "2026-10-14", preview, &[]));
    assert!(!text.contains("since"), "{text}");
    let text = stdout(&agents_on(dir.path(), "2026-10-15", preview, &[]));
    assert!(
        text.contains("  3 checks: 1 passing, 2 failing, none of them new since 14 October.\n"),
        "{text}"
    );

    // A report of `problems` alone keeps what `agents` found.
    let out = problems_on(dir.path(), "2026-10-16", &[]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    fs::remove_file(dir.path().join("failing")).unwrap();
    let text = stdout(&agents_on(dir.path(), "2026-10-17", preview, &[]));
    assert!(
        text.contains("  3 checks: 2 passing, 1 failing, not new since 15 October; 1 that failed on 15 October no longer fails.\n"),
        "{text}"
    );
}

/// What afdocs writes: two checks, each failing while a file named for it
/// is in the folder it runs in, and passing otherwise.
#[cfg(unix)]
const AFDOCS_FAILS_BY_FILE: &str = r#"status() { if [ -f "$1" ]; then echo fail; else echo pass; fi; }
printf '{"url":"%s","results":[{"id":"llms-txt-exists","category":"content-discoverability","status":"%s","message":"checked"},{"id":"redirect-behavior","category":"url-stability","status":"%s","message":"checked"}]}' "$2" "$(status llms-txt-exists)" "$(status redirect-behavior)"
exit 0"#;

/// With nothing failing now or on the last report, `agents` leaves out the
/// line about what's failing since then: the counts above it say it all.
/// One that failed then and passes now keeps it.
#[cfg(unix)]
#[test]
fn agents_says_nothing_since_when_nothing_fails_then_or_now() {
    let dir = project(MODEL);
    script(&dir.path().join("tools"), "afdocs", AFDOCS_FAILS_BY_FILE);
    let site = "https://docs.example.com";

    agents_on(dir.path(), "2026-10-09", site, &[]);
    let text = stdout(&agents_on(dir.path(), "2026-10-10", site, &[]));
    assert!(text.contains("2 pass."), "{text}");
    assert!(!text.contains(" checks: "), "{text}");
    let report = json(&agents_on(
        dir.path(),
        "2026-10-11",
        site,
        &["--format", "json"],
    ));
    let agents = &report["agents"];
    assert_eq!(agents["previous_run"], "2026-10-10");
    assert_eq!(
        (&agents["new"], &agents["resolved"]),
        (&0.into(), &0.into())
    );

    // The redirects fail, then pass again: the line says so.
    fs::write(dir.path().join("redirect-behavior"), "").unwrap();
    agents_on(dir.path(), "2026-10-12", site, &[]);
    fs::remove_file(dir.path().join("redirect-behavior")).unwrap();
    let text = stdout(&agents_on(dir.path(), "2026-10-13", site, &[]));
    assert!(
        text.contains(
            "  2 checks: 2 passing, 0 failing; 1 that failed on 12 October no longer fails.\n"
        ),
        "{text}"
    );
}

/// What afdocs writes: three checks, each with the status written in a
/// file named for it in the folder it runs in, and passing without one.
#[cfg(unix)]
const AFDOCS_STATUS_BY_FILE: &str = r#"status() { if [ -f "$1" ]; then read -r s < "$1"; echo "$s"; else echo pass; fi; }
printf '{"url":"%s","results":[{"id":"llms-txt-exists","category":"content-discoverability","status":"%s","message":"checked"},{"id":"llms-txt-links-resolve","category":"content-discoverability","status":"%s","message":"checked"},{"id":"redirect-behavior","category":"url-stability","status":"%s","message":"checked"}]}' "$2" "$(status llms-txt-exists)" "$(status llms-txt-links-resolve)" "$(status redirect-behavior)"
exit 0"#;

/// A check that went from failing to warning is still failing, so it isn't
/// one that no longer fails; and the check new since the last report is
/// named next to what says it's new, not after what no longer fails.
#[cfg(unix)]
#[test]
fn a_check_that_warns_instead_of_failing_still_fails() {
    let dir = project(MODEL);
    script(&dir.path().join("tools"), "afdocs", AFDOCS_STATUS_BY_FILE);
    let site = "https://docs.example.com";
    let set = |id: &str, status: &str| fs::write(dir.path().join(id), status).unwrap();

    set("llms-txt-exists", "fail");
    set("redirect-behavior", "fail");
    agents_on(dir.path(), "2026-10-24", site, &[]);

    // The first warns now, the second passes, and the links are new.
    set("llms-txt-exists", "warn");
    set("redirect-behavior", "pass");
    set("llms-txt-links-resolve", "warn");
    let text = stdout(&agents_on(dir.path(), "2026-10-25", site, &[]));
    assert!(
        text.contains(
            "  3 checks: 1 passing, 2 failing, 1 of them failing since 24 October: llms-txt-links-resolve; 1 that failed on 24 October no longer fails.\n"
        ),
        "{text}"
    );
}

/// A project declaring an API whose spec has three operations: `getInvoice`,
/// which its page links to, `deleteInvoice`, which it doesn't, and a
/// deprecated one.
fn api_project() -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    write(
        dir.path(),
        "ascribe.toml",
        "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[apis.public]\nspec = \"api/openapi.yaml\"\nreference = { renderer = \"redoc\", at = \"/api/\" }\n",
    );
    write(
        dir.path(),
        "api/openapi.yaml",
        "openapi: 3.1.0\ninfo: { title: Invoices, version: \"1\" }\npaths:\n  /invoices/{id}:\n    get:\n      operationId: getInvoice\n    delete:\n      operationId: deleteInvoice\n  /invoices:\n    get:\n      operationId: listInvoices\n      deprecated: true\n",
    );
    write(
        dir.path(),
        "docs/index.md",
        "---\ntitle: Invoices\n---\n\nFetch one with [Get an invoice](../api/openapi.yaml#getInvoice).\n",
    );
    dir
}

#[test]
fn check_and_the_inventory_say_which_operations_no_page_links_to() {
    let dir = api_project();
    let out = ascribe(dir.path(), &["check"]);
    assert_eq!(code_of(&out), 0, "{}", stderr(&out));
    let text = format!("{}{}", stdout(&out), stderr(&out));
    assert!(
        text.contains(
            "[ASC282] Advice: no page links to `deleteInvoice` (DELETE /invoices/{id}) in api/openapi.yaml"
        ),
        "{text}"
    );
    assert!(!text.contains("listInvoices"), "{text}");

    let text = stdout(&ascribe(dir.path(), &["report", "inventory"]));
    assert!(
        text.contains("APIs: public: 3 operations, 1 linked, 0 with a checked example."),
        "{text}"
    );
    let report = json(&ascribe(
        dir.path(),
        &["report", "inventory", "--format", "json"],
    ));
    assert_eq!(
        report["inventory"]["apis"],
        serde_json::json!([
            { "api": "public", "operations": 3, "linked": 1, "examples": 0 }
        ])
    );

    // A project without an API has no such key.
    let plain = project(MODEL);
    let report = json(&ascribe(plain.path(), &["report", "--format", "json"]));
    assert!(report["inventory"].get("apis").is_none(), "{report}");
}

#[test]
fn the_inventory_counts_the_operations_a_marked_example_claims() {
    let dir = api_project();
    write(
        dir.path(),
        "docs/index.md",
        "---\ntitle: Invoices\n---\n\nFetch one with [Get an invoice](../api/openapi.yaml#getInvoice).\n\n```json response=\"../api/openapi.yaml#getInvoice 200\"\n{}\n```\n\n```json response=\"../api/openapi.yaml#getInvoice 404\"\n{}\n```\n",
    );
    let report = json(&ascribe(
        dir.path(),
        &["report", "inventory", "--format", "json"],
    ));
    // Two markers, one operation.
    assert_eq!(report["inventory"]["apis"][0]["examples"], 1, "{report}");
}
