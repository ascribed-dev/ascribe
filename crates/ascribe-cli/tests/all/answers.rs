//! The commands that answer questions (`explain`, `model`, `outline`, `link`,
//! `refs`, `render`, and `search`): their JSON, exit codes, and limits, by
//! running the binary. `tests/output.rs` has their output on `examples/quill`, which the
//! command reference shows.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;

/// A model with two builds: `cloud` leaves out what's only for
/// self-managed.
const MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[dimensions.deployment]
values = ["cloud", "self-managed"]
versionless = ["cloud", "self-managed"]

[phrases]
product = "Quill"

[editor]
build = "cloud"

[builds.cloud]
variants = { deployment = "cloud" }
availability = { filter = "cloud" }

[builds.self-managed]
variants = { deployment = "self-managed" }
availability = { filter = "self-managed" }
"#;

const INSTALL: &str = "---
title: Install Quill
---

@include: _fragments/before.md

## Configure
@id: configure

See [keys](keys.md#rotate-keys) and [{product}](https://quill.dev).

## Upgrade
@available: self-managed

Upgrade {product}.
";

const BEFORE: &str = "## Before you start

You need {product}.
";

const KEYS: &str = "---
title: API keys
---

## Rotate keys

Rotate them.
";

const SERVER: &str = "---
title: Run the server
available: self-managed
---

Run it.
";

/// A project in a subfolder, `site/`, of a temporary directory.
fn project() -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let site = dir.path().join("site");
    write(&site.join("ascribe.toml"), MODEL);
    write(&site.join("docs/install.md"), INSTALL);
    write(&site.join("docs/_fragments/before.md"), BEFORE);
    write(&site.join("docs/keys.md"), KEYS);
    write(&site.join("docs/server.md"), SERVER);
    dir
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().expect("a parent")).expect("create directories");
    fs::write(path, text).expect("write a file");
}

fn ascribe(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(dir)
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("run ascribe")
}

