//! Every operation of `ascribe/edit`, with valid arguments, at every line and
//! inline node of the example projects' pages: each edit the server returns
//! is canonical where it wrote, and adds no diagnostic to the project.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ascribe_check::{Project, SourceFile, check_project};
use ascribe_core::{LineIndex, Span, WideEncoding, WideLineCol};
use ascribe_syntax::{Block, BlockKind};
use serde_json::{Value, json};
use support::Client;

fn examples() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

#[test]
fn edits_on_the_quill_pages_are_canonical_and_add_no_diagnostics() {
    every_edit(&examples().join("quill"));
}

#[test]
fn edits_on_the_monorepo_docs_pages_are_canonical_and_add_no_diagnostics() {
    every_edit(&examples().join("monorepo/docs"));
}

#[test]
fn edits_on_the_monorepo_handbook_pages_are_canonical_and_add_no_diagnostics() {
    every_edit(&examples().join("monorepo/handbook"));
}

#[test]
fn edits_on_the_monorepo_security_pages_are_canonical_and_add_no_diagnostics() {
    every_edit(&examples().join("monorepo/handbook/pages/security"));
}

/// The diagnostics of a project, by file and code.
fn diagnostics(project: &Project) -> BTreeMap<(String, String), usize> {
    let build = project.model().editor_default_build().clone();
    let mut out = BTreeMap::new();
    for d in check_project(project, &build) {
        let file = project
            .file(d.location.file)
            .map(|f| f.display_path.to_owned())
            .unwrap_or_default();
        *out.entry((file, d.code.to_owned())).or_default() += 1;
    }
    out
}

fn every_edit(root: &Path) {
    let root = support::real_path(root);
    let project = Project::load(&root.join("ascribe.toml")).expect("the example loads");
    let baseline = diagnostics(&project);
    let model = project.model().clone();
    let options = ascribe_fmt::options_from_model(&model);
    let mut client = Client::start(&root);
    let content = root.join(project.content_root().as_str());
    let mut checked = BTreeSet::new();
    let mut edits = 0;
    for source in project.sources() {
        let path = content.join(source.path.as_str());
        let uri = support::uri(&path);
        let text = &source.text;
        let targets = client
            .request(
                "ascribe/targets",
                json!({
                    "textDocument": { "uri": uri.as_str() },
                    "kinds": ["pages", "fragments", "images", "snippets", "phrases", "dimensions", "widgets", "features"],
                }),
            )
            .response_result
            .expect("targets");
        let doc = ascribe_syntax::parse(text, &options);
        for (start, end) in ranges(text, &doc.blocks) {
            for (action, args) in arguments(&targets, &source.path.to_string()) {
                // The page's own settings don't depend on where the cursor is.
                if action.starts_with("setPage") && start > 0 {
                    continue;
                }
                let response = client.request(
                    "ascribe/edit",
                    json!({
                        "textDocument": { "uri": uri.as_str() },
                        "range": { "start": position(text, start), "end": position(text, end) },
                        "action": action,
                        "args": args,
                    }),
                );
                let result = response.response_result.expect("the request succeeds");
                if result.get("error").is_some() {
                    continue;
                }
                let changes = result["edit"]["changes"][uri.as_str()]
                    .as_array()
                    .expect("edits to the page");
                let after = apply(text, changes);
                if !checked.insert((source.path.to_string(), after.clone())) {
                    continue;
                }
                edits += 1;
                let what = format!("{action} at {start}..{end} of {}", source.path);
                assert_canonical(text, &after, &options, &model, &what);
                let sources: Vec<SourceFile> = project
                    .sources()
                    .iter()
                    .map(|s| SourceFile {
                        text: if s.path == source.path {
                            after.clone()
                        } else {
                            s.text.clone()
                        },
                        ..s.clone()
                    })
                    .collect();
                let edited = Project::from_parts(
                    root.clone(),
                    project.content_root().clone(),
                    model.clone(),
                    project.model_text().to_owned(),
                    sources,
                );
                for (key, count) in diagnostics(&edited) {
                    let before = baseline.get(&key).copied().unwrap_or_default();
                    assert!(count <= before, "{what} adds {key:?}:\n{after}");
                }
            }
        }
    }
    assert!(edits > 0, "no edit was made in {}", root.display());
}

