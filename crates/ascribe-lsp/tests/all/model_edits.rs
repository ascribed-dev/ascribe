//! The content model's actions: `ascribe/edit`'s `makePhrase`,
//! `addGlossaryTerm`, and `promoteFeature`. Each edits `ascribe.toml` in
//! place, keeping its comments and order, and the edits, applied, leave the
//! project with no diagnostic it didn't have, in the fixtures and the example
//! projects.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use crate::support;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::support::{Client, Fixture};
use ascribe_check::{Project, SourceFile, check_project};
use ascribe_core::{FileId, LineIndex, WideEncoding, WideLineCol};
use serde_json::{Value, json};

const MODEL: &str = r#"# The model.
spec = "0.1"

[project]
content-root = "docs"

[dimensions.deployment]
values = ["cloud", "self-managed"]
versionless = ["cloud"]

# Phrases.
[phrases]
product = "Quill" # the name

[features.sso]
name = "Single sign-on"
available = "cloud beta" # for now

[glossary.terms.api]
term = "API"
definition = "An interface."

[editor]
build = "site"
"#;

fn fixture() -> Fixture {
    Fixture::new(
        MODEL,
        &[
            (
                "docs/index.md",
                "---\ntitle: Home\n---\n# Home\n\nThe Quill Cloud console. Quill Cloud is fast.\n\n## Quill Cloud\n\n## Quill Cloud setup\n@id: setup\n\n- Use Quill Cloud\n\n`Quill Cloud` and [Quill Cloud](other.md) and \\Quill Cloud and Quill Cloudy.\n\n```\nQuill Cloud\n```\n",
            ),
            (
                "docs/other.md",
                "---\ntitle: Other\n---\n# Other\n\nSign in to Quill Cloud.\n",
            ),
        ],
    )
}

/// The text without its markers (`‸` or `«…»`), and the range they mark.
fn unmark(marked: &str) -> (String, usize, usize) {
    if let Some(at) = marked.find('‸') {
        return (marked.replacen('‸', "", 1), at, at);
    }
    let start = marked.find('«').expect("a selection");
    let text = marked.replacen('«', "", 1);
    let end = text.find('»').expect("its end");
    (text.replacen('»', "", 1), start, end)
}

fn position(text: &str, offset: usize) -> Value {
    let p = LineIndex::new(text)
        .wide_line_col(WideEncoding::Utf16, offset)
        .expect("a position");
    json!({ "line": p.line, "character": p.col })
}

fn offset(text: &str, position: &Value) -> usize {
    LineIndex::new(text)
        .wide_offset(
            WideEncoding::Utf16,
            WideLineCol {
                line: position["line"].as_u64().unwrap() as u32,
                col: position["character"].as_u64().unwrap() as u32,
            },
        )
        .expect("an offset")
}

/// `text` with LSP text edits applied.
fn apply(text: &str, edits: &[Value]) -> String {
    let edits: Vec<ascribe_core::TextEdit> = edits
        .iter()
        .map(|e| {
            ascribe_core::TextEdit::replace(
                ascribe_core::Span::new(
                    offset(text, &e["range"]["start"]),
                    offset(text, &e["range"]["end"]),
                ),
                e["newText"].as_str().expect("text"),
            )
        })
        .collect();
    ascribe_core::apply_edits(text, &edits).expect("edits that apply")
}

/// A workspace edit's text edits, by file path.
fn changes(edit: &Value) -> BTreeMap<PathBuf, Vec<Value>> {
    edit["changes"]
        .as_object()
        .expect("changes")
        .iter()
        .map(|(uri, edits)| {
            let url = uri.strip_prefix("file://").expect("a file URI");
            let decoded = percent_decode(url);
            // `/C:/dir` on Windows.
            let path = match decoded.strip_prefix('/') {
                Some(rest) if rest.as_bytes().get(1) == Some(&b':') => rest.to_owned(),
                _ => decoded,
            };
            (
                PathBuf::from(path),
                edits.as_array().expect("edits").clone(),
            )
        })
        .collect()
}

fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).expect("hex");
            out.push(u8::from_str_radix(hex, 16).expect("hex"));
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).expect("UTF-8")
}