/// Runs `ascribe` and expects it to exit with `code`.
fn run(dir: &Path, args: &[&str], code: i32) -> Output {
    let out = ascribe(dir, args);
    assert_eq!(
        out.status.code(),
        Some(code),
        "ascribe {args:?}\nstdout: {}\nstderr: {}",
        stdout(&out),
        stderr(&out)
    );
    out
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Runs `ascribe ... --format json`, expects `code`, and checks the
/// document's version.
fn json(dir: &Path, args: &[&str], code: i32) -> Value {
    let mut args = args.to_vec();
    args.extend(["--format", "json"]);
    let out = run(dir, &args, code);
    let value: Value = serde_json::from_slice(&out.stdout).expect("the answer is JSON");
    // `model`'s answer is at its own version; every other one here is at 1.
    let version = if args.contains(&"model") {
        ascribe_query::model::MODEL_SCHEMA_VERSION
    } else {
        1
    };
    assert_eq!(value["schema_version"], version, "{value}");
    assert_eq!(value["ascribe_version"], env!("CARGO_PKG_VERSION"));
    value
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn explain_answers_by_code_or_name() {
    let dir = tempfile::tempdir().unwrap();
    let by_code = json(dir.path(), &["explain", "asc036"], 0);
    let by_name = json(dir.path(), &["explain", "link-target-missing"], 0);
    assert_eq!(by_code, by_name);
    assert_eq!(by_code["code"], "ASC036");
    assert_eq!(by_code["severity"], "error");
    assert!(by_code["example"]["wrong"].is_string());
    assert!(
        by_code["docs"].as_str().unwrap().ends_with("/go/ASC036/"),
        "{by_code}"
    );
}

#[test]
fn explain_names_the_closest_for_an_unknown_code() {
    let dir = tempfile::tempdir().unwrap();
    let out = run(dir.path(), &["explain", "link-targt"], 2);
    assert!(stdout(&out).is_empty());
    assert!(
        stderr(&out).contains("ASC036 link-target-missing"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn explain_with_no_code_names_the_list() {
    let dir = tempfile::tempdir().unwrap();
    let out = run(dir.path(), &["explain"], 2);
    assert!(stdout(&out).is_empty());
    assert!(
        stderr(&out).contains("`ascribe explain --list`"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn explain_lists_every_diagnostic() {
    let dir = tempfile::tempdir().unwrap();
    let text = stdout(&run(dir.path(), &["explain", "--list"], 0));
    let listed = json(dir.path(), &["explain", "--list"], 0);
    let diagnostics = listed["diagnostics"].as_array().unwrap();
    assert_eq!(text.lines().count(), diagnostics.len());
    let registry =
        fs::read_to_string(repository_root().join("tests/conformance/diagnostics.toml")).unwrap();
    let codes = registry
        .lines()
        .filter(|l| l.starts_with("code = "))
        .count();
    assert_eq!(diagnostics.len(), codes);
    assert!(text.lines().any(|l| l == "ASC036 link-target-missing"));
}

#[test]
fn model_fits_its_budget_for_quill() {
    let root = repository_root();
    let text = stdout(&run(&root, &["model", "examples/quill"], 0));
    assert!(text.chars().count() <= 4000, "{} characters", text.len());
    assert!(!text.contains("more:"), "{text}");
    assert!(text.contains("`{product}`: Quill"), "{text}");
}

#[test]
fn model_reads_every_example_content_model() {
    let root = repository_root();
    for entry in fs::read_dir(root.join("examples/content-models")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "toml") {
            continue;
        }
        let config = path.to_str().unwrap();
        let text = stdout(&run(&root, &["--config", config, "model"], 0));
        assert!(text.starts_with("# Content model"), "{config}: {text}");
        let value = json(&root, &["--config", config, "model"], 0);
        for section in [
            "types",
            "dimensions",
            "phrases",
            "features",
            "glossary",
            "widgets",
            "builds",
            "sources",
            "project",
            "consumer",
            "editor",
            "checks",
            "crawlers",
            "listing",
            "permalinks",
        ] {
            assert!(value[section].is_array(), "{config}: no {section}");
        }
    }
}

#[test]
fn model_cuts_a_long_list_and_names_the_section() {
    let dir = tempfile::tempdir().unwrap();
    let mut model = String::from("spec = \"0.1\"\n\n[phrases]\n");
    for i in 0..400 {
        let _ = writeln!(model, "phrase-{i} = \"The value of phrase number {i}\"");
    }
    write(&dir.path().join("ascribe.toml"), &model);
    fs::create_dir(dir.path().join("docs")).unwrap();
    let text = stdout(&run(dir.path(), &["model"], 0));
    assert!(text.chars().count() <= 4000, "{} characters", text.len());
    assert!(
        text.contains("more: `ascribe model --section phrases`"),
        "{text}"
    );
    let full = stdout(&run(dir.path(), &["model", "--section", "phrases"], 0));
    assert!(full.contains("phrase-399"), "{full}");
    let one = json(dir.path(), &["model", "--section", "phrases"], 0);
    assert_eq!(one["phrases"].as_array().unwrap().len(), 400);
    assert!(one.get("types").is_none(), "{one}");
}

/// Every key of a section of settings, from the registry, as a JSON
/// section has them: `(table, key)` for the keys a table of its own has, such
/// as `[checks.vale]`'s, and the key alone for the rest.
fn registry_keys(section: &str) -> Vec<String> {
    let all = ascribe_model::settings::settings();
    let mut out = Vec::new();
    for s in all.iter().filter(|s| s.table == section) {
        let inner = format!("{section}.{}", s.key);
        if s.kind == "table" && all.iter().any(|t| t.table == inner) {
            out.extend(
                registry_keys(&inner)
                    .into_iter()
                    .map(|k| format!("{}.{k}", s.key)),
            );
        } else {
            out.push(s.key.clone());
        }
    }
    out
}

#[test]
fn model_shows_each_section_of_settings_with_its_values_and_defaults() {
    let root = repository_root();
    let text = stdout(&run(
        &root,
        &["--config", "docs", "model", "--section", "consumer"],
        0,
    ));
    assert!(text.contains("## `[consumer]`"), "{text}");
    for key in registry_keys("consumer") {
        assert!(text.contains(&format!("- `{key}`: ")), "{key}: {text}");
    }
    // The docs set `base-path`, and leave `slugger` to its default.
    assert!(
        text.contains("- `base-path`: `\"/docs/\"` (default `\"/\"`)"),
        "{text}"
    );
    assert!(
        text.contains("- `slugger`: `\"github\"` (the default)"),
        "{text}"
    );

    let whole = json(&root, &["--config", "docs", "model"], 0);
    for section in [
        "project",
        "consumer",
        "editor",
        "checks",
        "crawlers",
        "listing",
        "permalinks",
    ] {
        let keys: Vec<String> = whole[section]
            .as_array()
            .unwrap_or_else(|| panic!("no {section}: {whole}"))
            .iter()
            // A check in `[checks]` is one the file names.
            .filter(|s| !(section == "checks" && s["table"] == "checks"))
            .map(|s| {
                let table = s["table"].as_str().unwrap();
                let key = s["key"].as_str().unwrap();
                match table.strip_prefix(&format!("{section}.")) {
                    Some(inner) => format!("{inner}.{key}"),
                    None => key.to_owned(),
                }
            })
            .collect();
        assert_eq!(keys, registry_keys(section), "{section}");
    }
}

#[test]
fn model_shows_settings_the_file_sets_and_the_defaults_it_leaves() {
    let dir = tempfile::tempdir().unwrap();
    write(
        &dir.path().join("ascribe.toml"),
        "spec = \"0.1\"\n\n[consumer]\nsite = \"https://example.com\"\n\n\
         [checks]\npage-size = \"warning\"\n\n[checks.links]\nreuse-days = 7\n",
    );
    fs::create_dir(dir.path().join("docs")).unwrap();
    let report = json(dir.path(), &["model"], 0);
    let find = |section: &str, table: &str, key: &str| -> Value {
        report[section]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["table"] == table && s["key"] == key)
            .unwrap_or_else(|| panic!("no {table} {key}: {report}"))
            .clone()
    };
    let site = find("consumer", "consumer", "site");
    assert_eq!(site["value"], "https://example.com");
    assert_eq!(site["set"], true);
    assert!(site.get("default").is_none(), "{site}");
    let base = find("consumer", "consumer", "base-path");
    assert_eq!(base["value"], "/");
    assert_eq!(base["set"], false);
    assert_eq!(base["default"], "/");
    let reuse = find("checks", "checks.links", "reuse-days");
    assert_eq!(
        (reuse["value"].clone(), reuse["default"].clone()),
        (7.into(), 14.into())
    );
    let check = find("checks", "checks", "page-size");
    assert_eq!(check["value"], "warning");
    assert_eq!(check["kind"], "level");
    // The editor's build, which the loader chose: the only one.
    let build = find("editor", "editor", "build");
    assert_eq!(
        (build["value"].clone(), build["set"].clone()),
        ("site".into(), false.into())
    );

    let text = stdout(&run(dir.path(), &["model"], 0));
    assert!(
        text.contains(
            "`permalinks`, each with its keys and values: `ascribe model --section <name>`"
        ),
        "{text}"
    );
    assert!(!text.contains("`[consumer]`"), "{text}");
    let checks = stdout(&run(dir.path(), &["model", "--section", "checks"], 0));
    assert!(
        checks.contains("- `links.reuse-days`: `7` (default `14`)"),
        "{checks}"
    );
    assert!(checks.contains("- `page-size`: `\"warning\"`"), "{checks}");
}

#[test]
fn model_shows_sources() {
    let dir = tempfile::tempdir().unwrap();
    write(
        &dir.path().join("ascribe.toml"),
        "spec = \"0.1\"\n\n[sources.code]\npath = \"..\"\ninclude = [\"src/**\"]\nignore = [\"**/target/**\"]\n",
    );
    fs::create_dir(dir.path().join("docs")).unwrap();
    let text = stdout(&run(dir.path(), &["model"], 0));
    assert!(
        text.contains(
            "## Sources\n\n- `code:` (folder `..`; files `src/**`; not `**/target/**`)\n"
        ),
        "{text}"
    );
    let one = json(dir.path(), &["model", "--section", "sources"], 0);
    let source = &one["sources"][0];
    assert_eq!(source["name"], "code");
    assert_eq!(source["path"], "..");
    assert_eq!(source["include"][0], "src/**");
    assert_eq!(source["ignore"][0], "**/target/**");
    assert!(source["git"].is_null(), "{one}");
    assert!(source.get("examples").is_none(), "{one}");
    assert!(one.get("types").is_none(), "{one}");
}

/// A source with an `ascribe.examples.toml` says how many of its files are
/// examples, and each observation's environment and how many of its files
/// are as it found them.
#[test]
fn model_shows_a_source_s_tested_examples() {
    let dir = tempfile::tempdir().unwrap();
    write(
        &dir.path().join("ascribe.toml"),
        "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[sources.code]\npath = \"code\"\nignore = [\"examples/hidden.ts\"]\n",
    );
    fs::create_dir(dir.path().join("docs")).unwrap();
    let code = dir.path().join("code");
    write(
        &code.join("ascribe.examples.toml"),
        "include = [\"examples/**\"]\n\n[environment]\nnode = { label = \"Node\", command = \"node --version\" }\n",
    );
    write(&code.join("examples/a.ts"), "a();\n");
    write(&code.join("examples/b.ts"), "b();\n");
    write(&code.join("examples/hidden.ts"), "hidden();\n");
    write(&code.join("helper.ts"), "helper();\n");
    write(
        &code.join("examples.observations/node-22.toml"),
        &format!(
            "version = 1\nascribe = \"0.4.0\"\ncommand = \"x\"\n\n[environment]\nos = \"linux\"\narch = \"x64\"\nnode = \"v22.4.0\"\n\n[files]\n\"examples/a.ts\" = \"{}\"\n\"examples/b.ts\" = \"{}\"\n",
            ascribe_model::example_hash(b"a();\n"),
            ascribe_model::example_hash(b"old\n"),
        ),
    );
    let one = json(dir.path(), &["model", "--section", "sources"], 0);
    assert_eq!(
        one["sources"][0]["examples"],
        serde_json::json!({
            "valid": true,
            "files": 2,
            "observations": [{
                "name": "node-22",
                "environment": { "arch": "x64", "node": "v22.4.0", "os": "linux" },
                "tested": "Node 22.4.0 on Linux",
                "listed": 2,
                "current": 1,
            }],
        })
    );
    let text = stdout(&run(dir.path(), &["model", "--section", "sources"], 0));
    assert!(
        text.contains(
            "- `code:` (folder `code`; not `examples/hidden.ts`; 2 tested examples, observation `node-22`: Node 22.4.0 on Linux, 1 of 2 files as observed)\n"
        ),
        "{text}"
    );
}

#[test]
fn model_with_errors_says_to_check() {
    let dir = tempfile::tempdir().unwrap();
    write(
        &dir.path().join("ascribe.toml"),
        "spec = \"0.1\"\n[phrases\n",
    );
    let out = run(dir.path(), &["model"], 2);
    assert!(stderr(&out).contains("ascribe check"), "{}", stderr(&out));
}

#[test]
fn outline_lists_headings_from_fragments() {
    let dir = project();
    let value = json(dir.path(), &["outline", "site/docs/install.md"], 0);
    assert_eq!(value["page"], "install.md");
    assert_eq!(value["file"], "docs/install.md");
    assert_eq!(value["title"], "Install Quill");
    assert_eq!(value["type"], "page");
    let headings = value["headings"].as_array().unwrap();
    let ids: Vec<&str> = headings.iter().map(|h| h["id"].as_str().unwrap()).collect();
    assert_eq!(ids, ["before-you-start", "configure", "upgrade"]);
    assert_eq!(headings[0]["fragment"], "_fragments/before.md");
    assert_eq!(headings[0]["file"], "docs/_fragments/before.md");
    assert_eq!(headings[0]["line"], 1);
    assert_eq!(headings[1]["explicit_id"], true);
    assert_eq!(headings[2]["explicit_id"], false);
}

#[test]
fn outline_with_a_build_shows_what_it_publishes() {
    let dir = project();
    let value = json(
        dir.path(),
        &["outline", "site/docs/install.md", "--build", "cloud"],
        0,
    );
    let ids: Vec<&str> = value["headings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["before-you-start", "configure"]);

    let out = run(
        dir.path(),
        &["outline", "site/docs/server.md", "--build", "cloud"],
        1,
    );
    assert!(
        stderr(&out).contains("build `cloud` doesn't publish `server.md`"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn outline_of_a_file_that_isnt_a_source_fails() {
    let dir = project();
    let out = run(dir.path(), &["outline", "site/docs/missing.md"], 2);
    assert!(stderr(&out).contains("missing.md"), "{}", stderr(&out));
}

#[test]
fn link_answers_from_a_page() {
    let dir = project();
    let from = "site/docs/install.md";
    let value = json(
        dir.path(),
        &["link", "keys.md#rotate-keys", "--from", from],
        0,
    );
    assert_eq!(value["exists"], true);
    assert_eq!(value["from"], from, "the page as it was given");
    assert_eq!(value["kind"], "heading");
    assert_eq!(value["title"], "Rotate keys");
    assert_eq!(value["href"], "keys.md#rotate-keys");

    let own = json(dir.path(), &["link", "#configure", "--from", from], 0);
    assert_eq!(own["href"], "#configure");

    let external = json(
        dir.path(),
        &["link", "https://quill.dev", "--from", from],
        0,
    );
    assert_eq!(external["kind"], "external");

    let missing = json(dir.path(), &["link", "key.md", "--from", from], 1);
    assert_eq!(missing["exists"], false);
    assert!(missing["href"].is_null());
    assert_eq!(missing["closest"][0]["path"], "keys.md");

    // A page whose name is far from the one written is found by the heading
    // the link names.
    let renamed = json(
        dir.path(),
        &["link", "credentials.md#rotate-keys", "--from", from],
        1,
    );
    assert_eq!(renamed["closest"][0]["href"], "keys.md#rotate-keys");
    assert_eq!(renamed["closest"][0]["title"], "Rotate keys");

    let fragment = json(
        dir.path(),
        &["link", "_fragments/before.md", "--from", from],
        1,
    );
    assert_eq!(fragment["kind"], "fragment");
    assert_eq!(fragment["closest"][0]["path"], "install.md");

    // A page given from the content root is named back from the folder the
    // command was run in, as `ascribe check` names it.
    let site = dir.path().join("site");
    let value = json(&site, &["link", "keys.md", "--from", "install.md"], 0);
    assert_eq!(value["from"], "docs/install.md");
}

#[test]
fn refs_finds_uses_through_includes() {
    let dir = project();
    let heading = json(dir.path(), &["refs", "site/docs/keys.md#rotate-keys"], 0);
    assert_eq!(heading["kind"], "heading");
    assert_eq!(heading["total"], 1);
    assert_eq!(heading["places"][0]["file"], "docs/install.md");
    assert_eq!(heading["places"][0]["use"], "link");

    let fragment = json(dir.path(), &["refs", "site/docs/_fragments/before.md"], 0);
    assert_eq!(fragment["kind"], "fragment");
    assert_eq!(fragment["places"][0]["use"], "include");
    assert_eq!(fragment["places"][0]["line"], 5);

    let phrase = json(
        dir.path(),
        &["refs", "phrase:{product}", "--config", "site"],
        0,
    );
    assert_eq!(phrase["total"], 3);
    assert_eq!(phrase["truncated"], false);
    assert!(phrase["next_command"].is_null());

    // `--config` is how to name the project: `refs` has no option of its own.
    run(
        dir.path(),
        &["refs", "phrase:product", "--project", "site"],
        2,
    );
}

#[test]
fn refs_cuts_the_list_and_says_how_to_see_it_all() {
    let dir = project();
    let args = ["refs", "phrase:product", "--config", "site", "--limit", "1"];
    let value = json(dir.path(), &args, 0);
    assert_eq!(value["shown"], 1);
    assert_eq!(value["total"], 3);
    assert_eq!(value["truncated"], true);
    assert_eq!(
        value["next_command"],
        "ascribe refs phrase:product --limit 3 --config site --format json"
    );
    let text = stdout(&run(dir.path(), &args, 0));
    assert!(text.contains("Showing 1 of them"), "{text}");

    // A word a shell would split is quoted, so the command runs as given.
    write(
        &dir.path().join("site/docs/api keys.md"),
        "---\ntitle: API keys\n---\n\nSee [rotating](keys.md) and [them](keys.md#rotate-keys).\n",
    );
    write(
        &dir.path().join("site/docs/more.md"),
        "---\ntitle: More\n---\n\nSee [keys](api%20keys.md) and [again](<api keys.md>).\n",
    );
    let spaced = json(
        dir.path(),
        &["refs", "site/docs/api keys.md", "--limit", "1"],
        0,
    );
    assert_eq!(spaced["total"], 2, "{spaced}");
    assert_eq!(
        spaced["next_command"],
        "ascribe refs 'site/docs/api keys.md' --limit 2 --format json"
    );
}

#[test]
fn refs_lists_no_use_from_inside_a_memo() {
    let dir = project();
    let install = || json(dir.path(), &["refs", "site/docs/install.md"], 0)["total"].clone();
    let before = install();
    let keys = dir.path().join("site/docs/keys.md");
    let text = fs::read_to_string(&keys).expect("keys.md");
    let memo = "@memo:\nSee [install](install.md), [the incident](notes/gone.md), and \
                {product}.\n@end\n";
    write(&keys, &format!("{text}\n{memo}Text.\n"));
    assert_eq!(install(), before);
    let phrase = json(
        dir.path(),
        &["refs", "phrase:product", "--config", "site"],
        0,
    );
    assert_eq!(phrase["total"], 3, "{phrase}");
    // The same link outside the memo is a use.
    write(&keys, &format!("{text}\nSee [install](install.md).\n"));
    assert_ne!(install(), before);
}

#[test]
fn a_memo_about_a_term_is_outlined_and_found_under_either_name() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let site = dir.path().join("site");
    write(
        &site.join("ascribe.toml"),
        "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[glossary]\npages = [\"glossary.md\"]\n",
    );
    write(
        &site.join("docs/glossary.md"),
        "---\ntitle: Glossary\n---\n\n## Region\n@id: area\n@term\n\nWhere a workspace runs.\n",
    );
    write(
        &site.join("memos/region.md"),
        "---\nabout: term:area\nquote: Where a workspace runs\n---\n\nWhy.\n",
    );
    let outline = json(dir.path(), &["outline", "site/docs/glossary.md"], 0);
    assert_eq!(
        outline["headings"][0]["memos"][0]["file"], "memos/region.md",
        "{outline}"
    );
    for args in [
        &["refs", "term:area", "--config", "site"][..],
        &["refs", "site/docs/glossary.md#area"],
    ] {
        let refs = json(dir.path(), args, 0);
        let memos: Vec<&str> = refs["places"]
            .as_array()
            .expect("places")
            .iter()
            .filter(|p| p["use"] == "memo")
            .map(|p| p["file"].as_str().unwrap())
            .collect();
        assert_eq!(memos, ["memos/region.md"], "{args:?}: {refs}");
    }
}

#[test]
fn a_personal_memo_is_listed_and_not_checked() {
    let dir = project();
    let site = dir.path().join("site");
    write(
        &site.join("memos/team.md"),
        "---\nabout: install.md#configure\n---\n\nThe team's reason.\n",
    );
    write(
        &site.join(".ascribe/memos/mine.md"),
        "---\nabout: install.md#configure\n---\n\nMy reason.\n",
    );
    let outline = json(dir.path(), &["outline", "site/docs/install.md"], 0);
    let configure = outline["headings"]
        .as_array()
        .and_then(|h| h.iter().find(|h| h["id"] == "configure"))
        .expect("the heading");
    let memos: Vec<(&str, bool)> = configure["memos"]
        .as_array()
        .expect("memos")
        .iter()
        .map(|m| {
            (
                m["file"].as_str().unwrap(),
                m["personal"].as_bool().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        memos,
        [("memos/team.md", false), (".ascribe/memos/mine.md", true)]
    );
    let refs = json(dir.path(), &["refs", "site/docs/install.md#configure"], 0);
    let personal: Vec<bool> = refs["places"]
        .as_array()
        .expect("places")
        .iter()
        .filter(|p| p["use"] == "memo")
        .map(|p| p["personal"].as_bool().unwrap())
        .collect();
    assert_eq!(personal, [false, true], "{refs}");
    // A personal memo about nothing is the editor's to report, not the
    // check's, whose report is the same on every machine.
    write(
        &site.join(".ascribe/memos/gone.md"),
        "---\nabout: nowhere.md\n---\n\nLost.\n",
    );
    let check = stdout(&run(&site, &["check", "--format", "json"], 0));
    assert!(!check.contains("memo-detached"), "{check}");
}

#[test]
fn refs_to_nothing_is_exit_code_1() {
    let dir = project();
    let value = json(dir.path(), &["refs", "phrase:nope", "--config", "site"], 1);
    assert_eq!(value["exists"], false);
    run(dir.path(), &["refs", "site/docs/keys.md#nope"], 1);
    run(dir.path(), &["refs", "phrase:", "--config", "site"], 2);
}

#[test]
fn refs_lists_where_an_image_is_shown_linked_and_named() {
    let dir = project();
    let site = dir.path().join("site");
    for image in ["docs/images/keys.png", "docs/images/unused.png"] {
        fs::create_dir_all(site.join(image).parent().unwrap()).unwrap();
        fs::write(site.join(image), b"\x89PNG").unwrap();
    }
    write(&site.join("docs/notes.txt"), "Not a page.\n");
    write(
        &site.join("docs/shown.md"),
        "---\ntitle: Shown\ncover: images/keys.png\n---\n\n![Keys](images/keys.png)\n\n\
         See [the picture](images/keys.png).\n\n<img src=\"images/keys.png\" alt=\"Keys\">\n",
    );
    let refs = json(dir.path(), &["refs", "site/docs/images/keys.png"], 0);
    assert_eq!(refs["kind"], "image", "{refs}");
    assert_eq!(refs["exists"], true);
    let places: Vec<(&str, u64, u64, &str)> = refs["places"]
        .as_array()
        .expect("places")
        .iter()
        .map(|p| {
            (
                p["file"].as_str().unwrap(),
                p["line"].as_u64().unwrap(),
                p["column"].as_u64().unwrap(),
                p["use"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        places,
        [
            ("docs/shown.md", 3, 8, "frontmatter"),
            ("docs/shown.md", 6, 1, "image"),
            ("docs/shown.md", 8, 5, "link"),
            ("docs/shown.md", 10, 11, "html"),
        ],
        "{refs}"
    );
    // From the content root too, as a page is.
    let from_root = json(&site, &["refs", "images/keys.png"], 0);
    assert_eq!(from_root["total"], 4, "{from_root}");

    // An image nothing uses exists, so it's no problem.
    let unused = json(dir.path(), &["refs", "site/docs/images/unused.png"], 0);
    assert_eq!(unused["kind"], "image");
    assert_eq!(unused["total"], 0);
    let text = stdout(&run(
        dir.path(),
        &["refs", "site/docs/images/unused.png"],
        0,
    ));
    assert_eq!(text, "0 places use site/docs/images/unused.png.\n");

    // A file that's neither a source nor an image says what `refs` takes.
    let out = run(dir.path(), &["refs", "site/docs/notes.txt"], 2);
    let err = stderr(&out);
    assert!(
        err.contains("it's a file, but not a page, a fragment, or an image"),
        "{err}"
    );
    assert!(!stdout(&out).contains("doesn't exist"));
    run(dir.path(), &["refs", "site/docs/images/gone.png"], 1);
}

#[test]
fn refs_finds_the_image_reviews_page_shows_in_ascribes_docs() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (dir, target) in [
        ("docs", "images/review-site-preview.png"),
        (".", "docs/content/images/review-site-preview.png"),
    ] {
        let refs = json(&repo.join(dir), &["refs", target], 0);
        assert_eq!(refs["kind"], "image", "{refs}");
        let review = refs["places"]
            .as_array()
            .expect("places")
            .iter()
            .any(|p| p["file"] == "content/review/index.md" && p["use"] == "image");
        assert!(review, "from {dir}: {refs}");
    }
}

#[test]
fn render_writes_the_page_a_build_publishes() {
    let dir = project();
    let page = "site/docs/install.md";
    let text = stdout(&run(dir.path(), &["render", page, "--build", "cloud"], 0));
    assert!(text.contains("You need Quill."), "{text}");
    assert!(!text.contains("Upgrade"), "{text}");
    assert!(!text.starts_with("---"), "{text}");
    let other = stdout(&run(
        dir.path(),
        &["render", page, "--build", "self-managed", "--frontmatter"],
        0,
    ));
    assert!(
        other.starts_with("---\ntitle: Install Quill\n---\n"),
        "{other}"
    );
    assert!(other.contains("Upgrade Quill."), "{other}");
}

#[test]
fn render_says_a_reader_sees_one_arm_of_a_switcher() {
    let dir = tempfile::tempdir().unwrap();
    write(
        &dir.path().join("ascribe.toml"),
        &MODEL.replace(
            "[phrases]",
            "[dimensions.pm]\nvalues = [\"npm\", \"pnpm\"]\nversionless = [\"npm\", \"pnpm\"]\n\n[phrases]",
        ),
    );
    write(
        &dir.path().join("docs/setup.md"),
        "---\ntitle: Set up\n---\n\n@variant {pm=npm}:\nRun npm.\n@variant {pm=pnpm}:\nRun pnpm.\n@end\n\n.Docker\n@variant:\nUse the image.\n\n.Binary\n@variant:\nUse the binary.\n@end\n\n@variant {deployment=cloud}:\nSign in.\n@variant {deployment=self-managed}:\nStart it.\n@end\n",
    );
    let args = ["render", "docs/setup.md", "--build", "cloud"];
    let text = stdout(&run(dir.path(), &args, 0));
    assert!(
        text.contains(
            "A reader sees one of the next 2 sections at a time, chosen by pm.\n\n## pm: npm\n"
        ),
        "{text}"
    );
    assert!(
        text.contains("A reader sees one of the next 2 sections at a time.\n\n## Docker\n"),
        "{text}"
    );
    assert_eq!(
        text.matches("A reader sees").count(),
        2,
        "a group the build reduces to one arm isn't one: {text}"
    );
    let value = json(dir.path(), &args, 0);
    assert_eq!(
        value["text"].as_str().unwrap(),
        text,
        "the JSON's text is the plain output's, as the text is"
    );
}

#[test]
fn render_says_why_a_build_leaves_a_page_out() {
    let dir = project();
    let out = run(
        dir.path(),
        &["render", "site/docs/server.md", "--build", "cloud"],
        1,
    );
    assert!(stdout(&out).is_empty(), "{}", stdout(&out));
    assert!(stderr(&out).contains("cloud"), "{}", stderr(&out));
    let value = json(
        dir.path(),
        &["render", "site/docs/server.md", "--build", "cloud"],
        1,
    );
    assert!(value["not_published"].is_string());
    assert_eq!(value["text"], "");
}

#[test]
fn render_needs_a_build_and_a_page() {
    let dir = project();
    let out = run(dir.path(), &["render", "site/docs/keys.md"], 2);
    assert!(stderr(&out).contains("--build"), "{}", stderr(&out));
    run(
        dir.path(),
        &[
            "render",
            "site/docs/_fragments/before.md",
            "--build",
            "cloud",
        ],
        2,
    );
    run(
        dir.path(),
        &["render", "site/docs/keys.md", "--build", "nope"],
        2,
    );
}

#[test]
fn search_finds_words_through_phrases_in_the_builds_that_publish_them() {
    let dir = project();
    let out = run(
        dir.path(),
        &["search", "upgrade quill", "--config", "site"],
        0,
    );
    let text = stdout(&out);
    assert!(
        text.starts_with("docs/install.md:15: Upgrade Quill.\n"),
        "{text}"
    );
    assert!(text.contains("through phrase product"), "{text}");
    assert!(text.contains("in build self-managed"), "{text}");
    assert!(text.ends_with("1 place matches, on 1 page.\n"), "{text}");
}

#[test]
fn search_finds_a_phrase_value_at_its_line_in_the_model() {
    let quill = repository_root().join("examples/quill");
    let out = run(&quill, &["search", "\"quill cloud\""], 0);
    let text = stdout(&out);
    assert!(text.starts_with("ascribe.toml:34: Quill Cloud\n"), "{text}");
    assert!(text.contains("`ascribe refs phrase:cloud`"), "{text}");
}

#[test]
fn search_exits_1_when_nothing_matches() {
    let dir = project();
    let out = run(dir.path(), &["search", "zebra", "--config", "site"], 1);
    assert_eq!(stdout(&out), "No place matches.\n");
}

#[test]
fn search_cuts_the_list_and_says_how_to_see_it_all() {
    let dir = project();
    let args = ["search", "upgrade", "--config", "site", "--limit", "1"];
    let text = stdout(&run(dir.path(), &args, 0));
    assert!(
        text.ends_with(
            "2 places match, on 1 page.\nShowing 1 of them; for all: `ascribe search upgrade --limit 2 --config site`\n"
        ),
        "{text}"
    );
}

#[test]
fn search_says_to_quote_several_words() {
    let dir = project();
    let out = run(
        dir.path(),
        &["search", "rate", "limit", "--config", "site"],
        2,
    );
    assert!(
        stderr(&out).contains("quote several words together: ascribe search 'rate limit'"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn search_stops_on_a_query_or_a_build_it_cannot_read() {
    let dir = project();
    for args in [
        &["search", "\"rate limit", "--config", "site"][..],
        &["search", " ", "--config", "site"],
        &["search", "quill", "--build", "nope", "--config", "site"],
    ] {
        let out = run(dir.path(), args, 2);
        assert!(stderr(&out).starts_with("error: "), "{}", stderr(&out));
    }
}
