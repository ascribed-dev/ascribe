//! The commands that answer questions (`explain`, `model`, `outline`, `link`,
//! `refs`, `resolve`, and `search`): their JSON, exit codes, and limits, by
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
fn explain_text_writes_links_for_a_terminal() {
    let dir = tempfile::tempdir().unwrap();
    // The JSON keeps the registry's Markdown, for a tool to render.
    let value = json(dir.path(), &["explain", "page-orphan"], 0);
    assert!(
        value["fix"]
            .as_str()
            .unwrap()
            .contains("[`[checks]`](content-model/checks.md#19-checks)"),
        "{value}"
    );
    let text = stdout(&run(dir.path(), &["explain", "page-orphan"], 0));
    assert!(
        text.contains("turn the check off in `[checks]` (`page-orphan = \"off\"`)"),
        "{text}"
    );
    assert!(!text.contains("checks.md#"), "{text}");
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
        for section in ascribe_query::Section::ALL.map(|s| s.name()) {
            let shown = if section == "fragments" {
                value[section].is_object()
            } else {
                value[section].is_array()
            };
            assert!(shown, "{config}: no {section}");
        }
    }
}

#[test]
fn model_shows_each_section_alone() {
    let root = repository_root();
    let config = "examples/content-models/full.toml";
    for section in ascribe_query::Section::ALL.map(|s| s.name()) {
        let value = json(
            &root,
            &["--config", config, "model", "--section", section],
            0,
        );
        let shown: Vec<&String> = value
            .as_object()
            .unwrap()
            .keys()
            .filter(|k| !k.ends_with("_version"))
            .collect();
        assert_eq!(shown, [section], "{value}");
        let text = stdout(&run(
            &root,
            &["--config", config, "model", "--section", section],
            0,
        ));
        assert!(
            text.starts_with("# Content model\n\n## "),
            "{section}: {text}"
        );
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

#[test]
fn model_shows_the_tables_that_say_where_things_come_from() {
    let dir = tempfile::tempdir().unwrap();
    write(
        &dir.path().join("shared/ascribe.toml"),
        "spec = \"0.1\"\n\n[phrases]\nproduct = \"Quill\"\n",
    );
    write(&dir.path().join("api/openapi.yaml"), "openapi: 3.1.0\n");
    write(
        &dir.path().join("ascribe.toml"),
        "spec = \"0.1\"\n\n[from.\"shared/ascribe.toml\"]\ntake = [\"phrases.product\"]\n\n\
         [frontmatter]\nowner = \"string\"\n\n\
         [apis.loom]\nspec = \"api/openapi.yaml\"\n\
         reference = \"https://loom.dev/api/{operationId}\"\n\n\
         [[intended]]\ncheck = \"phrase-unused\"\nphrase = \"product\"\n\
         reason = \"The next release's pages use it.\"\n",
    );
    fs::create_dir(dir.path().join("docs")).unwrap();

    let text = stdout(&run(dir.path(), &["model"], 0));
    for line in [
        "## Frontmatter every type shares\n\n- `owner` (string, required)\n",
        "## APIs\n\n- `loom`: spec `api/openapi.yaml`, reference \
         `https://loom.dev/api/{operationId}`\n",
        "## Taken from other files\n\n- `shared/ascribe.toml`: takes `phrases.product`\n",
        "## Intended problems\n\n- `phrase-unused` about phrase `product`: \
         The next release's pages use it.\n",
    ] {
        assert!(text.contains(line), "{line}\n{text}");
    }
    // Built in, so in every project: shown when the file writes them.
    assert!(!text.contains("## Lifecycle states"), "{text}");
    assert!(!text.contains("## Note types"), "{text}");

    let value = json(dir.path(), &["model"], 0);
    assert_eq!(
        value["apis"],
        serde_json::json!([{
            "name": "loom",
            "spec": "api/openapi.yaml",
            "reference": "https://loom.dev/api/{operationId}",
        }])
    );
    assert_eq!(
        value["from"],
        serde_json::json!([{ "file": "shared/ascribe.toml", "take": ["phrases.product"] }])
    );
    assert_eq!(
        value["intended"],
        serde_json::json!([{
            "check": "phrase-unused",
            "kind": "phrase",
            "name": "product",
            "reason": "The next release's pages use it.",
        }])
    );
    assert_eq!(value["notes"][0]["name"], "note");
    assert_eq!(value["notes"][0]["builtin"], true);
}

#[test]
fn model_lists_the_implicit_release_dimension() {
    let dir = tempfile::tempdir().unwrap();
    write(
        &dir.path().join("ascribe.toml"),
        "spec = \"0.1\"\n\n[dimensions.pm]\nvalues = [\"npm\"]\n\n\
         [releases.ascribe]\nlabel = \"Ascribe\"\nlive = [\"0.4.1\"]\n",
    );
    fs::create_dir(dir.path().join("docs")).unwrap();
    let text = stdout(&run(dir.path(), &["model", "--section", "dimensions"], 0));
    assert!(text.contains("- `pm` (pm): `npm`\n"), "{text}");
    assert!(
        text.contains("- `release` (Release): `ascribe`\n"),
        "{text}"
    );
    let value = json(dir.path(), &["model", "--section", "dimensions"], 0);
    let release = &value["dimensions"][1];
    assert_eq!(release["name"], "release");
    assert_eq!(release["values"][0]["label"], "Ascribe");
}

#[test]
fn model_says_which_table_declares_each_field() {
    let dir = tempfile::tempdir().unwrap();
    write(
        &dir.path().join("ascribe.toml"),
        "spec = \"0.1\"\n\n[frontmatter]\ndescription = \"string\"\n\n\
         [types.guide.frontmatter]\nsteps = \"number\"\n\n\
         [types.index.frontmatter]\ndescription = \"string?\"\n\n\
         [types.note]\n",
    );
    fs::create_dir(dir.path().join("docs")).unwrap();
    let text = stdout(&run(dir.path(), &["model", "--section", "types"], 0));
    assert!(
        text.contains(
            "- `guide`: frontmatter `title` (string, required, implicit), \
             `description` (string, required, from `[frontmatter]`), `steps` (number, required, own)\n"
        ),
        "{text}"
    );
    assert!(
        text.contains(
            "- `index`: frontmatter `title` (string, required, implicit), `description` (string, own)\n"
        ),
        "{text}"
    );
    let value = json(dir.path(), &["model", "--section", "types"], 0);
    let declared = |ty: usize| -> Vec<(String, String)> {
        value["types"][ty]["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| {
                (
                    f["name"].as_str().unwrap().to_owned(),
                    f["declared"].as_str().unwrap().to_owned(),
                )
            })
            .collect()
    };
    let pairs = |list: &[(&str, &str)]| -> Vec<(String, String)> {
        list.iter()
            .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
            .collect()
    };
    assert_eq!(
        declared(0),
        pairs(&[
            ("title", "implicit"),
            ("description", "frontmatter"),
            ("steps", "type")
        ])
    );
    assert_eq!(
        declared(1),
        pairs(&[("title", "implicit"), ("description", "type")])
    );
    assert_eq!(
        declared(2),
        pairs(&[("title", "implicit"), ("description", "frontmatter")])
    );
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
        "project", "consumer", "editor", "checks", "crawlers", "listing",
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
fn model_shows_agents_on_by_default_only_with_a_site() {
    for (consumer, on) in [("site = \"https://example.com\"\n", true), ("", false)] {
        let dir = tempfile::tempdir().unwrap();
        write(
            &dir.path().join("ascribe.toml"),
            &format!("spec = \"0.1\"\n\n[consumer]\n{consumer}"),
        );
        fs::create_dir(dir.path().join("docs")).unwrap();
        let report = json(dir.path(), &["model", "--section", "consumer"], 0);
        let agents = report["consumer"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["key"] == "agents")
            .unwrap_or_else(|| panic!("no agents: {report}"))
            .clone();
        assert_eq!(agents["value"], on, "{agents}");
        assert_eq!(agents["default"], on, "{agents}");
        assert_eq!(agents["set"], false, "{agents}");
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
        text.contains("`listing`, each with its keys and values: `ascribe model --section <name>`"),
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

/// A model with a release target, `server`, whose lines are 2.0 and 2.1,
/// and 2.2 planned.
const RELEASES: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[dimensions.deployment]
values = ["cloud", "server"]
versionless = ["cloud"]

[releases.server]
live = ["2.1", "2.0"]
planned = ["2.2"]
"#;

fn releases() -> TempDir {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let site = dir.path().join("site");
    write(&site.join("ascribe.toml"), RELEASES);
    write(
        &site.join("docs/a.md"),
        "---\ntitle: A\navailable: server 2.1\n---\n\n## Old\n@available: server (preview 2.0, ga 2.1.3)\n\nOld.\n\n## Next\n@available: cloud, server preview planned\n\nNext.\n",
    );
    write(
        &site.join("docs/b.md"),
        "---\ntitle: B\n---\n\n@available: cloud\nCloud only.\n\n@available: deployment\nEverywhere.\n\n@available: server 2.0\nSince 2.0.\n",
    );
    dir
}

#[test]
fn refs_lists_the_marks_that_name_a_version_in_a_release_line() {
    let dir = releases();
    let refs = |target: &str| {
        let value = json(dir.path(), &["refs", target, "--config", "site"], 0);
        assert_eq!(value["kind"], "release", "{value}");
        assert_eq!(value["exists"], true, "{value}");
        value["places"]
            .as_array()
            .expect("places")
            .iter()
            .map(|p| {
                assert_eq!(p["use"], "availability", "{p}");
                format!(
                    "{}:{} {}",
                    p["file"].as_str().unwrap(),
                    p["line"],
                    p["spec"].as_str().unwrap()
                )
            })
            .collect::<Vec<_>>()
    };
    // A history names each of its versions; a version is in each line its
    // parts start with.
    assert_eq!(
        refs("release:server@2.1"),
        [
            "docs/a.md:3 server 2.1",
            "docs/a.md:7 server (preview 2.0, ga 2.1.3)",
        ]
    );
    assert_eq!(
        refs("release:server@2.0"),
        [
            "docs/a.md:7 server (preview 2.0, ga 2.1.3)",
            "docs/b.md:11 server 2.0",
        ]
    );
    assert_eq!(
        refs("release:server@2.1.3"),
        ["docs/a.md:7 server (preview 2.0, ga 2.1.3)"]
    );
    // `planned` is the planned line.
    assert_eq!(
        refs("release:server@2.2"),
        ["docs/a.md:12 cloud, server preview planned"]
    );
    assert!(refs("release:server@3").is_empty());

    let text = stdout(&run(
        dir.path(),
        &["refs", "release:server@2.0", "--config", "site"],
        0,
    ));
    assert_eq!(
        text,
        "docs/a.md:7:13: availability `server (preview 2.0, ga 2.1.3)`\n\
         docs/b.md:11:13: availability `server 2.0`\n\
         \n\
         2 places use release:server@2.0.\n"
    );
}

#[test]
fn refs_of_a_release_line_needs_a_release_target_and_a_version() {
    let dir = releases();
    // `cloud` is a target the project lists no releases for.
    let value = json(
        dir.path(),
        &["refs", "release:cloud@1.0", "--config", "site"],
        1,
    );
    assert_eq!(value["exists"], false);
    assert_eq!(value["total"], 0);
    for bad in [
        "release:server",
        "release:@2.0",
        "release:server@",
        "release:server@v2",
    ] {
        let out = run(dir.path(), &["refs", bad, "--config", "site"], 2);
        assert!(
            stderr(&out).contains("isn't a target"),
            "{bad}: {}",
            stderr(&out)
        );
    }
}

#[test]
fn refs_shows_the_spec_beside_each_availability() {
    let dir = releases();
    let value = json(
        dir.path(),
        &["refs", "dimension:deployment", "--config", "site"],
        0,
    );
    let specs: Vec<&str> = value["places"]
        .as_array()
        .expect("places")
        .iter()
        .filter(|p| p["use"] == "availability")
        .map(|p| p["spec"].as_str().expect("a spec"))
        .collect();
    assert!(specs.contains(&"cloud, server preview planned"), "{value}");
    assert!(specs.contains(&"deployment"), "{value}");
    let text = stdout(&run(
        dir.path(),
        &["refs", "dimension:deployment", "--config", "site"],
        0,
    ));
    assert!(
        text.contains("docs/b.md:8:13: availability `deployment`\n"),
        "{text}"
    );
    // Every other use has none.
    let other = json(dir.path(), &["refs", "site/docs/a.md"], 0);
    assert!(other["places"].as_array().expect("places").is_empty());
    let project = project();
    let phrase = json(
        project.path(),
        &["refs", "phrase:product", "--config", "site"],
        0,
    );
    assert!(phrase["places"][0]["spec"].is_null(), "{phrase}");
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
    let gone = json(dir.path(), &["refs", "site/docs/images/gone.png"], 1);
    assert_eq!(gone["kind"], "image", "{gone}");
    // An image has no headings to name.
    let err = stderr(&run(
        dir.path(),
        &["refs", "site/docs/images/keys.png#top"],
        2,
    ));
    assert!(
        err.contains("it's an image, which has no headings"),
        "{err}"
    );
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
fn resolve_writes_the_page_a_build_publishes() {
    let dir = project();
    let page = "site/docs/install.md";
    let text = stdout(&run(dir.path(), &["resolve", page, "--build", "cloud"], 0));
    assert!(text.contains("You need Quill."), "{text}");
    assert!(!text.contains("Upgrade"), "{text}");
    assert!(!text.starts_with("---"), "{text}");
    let other = stdout(&run(
        dir.path(),
        &["resolve", page, "--build", "self-managed", "--frontmatter"],
        0,
    ));
    assert!(
        other.starts_with("---\ntitle: Install Quill\n---\n"),
        "{other}"
    );
    assert!(other.contains("Upgrade Quill."), "{other}");
}

#[test]
fn resolve_says_a_reader_sees_one_arm_of_a_switcher() {
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
    let args = ["resolve", "docs/setup.md", "--build", "cloud"];
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
fn resolve_says_why_a_build_leaves_a_page_out() {
    let dir = project();
    let out = run(
        dir.path(),
        &["resolve", "site/docs/server.md", "--build", "cloud"],
        1,
    );
    assert!(stdout(&out).is_empty(), "{}", stdout(&out));
    assert!(stderr(&out).contains("cloud"), "{}", stderr(&out));
    let value = json(
        dir.path(),
        &["resolve", "site/docs/server.md", "--build", "cloud"],
        1,
    );
    assert!(value["not_published"].is_string());
    assert_eq!(value["text"], "");
}

#[test]
fn resolve_needs_a_build_and_a_page() {
    let dir = project();
    let out = run(dir.path(), &["resolve", "site/docs/keys.md"], 2);
    assert!(stderr(&out).contains("--build"), "{}", stderr(&out));
    run(
        dir.path(),
        &[
            "resolve",
            "site/docs/_fragments/before.md",
            "--build",
            "cloud",
        ],
        2,
    );
    run(
        dir.path(),
        &["resolve", "site/docs/keys.md", "--build", "nope"],
        2,
    );
}

#[test]
fn resolve_shows_a_permalinks_help() {
    let dir = tempfile::tempdir().unwrap();
    write(
        &dir.path().join("ascribe.toml"),
        &format!("{MODEL}\n[consumer]\nsite = \"https://docs.quill.dev\"\n"),
    );
    write(
        &dir.path().join("ascribe.addresses"),
        "version = 1\n\n[moved]\n\"go:key-rotation\" = \"go:rotate-keys\"\n\"go:old-keys\" = \"keys.md\"\n\n\
         [retired]\n\"go:legacy-sync\" = \"The sync agent was removed in 4.0\"\n",
    );
    write(
        &dir.path().join("docs/keys.md"),
        "---\ntitle: API keys\n---\n\n## Rotate keys\n@id {permalink=true}: rotate-keys\n\n@summary\nRotate a key every 90 days.\n\nRun `quill keys rotate`.\n",
    );
    let text = stdout(&run(
        dir.path(),
        &["resolve", "go:rotate-keys", "--build", "cloud"],
        0,
    ));
    assert_eq!(
        text,
        "Rotate keys\nRotate a key every 90 days.\n\nRun `quill keys rotate`.\n\nRead more: https://docs.quill.dev/go/rotate-keys/\n"
    );
    let value = json(
        dir.path(),
        &["resolve", "go:key-rotation", "--build", "cloud"],
        0,
    );
    assert_eq!(value["version"], 1, "{value}");
    assert_eq!(value["build"], "cloud", "{value}");
    assert!(value.get("line").is_none(), "{value}");
    assert_eq!(value["name"], "key-rotation", "{value}");
    assert_eq!(value["title"], "Rotate keys", "{value}");
    assert_eq!(value["summary"], "Rotate a key every 90 days.", "{value}");
    assert_eq!(
        value["address"], "https://docs.quill.dev/go/key-rotation/",
        "an alias has its target's entry, at its own address: {value}"
    );
    assert_eq!(value["source"], "keys.md", "{value}");

    let unknown = run(
        dir.path(),
        &[
            "resolve",
            "go:rotate-key",
            "--build",
            "cloud",
            "--format",
            "json",
        ],
        1,
    );
    assert!(stdout(&unknown).is_empty(), "{}", stdout(&unknown));
    assert_eq!(
        stderr(&unknown),
        "error: build `cloud` publishes no permalink `rotate-key`; did you mean `rotate-keys`?\n"
    );
    let retired = run(
        dir.path(),
        &["resolve", "go:legacy-sync", "--build", "cloud"],
        1,
    );
    assert!(stdout(&retired).is_empty(), "{}", stdout(&retired));
    assert_eq!(
        stderr(&retired),
        "error: permalink `legacy-sync` is retired: The sync agent was removed in 4.0\n"
    );
    let moved = run(
        dir.path(),
        &["resolve", "go:old-keys", "--build", "cloud"],
        1,
    );
    assert!(stdout(&moved).is_empty(), "{}", stdout(&moved));
    assert_eq!(
        stderr(&moved),
        "error: permalink `old-keys` was moved, and has no entry of its own: its address leads to `keys.md`\n"
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
fn search_names_three_of_the_pages_a_fragment_is_on_and_the_command_for_all() {
    let dir = project();
    let site = dir.path().join("site");
    for n in 1..=4 {
        write(
            &site.join(format!("docs/more-{n}.md")),
            &format!("---\ntitle: More {n}\n---\n\n@include: _fragments/before.md\n"),
        );
    }
    let out = run(
        dir.path(),
        &["search", "\"you need\"", "--config", "site"],
        0,
    );
    assert_eq!(
        stdout(&out),
        "docs/_fragments/before.md:3: You need Quill.\n    on 5 pages: install.md #before-you-start, more-1.md #before-you-start, more-2.md #before-you-start, and 2 more: `ascribe refs _fragments/before.md`\n\n1 place matches, on 5 pages.\n"
    );
}

#[test]
fn search_names_the_builds_of_a_page_that_shows_a_place_in_fewer() {
    let dir = project();
    let site = dir.path().join("site");
    write(
        &site.join("docs/both.md"),
        "---\ntitle: Both\n---\n\n@variant {deployment=self-managed}:\n@include: _fragments/before.md\n@variant {deployment=cloud}:\nOther.\n@end\n",
    );
    let out = run(
        dir.path(),
        &["search", "\"you need\"", "--config", "site"],
        0,
    );
    let text = stdout(&out);
    assert!(
        text.contains("on 2 pages: both.md #before-you-start (in build self-managed), install.md #before-you-start\n"),
        "{text}"
    );
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
fn search_cuts_a_long_code_block_to_its_match_at_the_line_it_is_on() {
    let dir = project();
    // A JSON Schema's worth of code, with the words far apart in it.
    let mut code: Vec<String> = (0..400)
        .map(|n| format!("  \"field{n}\": {{ \"type\": \"string\" }},"))
        .collect();
    code[40] = "  \"strict\": \"fail when there are problems\",".to_owned();
    code[390] = "  \"ci\": \"warnings in CI\",".to_owned();
    write(
        &dir.path().join("site/docs/schema.md"),
        &format!(
            "---\ntitle: Schema\n---\n\n```json\n{{\n{}\n}}\n```\n",
            code.join("\n")
        ),
    );
    let text = stdout(&run(
        dir.path(),
        &["search", "fail warnings CI", "--config", "site"],
        0,
    ));
    // The block opens at line 5 and its code at 6: `{` is 6, code[40] 47.
    let first = text.lines().next().unwrap_or_default();
    assert!(first.starts_with("docs/schema.md:47: …"), "{text}");
    assert!(first.contains("fail when"), "{text}");
    assert!(first.chars().count() < 120, "{text}");

    let value = json(
        dir.path(),
        &["search", "fail warnings CI", "--config", "site"],
        0,
    );
    let place = &value["places"][0];
    assert_eq!(place["line"], 47, "{place}");
    assert_eq!(place["in"], "code", "{place}");
    let excerpt: Vec<char> = place["text"].as_str().unwrap_or_default().chars().collect();
    assert!(excerpt.len() <= 242, "{} characters", excerpt.len());
    assert_eq!(excerpt.first(), Some(&'…'));
    assert_eq!(excerpt.last(), Some(&'…'));
    let at = usize::try_from(place["match"][0].as_u64().unwrap_or_default()).unwrap_or_default();
    let end = usize::try_from(place["match"][1].as_u64().unwrap_or_default()).unwrap_or_default();
    let matched: String = excerpt[at..].iter().take(4).collect();
    assert_eq!(matched, "fail", "{place}");
    assert_eq!(
        end,
        excerpt.len() - 1,
        "the match runs on past the cut: {place}"
    );
}

#[test]
fn search_exits_1_when_nothing_matches() {
    let dir = project();
    let out = run(dir.path(), &["search", "zebra", "--config", "site"], 1);
    assert_eq!(
        stdout(&out),
        "No place matches.\nThe words aren't in any page.\n"
    );
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
        stderr(&out).contains(
            "error: limit doesn't exist; to search for several words, quote them together: ascribe search 'rate limit'"
        ),
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
        &["search", "quill", "--type", "nope", "--config", "site"],
        &["search", "quill", "site/docs/nope.md"],
    ] {
        let out = run(dir.path(), args, 2);
        assert!(stderr(&out).starts_with("error: "), "{}", stderr(&out));
    }
}

/// The `file:line` of each place `ascribe search` lists, with its text.
fn places(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter(|line| !line.starts_with(' '))
        .filter_map(|line| {
            let (file, rest) = line.split_once(':')?;
            let (number, text) = rest.split_once(": ")?;
            number.parse::<u32>().ok()?;
            Some((format!("{file}:{number}"), text.to_owned()))
        })
        .collect()
}

/// The pages `ascribe search --summary` lists, from the project root.
fn summary_pages(text: &str) -> Vec<String> {
    text.lines()
        .skip_while(|line| *line != "By page:")
        .skip(1)
        .take_while(|line| line.starts_with("  ") && !line.starts_with("  and "))
        .map(|line| line.split_whitespace().nth(1).unwrap().to_owned())
        .collect()
}

#[test]
fn search_keeps_to_a_folder_and_a_fragment_to_its_pages_there() {
    let root = repository_root();
    let text = stdout(&run(
        &root,
        &[
            "search",
            "sidebar",
            "docs/content/navigation/",
            "--limit",
            "500",
        ],
        0,
    ));
    let lines: Vec<&str> = text.lines().collect();
    let mut found = 0;
    for (n, line) in lines.iter().enumerate() {
        let Some((file, _)) = places(line).into_iter().next() else {
            continue;
        };
        found += 1;
        if file.starts_with("content/navigation/") {
            continue;
        }
        // A fragment's or a snippet's line: shown only on pages there.
        let about = lines.get(n + 1).copied().unwrap_or_default();
        let (_, pages) = about
            .split_once(" page: ")
            .or_else(|| about.split_once(" pages: "))
            .unwrap_or_else(|| panic!("{line}\n{about}"));
        assert!(
            pages
                .split(", ")
                .all(|p| p.starts_with("navigation/") || p.starts_with("and ")),
            "{line}\n{about}"
        );
    }
    assert!(found > 0, "{text}");
    let summary = stdout(&run(
        &root,
        &[
            "search",
            "sidebar",
            "docs/content/navigation/",
            "--summary",
            "--limit",
            "500",
        ],
        0,
    ));
    let pages = summary_pages(&summary);
    assert!(!pages.is_empty(), "{summary}");
    assert!(
        pages.iter().all(|p| p.starts_with("content/navigation/")),
        "{summary}"
    );
    let all = summary_pages(&stdout(&run(
        &root,
        &[
            "search",
            "sidebar",
            "--config",
            "docs",
            "--summary",
            "--limit",
            "500",
        ],
        0,
    )));
    assert!(all.len() > pages.len());
}

#[test]
fn search_keeps_to_a_type() {
    let root = repository_root();
    let summary = stdout(&run(
        &root,
        &[
            "search",
            "sidebar",
            "--config",
            "docs",
            "--type",
            "how-to",
            "--summary",
            "--limit",
            "500",
        ],
        0,
    ));
    let pages = summary_pages(&summary);
    assert!(!pages.is_empty(), "{summary}");
    for page in pages {
        let text = fs::read_to_string(root.join("docs").join(&page)).unwrap();
        assert!(text.contains("\ntype: how-to\n"), "{page} isn't a how-to");
    }
}

#[test]
fn search_names_the_closest_type_and_says_to_quote_words() {
    let root = repository_root();
    let out = run(
        &root,
        &["search", "sidebar", "--config", "docs", "--type", "how-ot"],
        2,
    );
    assert_eq!(
        stderr(&out),
        "error: no content type is `how-ot`; did you mean how-to?\n"
    );
    let dir = project();
    let out = run(dir.path(), &["search", "quill", "site/ascribe.toml"], 2);
    assert_eq!(
        stderr(&out),
        "error: site/ascribe.toml has no page or fragment of the project\n"
    );
}

#[test]
fn search_with_word_does_not_find_a_longer_word() {
    let root = repository_root();
    let any = stdout(&run(
        &root,
        &["search", "limit", "--config", "docs", "--limit", "500"],
        0,
    ));
    let word = stdout(&run(
        &root,
        &[
            "search", "limit", "--config", "docs", "--word", "--limit", "500",
        ],
        0,
    ));
    let (any, word) = (places(&any), places(&word));
    assert!(!word.is_empty() && word.len() < any.len());
    let whole = |text: &str| {
        let lower = text.to_lowercase();
        lower.match_indices("limit").any(|(at, m)| {
            let before = lower[..at].chars().next_back();
            let after = lower[at + m.len()..].chars().next();
            !before.is_some_and(|c| c.is_alphanumeric() || c == '_')
                && !after.is_some_and(|c| c.is_alphanumeric() || c == '_')
        })
    };
    assert!(word.iter().all(|(_, text)| whole(text)), "{word:?}");
    assert!(
        any.iter()
            .any(|(_, text)| text.to_lowercase().contains("limits") && !whole(text)),
        "a place with only \"limits\" is left out"
    );
}

#[test]
fn search_with_case_does_not_find_another_case() {
    let root = repository_root();
    let any = places(&stdout(&run(
        &root,
        &["search", "vale", "--config", "docs", "--limit", "500"],
        0,
    )));
    let case = places(&stdout(&run(
        &root,
        &[
            "search", "Vale", "--config", "docs", "--case", "--limit", "500",
        ],
        0,
    )));
    assert!(!case.is_empty() && case.len() < any.len());
    assert!(
        case.iter().all(|(_, text)| text.contains("Vale")),
        "{case:?}"
    );
    assert!(
        any.iter().any(|(_, text)| !text.contains("Vale")),
        "a place with only \"vale\" is left out"
    );
}

#[test]
fn search_summarizes_by_page_most_first() {
    let root = repository_root();
    let text = stdout(&run(
        &root,
        &["search", "sidebar", "--config", "docs", "--summary"],
        0,
    ));
    assert!(text.starts_with("By page:\n"), "{text}");
    let counts: Vec<usize> = text
        .lines()
        .skip(1)
        .take_while(|line| line.starts_with("  ") && !line.starts_with("  and "))
        .map(|line| line.split_whitespace().next().unwrap().parse().unwrap())
        .collect();
    assert!(counts.windows(2).all(|w| w[0] >= w[1]), "{text}");
    assert!(
        text.contains("more pages: `ascribe search sidebar --summary --limit "),
        "{text}"
    );
    assert!(text.trim_end().ends_with(" pages."), "{text}");
}

#[test]
fn search_says_which_builds_not_searched_publish_the_words() {
    let quill = repository_root().join("examples/quill");
    let out = run(
        &quill,
        &[
            "search",
            "\"point the agent at your server\"",
            "--build",
            "cloud",
        ],
        1,
    );
    assert_eq!(
        stdout(&out),
        "No place matches.\nBuilds not searched publish them: `site`, `site@3.4`, `self-managed-3.3`.\n"
    );
}

#[test]
fn search_says_where_words_no_build_publishes_are_written() {
    let dir = project();
    let out = run(
        dir.path(),
        &["search", "\"{product}\"", "--config", "site"],
        1,
    );
    assert_eq!(
        stdout(&out),
        "No place matches.\nNo build searched publishes them. They're written at `docs/_fragments/before.md:3` and 2 more places.\n"
    );
}

#[test]
fn search_says_words_in_no_page_and_a_memo_is_in_none() {
    let dir = project();
    write(
        &dir.path().join("site/docs/keys.md"),
        &format!("{KEYS}\n@memo: Rotate before revoking.\n"),
    );
    for query in ["zebra", "revoking"] {
        let out = run(dir.path(), &["search", query, "--config", "site"], 1);
        assert_eq!(
            stdout(&out),
            "No place matches.\nThe words aren't in any page.\n"
        );
    }
}

#[test]
fn search_kept_to_paths_says_nothing_of_words_published_outside_them() {
    let dir = project();
    let out = run(
        dir.path(),
        &["search", "\"rotate them\"", "site/docs/install.md"],
        1,
    );
    assert_eq!(
        stdout(&out),
        "No place matches.\nThe words aren't in any page.\n"
    );
    run(
        dir.path(),
        &["search", "\"rotate them\"", "--config", "site"],
        0,
    );
}