fn same(a: &Path, b: &Path) -> bool {
    let norm = |p: &Path| p.to_string_lossy().replace('\\', "/").to_lowercase();
    norm(a) == norm(b)
}

/// What an action did: each changed file's text after it, and its edits.
struct Done {
    files: Vec<(PathBuf, String, Vec<Value>)>,
}

impl Done {
    /// The text after the action of a file it changed.
    fn after(&self, path: &Path) -> &str {
        self.files
            .iter()
            .find(|(p, _, _)| same(p, path))
            .map(|(_, text, _)| text.as_str())
            .unwrap_or_else(|| panic!("{} didn't change", path.display()))
    }

    fn changed(&self, path: &Path) -> bool {
        self.files.iter().any(|(p, _, _)| same(p, path))
    }

    fn edits(&self, path: &Path) -> &[Value] {
        self.files
            .iter()
            .find(|(p, _, _)| same(p, path))
            .map(|(_, _, e)| e.as_slice())
            .unwrap_or_default()
    }
}

/// Applies a workspace edit to the files it names, reading each from
/// `texts` or else from the disk.
fn apply_workspace_edit(edit: &Value, texts: &BTreeMap<PathBuf, String>) -> Done {
    let files = changes(edit)
        .into_iter()
        .map(|(path, edits)| {
            let text = texts
                .iter()
                .find(|(p, _)| same(p, &path))
                .map(|(_, t)| t.clone())
                .unwrap_or_else(|| std::fs::read_to_string(&path).expect("the file"));
            let after = apply(&text, &edits);
            (path, after, edits)
        })
        .collect();
    Done { files }
}

/// Runs `action` on `marked`, the text of the open page at `path`.
fn run(
    client: &mut Client,
    path: &Path,
    marked: &str,
    action: &str,
    args: Value,
) -> Result<Done, String> {
    let (text, start, end) = unmark(marked);
    client.open(path, 3, &text);
    let result = client
        .request(
            "ascribe/edit",
            json!({
                "textDocument": { "uri": support::uri(path).as_str() },
                "range": { "start": position(&text, start), "end": position(&text, end) },
                "action": action,
                "args": args,
                "version": 3,
            }),
        )
        .response_result
        .expect("the request succeeds");
    client.close(path);
    if let Some(error) = result.get("error") {
        return Err(error.as_str().expect("a message").to_owned());
    }
    let mut texts = BTreeMap::new();
    texts.insert(path.to_path_buf(), text);
    Ok(apply_workspace_edit(&result["edit"], &texts))
}

/// Whether each edit to `ascribe.toml` touches nothing but what it adds or
/// changes: an insertion, or a replacement no longer than `longest`.
fn assert_minimal(edits: &[Value], model: &str, longest: usize) {
    for e in edits {
        let a = offset(model, &e["range"]["start"]);
        let b = offset(model, &e["range"]["end"]);
        assert!(b - a <= longest, "the edit covers {:?}", &model[a..b]);
    }
}

#[test]
fn the_selection_becomes_a_phrase_declared_in_place() {
    let f = fixture();
    let mut client = Client::start(&f.root());
    let page = f.path("docs/index.md");
    let text = std::fs::read_to_string(&page).unwrap();
    let marked = text.replacen("The Quill Cloud", "The «Quill Cloud»", 1);
    let done = run(
        &mut client,
        &page,
        &marked,
        "makePhrase",
        json!({ "key": "cloud" }),
    )
    .unwrap();
    let model = done.after(&f.path("ascribe.toml"));
    assert_eq!(
        model,
        MODEL.replace(
            "product = \"Quill\" # the name\n",
            "product = \"Quill\" # the name\ncloud = \"Quill Cloud\"\n"
        )
    );
    assert_minimal(done.edits(&f.path("ascribe.toml")), MODEL, 0);
    let after = done.after(&page);
    assert!(
        after.contains("The {cloud} console. Quill Cloud is fast."),
        "{after}"
    );
    assert!(
        !done.changed(&f.path("docs/other.md")),
        "only the selection"
    );
}