/// The ranges to try: the start of each line and a few characters into it,
/// just inside each `[`, `!`, and `{`, the first word of each line, and each
/// block whole and with the block after it.
fn ranges(text: &str, blocks: &[Block]) -> Vec<(usize, usize)> {
    let mut out = BTreeSet::new();
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        out.insert((at, at));
        let indent = line.len() - line.trim_start().len();
        let inside = (at + indent + 2).min(at + line.trim_end().len());
        if text.is_char_boundary(inside) {
            out.insert((inside, inside));
        }
        if let Some(word) = first_word(line) {
            out.insert((at + word.0, at + word.1));
        }
        at += line.len();
    }
    for (i, c) in text.char_indices() {
        if matches!(c, '[' | '!' | '{') {
            out.insert((i + 1, i + 1));
        }
    }
    let mut spans = Vec::new();
    collect_blocks(blocks, &mut spans);
    out.extend(spans);
    out.into_iter().collect()
}

/// The first run of three or more letters in a line.
fn first_word(line: &str) -> Option<(usize, usize)> {
    let mut start = None;
    for (i, c) in line.char_indices() {
        match (c.is_alphabetic(), start) {
            (true, None) => start = Some(i),
            (false, Some(s)) if i - s >= 3 => return Some((s, i)),
            (false, Some(_)) => start = None,
            _ => {}
        }
    }
    None
}

fn collect_blocks(blocks: &[Block], out: &mut Vec<(usize, usize)>) {
    for (i, block) in blocks.iter().enumerate() {
        out.push((block.span.start(), block.span.end()));
        if let Some(next) = blocks.get(i + 1) {
            out.push((block.span.start(), next.span.end()));
        }
        match &block.kind {
            BlockKind::BlockQuote(q) => collect_blocks(&q.children, out),
            BlockKind::List(l) => {
                for item in &l.items {
                    collect_blocks(&item.children, out);
                }
            }
            BlockKind::Container(c) => collect_blocks(&c.children, out),
            BlockKind::Group(g) => {
                for arm in &g.arms {
                    collect_blocks(&arm.children, out);
                }
            }
            _ => {}
        }
    }
}