#[test]
fn a_phrase_can_replace_every_occurrence_in_prose() {
    let f = fixture();
    let mut client = Client::start(&f.root());
    let page = f.path("docs/index.md");
    let text = std::fs::read_to_string(&page).unwrap();
    let marked = text.replacen("The Quill Cloud", "The «Quill Cloud»", 1);
    // The other occurrences `ascribe/targets` lists are those it replaces.
    let (plain, start, end) = unmark(&marked);
    client.open(&page, 3, &plain);
    let targets = client
        .request(
            "ascribe/targets",
            json!({
                "textDocument": { "uri": support::uri(&page).as_str() },
                "kinds": ["occurrences"],
                "range": { "start": position(&plain, start), "end": position(&plain, end) },
            }),
        )
        .response_result
        .expect("targets");
    client.close(&page);
    let listed = targets["occurrences"].as_array().expect("a list");
    assert_eq!(
        listed
            .iter()
            .map(|o| o["path"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["index.md", "index.md", "index.md", "index.md", "other.md"]
    );

    let done = run(
        &mut client,
        &page,
        &marked,
        "makePhrase",
        json!({ "key": "cloud", "everywhere": true }),
    )
    .unwrap();
    assert_eq!(
        done.after(&page),
        "---\ntitle: Home\n---\n# Home\n\nThe {cloud} console. {cloud} is fast.\n\n## Quill Cloud\n\n## {cloud} setup\n@id: setup\n\n- Use {cloud}\n\n`Quill Cloud` and [{cloud}](other.md) and \\Quill Cloud and Quill Cloudy.\n\n```\nQuill Cloud\n```\n",
        "not in code, a heading without an @id, after a backslash, or inside a word"
    );
    assert_eq!(
        done.after(&f.path("docs/other.md")),
        "---\ntitle: Other\n---\n# Other\n\nSign in to {cloud}.\n"
    );
}

#[test]
fn a_phrase_leaves_link_labels_and_urls_alone() {
    let page_text = "---\ntitle: Links\n---\n# Links\n\nQuill is here. See [Quill] and [Quill][] and [the Quill docs][ref], <https://quill.dev/Quill>, and https://example.org/Quill/x.\n\n[quill]: https://quill.dev/\n[ref]: https://quill.dev/docs\n";
    let f = Fixture::new(MODEL, &[("docs/links.md", page_text)]);
    let mut client = Client::start(&f.root());
    let page = f.path("docs/links.md");
    let marked = page_text.replacen("Quill is", "«Quill» is", 1);
    let done = run(
        &mut client,
        &page,
        &marked,
        "makePhrase",
        json!({ "key": "q", "everywhere": true }),
    )
    .unwrap();
    assert_eq!(
        done.after(&page),
        page_text
            .replacen("Quill is", "{q} is", 1)
            .replacen("the Quill docs", "the {q} docs", 1),
        "a reference link's label and a URL stay as they are"
    );
}

#[test]
fn a_phrase_needs_a_good_key_and_plain_text() {
    let f = fixture();
    let mut client = Client::start(&f.root());
    let page = f.path("docs/c.md");
    let mut error = |marked: &str, key: &str| {
        run(
            &mut client,
            &page,
            marked,
            "makePhrase",
            json!({ "key": key }),
        )
        .err()
        .expect("refused")
    };
    let e = error("# C\n\nSay «hello».\n", "Hello");
    assert!(e.contains("can't be a phrase key"), "{e}");
    let e = error("# C\n\nSay «hello».\n", "product");
    assert!(e.contains("already a phrase"), "{e}");
    let e = error("# C\n\nSay «*hello* there».\n", "greeting");
    assert!(e.contains("plain text"), "{e}");
    let e = error("# C\n\nSay ‸hello.\n", "greeting");
    assert!(e.contains("Select"), "{e}");
}

#[test]
fn a_glossary_term_is_a_table_after_the_last_one() {
    let f = fixture();
    let mut client = Client::start(&f.root());
    let page = f.path("docs/other.md");
    let text = std::fs::read_to_string(&page).unwrap();
    let marked = text.replacen("Quill Cloud", "«Quill Cloud»", 1);
    let done = run(
        &mut client,
        &page,
        &marked,
        "addGlossaryTerm",
        json!({
            "id": "cloud",
            "term": "Quill Cloud",
            "aliases": ["the cloud"],
            "definition": "The hosted service.",
            "link": "index.md#setup",
        }),
    )
    .unwrap();
    assert!(!done.changed(&page), "the page stays as it is");
    assert_eq!(
        done.after(&f.path("ascribe.toml")),
        MODEL.replace(
            "definition = \"An interface.\"\n",
            "definition = \"An interface.\"\n\n[glossary.terms.cloud]\nterm = \"Quill Cloud\"\naliases = [\"the cloud\"]\ndefinition = \"The hosted service.\"\nlink = \"index.md#setup\"\n"
        )
    );
    let mut error = |args: Value| match run(
        &mut client,
        &page,
        "# Other\n‸",
        "addGlossaryTerm",
        args.clone(),
    ) {
        Ok(done) => panic!(
            "{args} is accepted: {}",
            done.after(&f.path("ascribe.toml"))
        ),
        Err(e) => e,
    };
    let e = error(json!({ "id": "api", "term": "Interface", "definition": "x" }));
    assert!(e.contains("already has a term"), "{e}");
    let e = error(json!({ "id": "app", "term": "api", "definition": "x" }));
    assert!(
        e.contains("ascribe.toml"),
        "a duplicate term is the model's problem: {e}"
    );
    let e = error(json!({ "id": "app", "term": "App", "definition": "x", "link": "missing.md" }));
    assert!(e.contains("ascribe.toml"), "{e}");
    let e =
        error(json!({ "id": "app", "term": "App", "definition": "x", "link": "index.md#nowhere" }));
    assert!(e.contains("no heading with the id `nowhere`"), "{e}");
}

#[test]
fn a_features_availability_changes_in_place() {
    let f = fixture();
    let mut client = Client::start(&f.root());
    let page = f.path("docs/other.md");
    let done = run(
        &mut client,
        &page,
        "# Other\n‸",
        "promoteFeature",
        json!({ "key": "sso", "spec": "cloud, self-managed 2.0" }),
    )
    .unwrap();
    let model = done.after(&f.path("ascribe.toml"));
    assert_eq!(
        model,
        MODEL.replace(
            "available = \"cloud beta\" # for now",
            "available = \"cloud, self-managed 2.0\" # for now"
        )
    );
    assert_minimal(
        done.edits(&f.path("ascribe.toml")),
        MODEL,
        "\"cloud beta\"".len(),
    );
    let mut error = |args: Value| {
        run(&mut client, &page, "# Other\n‸", "promoteFeature", args)
            .err()
            .expect("refused")
    };
    let e = error(json!({ "key": "nope", "spec": "cloud" }));
    assert!(e.contains("isn't a feature"), "{e}");
    let e = error(json!({ "key": "sso", "spec": "cloud beta" }));
    assert!(e.contains("already"), "{e}");
    let e = error(json!({ "key": "sso", "spec": "cloud (" }));
    assert!(e.contains("isn't an availability spec"), "{e}");
    let e = error(json!({ "key": "sso", "spec": "mars" }));
    assert!(e.contains("ascribe.toml"), "an unknown target: {e}");
}

#[test]
fn an_unsaved_model_is_edited_as_it_is_in_the_editor() {
    let f = fixture();
    let mut client = Client::start(&f.root());
    let model_path = f.path("ascribe.toml");
    let unsaved = MODEL.replace("[phrases]\n", "[phrases]\nversion = \"3.4\"\n");
    client.open(&model_path, 1, &unsaved);
    client.settle();
    let page = f.path("docs/other.md");
    let (text, start, end) = unmark("# Other\n\nSign in to «Quill Cloud».\n");
    client.open(&page, 3, &text);
    let result = client
        .request(
            "ascribe/edit",
            json!({
                "textDocument": { "uri": support::uri(&page).as_str() },
                "range": { "start": position(&text, start), "end": position(&text, end) },
                "action": "makePhrase",
                "args": { "key": "cloud" },
            }),
        )
        .response_result
        .unwrap();
    let mut texts = BTreeMap::new();
    texts.insert(model_path.clone(), unsaved.clone());
    texts.insert(page.clone(), text);
    let done = apply_workspace_edit(&result["edit"], &texts);
    assert_eq!(
        done.after(&model_path),
        unsaved.replace(
            "product = \"Quill\" # the name\n",
            "product = \"Quill\" # the name\ncloud = \"Quill Cloud\"\n"
        )
    );

    // A model that doesn't load isn't edited.
    client.replace(&model_path, 2, "spec = \"0.1\"\n[phrases\n");
    client.settle();
    let result = client
        .request(
            "ascribe/edit",
            json!({
                "textDocument": { "uri": support::uri(&page).as_str() },
                "range": { "start": position(&texts[&page], start), "end": position(&texts[&page], end) },
                "action": "makePhrase",
                "args": { "key": "cloud" },
            }),
        )
        .response_result
        .unwrap();
    assert!(
        result["error"].as_str().unwrap().contains("has a problem"),
        "{result}"
    );
}

// -- The example projects --------------------------------------------------

fn examples() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

/// The diagnostics of a project, by file and code.
fn diagnostics(project: &Project) -> BTreeMap<(String, String), usize> {
    let build = project.model().editor_default_build().clone();
    let mut out = BTreeMap::new();
    for d in check_project(project, &build) {
        // Making a phrase of a glossary term's only occurrence leaves the
        // term in the phrase's value, which the search for uses doesn't
        // read, so the term reads as unused.
        if d.slug == ascribe_core::diagnostics::GLOSSARY_TERM_UNUSED {
            continue;
        }
        let file = project
            .file(d.location.file)
            .map(|f| f.display_path.to_owned())
            .unwrap_or_default();
        *out.entry((file, d.code.to_owned())).or_default() += 1;
    }
    out
}

/// Asserts that `done`, applied to the example at `root`, loads and adds no
/// diagnostic.
fn assert_no_new_diagnostics(root: &Path, done: &Done, what: &str) {
    let project = Project::load(&root.join("ascribe.toml")).expect("the example loads");
    let baseline = diagnostics(&project);
    let model_path = root.join("ascribe.toml");
    let model_text = done
        .files
        .iter()
        .find(|(p, _, _)| same(p, &model_path))
        .map_or_else(|| project.model_text().to_owned(), |(_, t, _)| t.clone());
    let model = ascribe_model::load_str_in(&model_text, FileId::new(0), root)
        .unwrap_or_else(|e| panic!("{what}: the model doesn't load: {e:?}"));
    let content = root.join(project.content_root().as_str());
    let sources: Vec<SourceFile> = project
        .sources()
        .iter()
        .map(|s| {
            let path = content.join(s.path.as_str());
            let text = done
                .files
                .iter()
                .find(|(p, _, _)| same(p, &path))
                .map_or_else(|| s.text.clone(), |(_, t, _)| t.clone());
            SourceFile { text, ..s.clone() }
        })
        .collect();
    let edited = Project::from_parts(
        root.to_path_buf(),
        project.content_root().clone(),
        model,
        model_text,
        sources,
    );
    for (key, count) in diagnostics(&edited) {
        let before = baseline.get(&key).copied().unwrap_or_default();
        assert!(count <= before, "{what} adds {key:?}");
    }
}

/// Runs an action on a page of an example, as it is on disk, and checks
/// what it does.
fn on_example(root: &Path, page: &str, find: &str, action: &str, args: Value) -> Done {
    let root = support::real_path(root);
    let mut client = Client::start(&root);
    let path = root.join(page);
    let text = std::fs::read_to_string(&path).unwrap();
    let marked = if find.is_empty() {
        format!("‸{text}")
    } else {
        text.replacen(find, &format!("«{find}»"), 1)
    };
    let done = run(&mut client, &path, &marked, action, args)
        .unwrap_or_else(|e| panic!("{action} on {page}: {e}"));
    assert_no_new_diagnostics(&root, &done, &format!("{action} on {page}"));
    done
}

#[test]
fn content_model_actions_on_quill_add_no_diagnostics() {
    let root = examples().join("quill");
    let done = on_example(
        &root,
        "docs/install-agent.md",
        "API key",
        "makePhrase",
        json!({ "key": "api-key", "everywhere": true }),
    );
    let page = support::real_path(&root).join("docs/install-agent.md");
    assert_eq!(done.after(&page).matches("{api-key}").count(), 2);
    on_example(
        &root,
        "docs/quickstart.md",
        "agent",
        "addGlossaryTerm",
        json!({ "id": "agent", "term": "agent", "aliases": ["agents"], "definition": "The program that syncs.", "link": "/install-agent.md" }),
    );
}

#[test]
fn content_model_actions_on_the_monorepo_docs_add_no_diagnostics() {
    let root = examples().join("monorepo/docs");
    on_example(
        &root,
        "content/getting-started.md",
        "feature flag",
        "makePhrase",
        json!({ "key": "flag", "everywhere": true }),
    );
    on_example(
        &root,
        "content/getting-started.md",
        "",
        "addGlossaryTerm",
        json!({ "id": "sdk", "term": "SDK", "definition": "A library for an app.", "link": "/reference/glossary.md" }),
    );
    let done = on_example(
        &root,
        "content/getting-started.md",
        "",
        "promoteFeature",
        json!({ "key": "audit-log", "spec": "cloud, self-hosted 2.5" }),
    );
    let model = std::fs::read_to_string(root.join("ascribe.toml")).unwrap();
    assert_eq!(
        done.after(&support::real_path(&root).join("ascribe.toml")),
        model.replace(
            "available = \"cloud, self-hosted beta 2.4\"",
            "available = \"cloud, self-hosted 2.5\""
        )
    );
}

// -- Renames ------------------------------------------------------------------

const RENAME_MODEL: &str = r#"spec = "0.1"

[project]
content-root = "docs"

[types.page]
default = true

[types.page.frontmatter]
title = { type = "string", phrases = true }

[dimensions.deployment]
values = ["cloud", # hosted
  "self-managed"]
labels = { self-managed = "Self-managed" }
versionless = ["cloud"]

[phrases]
product = "Quill"
api = "https://api.quill.dev/"

[features.sso]
name = "SSO"
available = "cloud, self-managed beta 2.4"

[sources.code]
path = "code"

[builds.site]

[builds.sm]
variants = { deployment = "self-managed" }
availability = { filter = "self-managed 2.4" }

[builds.both]
variants.deployment = ["cloud", "self-managed"]

[editor]
build = "site"
"#;

const RENAME_PAGE: &str = "---\ntitle: Use {product}\nvariant:\n  deployment: self-managed\navailable: self-managed 2.4\n---\n# Home\n\nRun {product} from [{product} docs]({api}guide).\n\n```shell phrases=true\n{product} run\n```\n\n@snippet {phrases=true}: code:app.sh\n\n@variant {deployment=cloud|self-managed}:\nBoth.\n@variant {deployment=cloud}:\nCloud.\n@end\n\n@available: cloud, self-managed beta 2.4\nPara.\n\n| A | B |\n|---|---|\n| x {available=\"self-managed 2.4\"} | y |\n";

fn rename_fixture() -> Fixture {
    Fixture::new(
        RENAME_MODEL,
        &[
            ("docs/index.md", RENAME_PAGE),
            ("code/app.sh", "#!/bin/sh\necho {product}\n"),
        ],
    )
}

/// The offset of the `n`th (from 0) occurrence of `what` in `text`, plus
/// `into`.
fn at(text: &str, what: &str, n: usize, into: usize) -> usize {
    text.match_indices(what).nth(n).expect("found").0 + into
}

fn prepare(
    client: &mut Client,
    path: &Path,
    text: &str,
    offset: usize,
) -> Result<(String, String), String> {
    let response = client.request(
        "textDocument/prepareRename",
        json!({
            "textDocument": { "uri": support::uri(path).as_str() },
            "position": position(text, offset),
        }),
    );
    let result = response.response_result.map_err(|error| error.message)?;
    let a = offset_of(text, &result["range"]["start"]);
    let b = offset_of(text, &result["range"]["end"]);
    Ok((
        text[a..b].to_owned(),
        result["placeholder"]
            .as_str()
            .expect("a placeholder")
            .to_owned(),
    ))
}

fn offset_of(text: &str, position: &Value) -> usize {
    offset(text, position)
}

fn rename(client: &mut Client, path: &Path, text: &str, offset: usize, new: &str) -> Option<Done> {
    let result = client
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": support::uri(path).as_str() },
                "position": position(text, offset),
                "newName": new,
            }),
        )
        .response_result
        .expect("an answer");
    if result.is_null() {
        return None;
    }
    let mut texts = BTreeMap::new();
    texts.insert(path.to_path_buf(), text.to_owned());
    Some(apply_workspace_edit(&result, &texts))
}

#[test]
fn prepare_rename_says_what_is_renamed_or_why_nothing_is() {
    let f = rename_fixture();
    let mut client = Client::start(&f.root());
    let page = f.path("docs/index.md");
    let text = RENAME_PAGE;
    client.open(&page, 1, text);
    let mut ask = |offset| prepare(&mut client, &page, text, offset);
    assert_eq!(
        ask(at(text, "{product}", 1, 3)),
        Ok(("product".to_owned(), "product".to_owned()))
    );
    assert_eq!(
        ask(at(text, "self-managed}", 0, 2)),
        Ok(("self-managed".to_owned(), "self-managed".to_owned()))
    );
    assert_eq!(
        ask(at(text, "# Home", 0, 4)),
        Ok(("Home".to_owned(), "Home".to_owned()))
    );
    let e = ask(at(text, "Para.", 0, 1)).expect_err("nothing to rename");
    assert!(e.contains("Put the cursor on a phrase"), "{e}");

    let model = f.path("ascribe.toml");
    client.open(&model, 1, RENAME_MODEL);
    let mut ask = |offset| prepare(&mut client, &model, RENAME_MODEL, offset);
    assert_eq!(
        ask(at(RENAME_MODEL, "product =", 0, 2)),
        Ok(("product".to_owned(), "product".to_owned()))
    );
    assert_eq!(
        ask(at(RENAME_MODEL, "\"self-managed\"]", 0, 3)),
        Ok(("self-managed".to_owned(), "self-managed".to_owned()))
    );
    assert!(ask(at(RENAME_MODEL, "spec", 0, 1)).is_err());
}