/// Valid arguments for every action, from the page's targets.
fn arguments(targets: &Value, path: &str) -> Vec<(&'static str, Value)> {
    let list = |key: &str| targets[key].as_array().cloned().unwrap_or_default();
    let page_link = list("pages")
        .iter()
        .find(|p| p["path"] != path)
        .map(|p| p["link"].as_str().unwrap().to_owned())
        .unwrap_or_else(|| "index.md".to_owned());
    let mut out = vec![
        ("wrapNote", json!({ "type": "tip" })),
        ("setNoteType", json!({ "type": "warning" })),
        ("unwrapNote", json!({})),
        ("noteToDetails", json!({ "title": "More" })),
        ("wrapDetails", json!({ "title": "More" })),
        ("unwrapDetails", json!({})),
        ("makeSteps", json!({})),
        ("removeSteps", json!({})),
        ("addHeadingId", json!({})),
        ("insertNote", json!({ "type": "note" })),
        ("insertSteps", json!({ "count": 2 })),
        ("removeVariantArm", json!({})),
        ("insertDetails", json!({ "title": "More" })),
        ("linkSelection", json!({ "destination": page_link })),
        ("insertLink", json!({ "destination": page_link })),
        ("setLinkTarget", json!({ "destination": page_link })),
        ("useTargetTitle", json!({})),
        ("setImageWidth", json!({ "width": 400 })),
        ("setImageAlt", json!({ "alt": "A new description" })),
    ];
    for dimension in list("dimensions") {
        let values: Vec<String> = dimension["values"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["value"].as_str().unwrap().to_owned())
            .collect();
        out.push((
            "insertVariantGroup",
            json!({ "dimension": dimension["name"], "values": values }),
        ));
        if let Some(value) = values.first() {
            out.push(("markAvailable", json!({ "spec": value })));
        }
        for value in &values {
            out.push(("addVariantArm", json!({ "value": value })));
            out.push(("setPageAvailable", json!({ "spec": value })));
            out.push((
                "setPageVariant",
                json!({ "dimension": dimension["name"], "value": value }),
            ));
        }
    }
    if let Some(feature) = list("features").first() {
        out.push(("markAvailable", json!({ "spec": feature["key"] })));
    }
    if let Some(phrase) = list("phrases").first() {
        out.push(("insertPhrase", json!({ "key": phrase["key"] })));
    }
    if let Some(fragment) = list("fragments").first() {
        out.push(("insertInclude", json!({ "path": fragment["include"] })));
    }
    if let Some(image) = list("images").first() {
        out.push((
            "insertImage",
            json!({ "path": image["link"], "alt": "A picture", "attributes": {} }),
        ));
    }
    if let Some(file) = list("snippets")
        .iter()
        .flat_map(|s| s["files"].as_array().cloned().unwrap_or_default())
        .next()
    {
        out.push((
            "insertSnippet",
            json!({ "address": file["address"], "lang": "text" }),
        ));
    }
    for widget in list("widgets") {
        let mut attributes = serde_json::Map::new();
        for a in widget["attributes"].as_array().unwrap() {
            if a["required"] == true {
                let value = match a["type"].as_str().unwrap() {
                    "number" => json!(1),
                    "boolean" => json!(true),
                    "enum" | "set" if !a["values"].as_array().unwrap().is_empty() => {
                        a["values"][0].clone()
                    }
                    _ => json!("value"),
                };
                attributes.insert(a["key"].as_str().unwrap().to_owned(), value);
            }
        }
        let primary = match widget["primary"].as_str().unwrap() {
            "none" => Value::Null,
            "identifier" => json!("word"),
            _ => json!("Some text."),
        };
        out.push((
            "insertWidget",
            json!({ "name": widget["name"], "primary": primary, "attributes": attributes }),
        ));
        out.push((
            "insertWidget",
            json!({ "name": widget["name"], "attributes": attributes }),
        ));
    }
    out
}

/// Formatting the page after the edit changes nothing the edit wrote: no
/// formatting edit touches the bytes between the first and the last that
/// differ.
fn assert_canonical(
    before: &str,
    after: &str,
    options: &ascribe_syntax::ParseOptions,
    model: &ascribe_model::ContentModel,
    what: &str,
) {
    let prefix = before
        .bytes()
        .zip(after.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    let suffix = before
        .bytes()
        .rev()
        .zip(after.bytes().rev())
        .take_while(|(a, b)| a == b)
        .count()
        .min(before.len().min(after.len()) - prefix);
    let changed = Span::new(prefix, after.len() - suffix);
    for e in ascribe_fmt::format(after, options, model) {
        let touches = e.span.start() < changed.end() && changed.start() < e.span.end()
            || (e.span.is_empty()
                && changed.start() < e.span.start()
                && e.span.start() < changed.end());
        assert!(
            !touches,
            "{what} isn't canonical: the formatter changes {:?} to {:?} in\n{after}",
            &after[e.span.range()],
            e.new_text
        );
    }
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

fn apply(text: &str, edits: &[Value]) -> String {
    let edits: Vec<ascribe_core::TextEdit> = edits
        .iter()
        .map(|e| {
            ascribe_core::TextEdit::replace(
                Span::new(
                    offset(text, &e["range"]["start"]),
                    offset(text, &e["range"]["end"]),
                ),
                e["newText"].as_str().expect("text"),
            )
        })
        .collect();
    ascribe_core::apply_edits(text, &edits).expect("edits that apply")
}