#[test]
fn a_phrase_rename_reaches_every_kind_of_use() {
    let f = rename_fixture();
    let mut client = Client::start(&f.root());
    let page = f.path("docs/index.md");
    client.open(&page, 1, RENAME_PAGE);
    let done = rename(
        &mut client,
        &page,
        RENAME_PAGE,
        at(RENAME_PAGE, "{product}", 1, 2),
        "name",
    )
    .expect("renamed");
    let after = done.after(&page);
    assert_eq!(after.matches("{name}").count(), 4, "{after}");
    assert!(!after.contains("{product}"), "{after}");
    assert!(
        after.contains("title: Use {name}\n"),
        "the frontmatter: {after}"
    );
    assert!(
        after.contains("({api}guide)"),
        "other phrases stay: {after}"
    );
    assert_eq!(
        done.after(&f.path("code/app.sh")),
        "#!/bin/sh\necho {name}\n"
    );
    assert_eq!(
        done.after(&f.path("ascribe.toml")),
        RENAME_MODEL.replace("product = \"Quill\"", "name = \"Quill\"")
    );
}

#[test]
fn renames_leave_frontmatter_comments_and_read_block_scalars() {
    let page_text = "---\ntitle: |\n  Note: {product} # {product}\nvariant:\n  deployment: [self-managed] # self-managed\navailable: self-managed 2.4 # self-managed\n---\n# Home\n";
    let f = Fixture::new(
        RENAME_MODEL,
        &[("docs/index.md", page_text), ("code/app.sh", "echo\n")],
    );
    let mut client = Client::start(&f.root());
    let model = f.path("ascribe.toml");
    let page = f.path("docs/index.md");
    client.open(&page, 1, page_text);
    client.open(&model, 1, RENAME_MODEL);
    let phrase = rename(
        &mut client,
        &model,
        RENAME_MODEL,
        at(RENAME_MODEL, "product =", 0, 1),
        "name",
    )
    .expect("renamed");
    assert_eq!(
        phrase.after(&page),
        page_text.replace("{product}", "{name}"),
        "a block scalar's lines are the field's text, `#` and all"
    );
    let value = rename(
        &mut client,
        &model,
        RENAME_MODEL,
        at(RENAME_MODEL, "\"self-managed\"]", 0, 4),
        "on-prem",
    )
    .expect("renamed");
    assert_eq!(
        value.after(&page),
        page_text
            .replacen("[self-managed]", "[on-prem]", 1)
            .replacen("available: self-managed", "available: on-prem", 1),
        "comments stay as they are"
    );
}

#[test]
fn a_dimension_value_is_renamed_everywhere_from_a_page_or_the_model() {
    let f = rename_fixture();
    let mut client = Client::start(&f.root());
    let page = f.path("docs/index.md");
    client.open(&page, 1, RENAME_PAGE);
    let expected_page = RENAME_PAGE.replace("self-managed", "on-prem");
    let expected_model = RENAME_MODEL
        .replace("\"self-managed\"", "\"on-prem\"")
        .replace(
            "self-managed = \"Self-managed\"",
            "on-prem = \"Self-managed\"",
        )
        .replace("cloud, self-managed beta 2.4", "cloud, on-prem beta 2.4")
        .replace("self-managed 2.4", "on-prem 2.4");
    let from_page = rename(
        &mut client,
        &page,
        RENAME_PAGE,
        at(RENAME_PAGE, "self-managed}", 0, 3),
        "on-prem",
    )
    .expect("renamed");
    assert_eq!(from_page.after(&page), expected_page);
    assert_eq!(from_page.after(&f.path("ascribe.toml")), expected_model);
    assert_minimal(
        from_page.edits(&f.path("ascribe.toml")),
        RENAME_MODEL,
        "\"cloud, self-managed beta 2.4\"".len(),
    );

    let model = f.path("ascribe.toml");
    client.open(&model, 1, RENAME_MODEL);
    let from_model = rename(
        &mut client,
        &model,
        RENAME_MODEL,
        at(RENAME_MODEL, "\"self-managed\"]", 0, 4),
        "on-prem",
    )
    .expect("renamed");
    assert_eq!(from_model.after(&page), expected_page);
    assert_eq!(from_model.after(&model), expected_model);

    // A name the model doesn't allow, or that's taken, renames nothing.
    for taken in [
        "cloud",
        "beta",
        "deployment",
        "sso",
        "on prem",
        "self-managed",
    ] {
        assert!(
            rename(
                &mut client,
                &page,
                RENAME_PAGE,
                at(RENAME_PAGE, "self-managed}", 0, 3),
                taken
            )
            .is_none(),
            "{taken}"
        );
    }
}

/// Renames at `find` in `page` of an example, and checks that nothing new
/// is wrong after.
fn rename_in_example(root: &Path, page: &str, find: &str, into: usize, new: &str) -> Done {
    let root = support::real_path(root);
    let mut client = Client::start(&root);
    let path = root.join(page);
    let text = std::fs::read_to_string(&path).unwrap();
    client.open(&path, 1, &text);
    client.settle();
    let done = rename(&mut client, &path, &text, at(&text, find, 0, into), new)
        .unwrap_or_else(|| panic!("no rename at {find} in {page}"));
    assert_no_new_diagnostics(&root, &done, &format!("renaming {find} in {page}"));
    done
}

#[test]
fn renames_in_quill_add_no_diagnostics() {
    let root = examples().join("quill");
    let done = rename_in_example(&root, "docs/install-agent.md", "{cloud}", 1, "hosted");
    assert!(done.files.len() >= 2);
    let done = rename_in_example(
        &root,
        "docs/install-agent.md",
        "deployment=self-managed",
        "deployment=".len(),
        "on-prem",
    );
    let model = std::fs::read_to_string(root.join("ascribe.toml")).unwrap();
    assert!(
        done.after(&support::real_path(&root).join("ascribe.toml"))
            .contains("filter = \"on-prem 3.3\""),
        "the build's filter"
    );
    assert!(model.contains("filter = \"self-managed 3.3\""));
}

#[test]
fn renames_in_the_monorepo_docs_add_no_diagnostics() {
    let root = examples().join("monorepo/docs");
    let done = rename_in_example(&root, "content/getting-started.md", "{product}", 1, "name");
    assert!(
        done.after(&support::real_path(&root).join("content/getting-started.md"))
            .contains("title: Get started with {name}"),
        "the title takes phrases"
    );
    rename_in_example(
        &root,
        "content/getting-started.md",
        "edition=self-hosted",
        "edition=".len(),
        "on-prem",
    );
}
