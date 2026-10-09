//! Every operation of `ascribe/edit`, with valid arguments, at a
//! representative set of ranges on every page of the example projects: each
//! edit the server returns is canonical where it wrote, and adds no
//! diagnostic to the project.
//!
//! The ranges are the start of each line, the position just inside each `[`,
//! `!`, and `{` (a link, an image, and an attribute block), and each block
//! whole and with the block after it (every nesting level, so an arm of a
//! group and an item of a list count).
//!
//! The pages' ranges are tried in parallel, each thread through its own
//! server. The check after an edit reaches what an edit to one file can
//! change, as the incremental index scopes it (`Affected::recheck`): the
//! file's own file-level diagnostics, and the page-level diagnostics of the
//! pages the file is part of (itself, and the pages that include it), of the
//! pages that link to any of those (and the pages including those linking
//! files), and of every page when one of them is a page the glossary's terms
//! link to. The baseline is computed the same way, so the two compare like
//! for like.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZero;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use ascribe_check::{PageChecker, Project, check_file};
use ascribe_core::{LineIndex, RelPath, Span, WideEncoding, WideLineCol};
use ascribe_model::Build;
use ascribe_resolve::FileKind;
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

/// The diagnostics an edit to one file can change (see the module
/// documentation), and what they were before any edit.
struct Reach {
    /// The pages whose page-level diagnostics are checked.
    pages: Vec<RelPath>,
    /// The diagnostics in reach before any edit, by file and code.
    baseline: BTreeMap<(String, String), usize>,
}

impl Reach {
    fn of(
        project: &Project,
        index: &ascribe_resolve::Project,
        build: &Build,
        path: &RelPath,
    ) -> Reach {
        let is_page = |p: &RelPath| index.file(p).is_some_and(|f| f.kind == FileKind::Page);
        // The pages the file is part of.
        let mut part_of: BTreeSet<RelPath> = index.including_pages(path).into_iter().collect();
        if is_page(path) {
            part_of.insert(path.clone());
        }
        let mut pages = part_of.clone();
        // The pages that link to the file or to a page it's part of: an edit
        // can take away an id a link names.
        for target in part_of.iter().chain(std::iter::once(path)) {
            for link in index.links_to(target) {
                if is_page(&link.file) {
                    pages.insert(link.file.clone());
                }
                pages.extend(index.including_pages(&link.file));
            }
        }
        // The glossary's terms link from every page.
        let glossary = ascribe_resolve::glossary_targets(project.model());
        if glossary.contains(path) || part_of.iter().any(|p| glossary.contains(p)) {
            pages.extend(
                index
                    .files()
                    .filter(|f| is_page(&f.path))
                    .map(|f| f.path.clone()),
            );
        }
        let mut reach = Reach {
            pages: pages.into_iter().collect(),
            baseline: BTreeMap::new(),
        };
        reach.baseline = reach.diagnostics(project, build, path);
        reach
    }

    /// The diagnostics in reach, by file and code: the file-level ones of
    /// `path`, and the page-level ones of the pages, as `ascribe check`
    /// finds them on the project's texts.
    fn diagnostics(
        &self,
        project: &Project,
        build: &Build,
        path: &RelPath,
    ) -> BTreeMap<(String, String), usize> {
        let file = project.source_at(path).expect("the edited file");
        let mut all = check_file(project, file);
        all.extend(PageChecker::new(project).check_pages(build, &self.pages));
        let mut out = BTreeMap::new();
        for d in all {
            let file = project
                .file(d.location.file)
                .map(|f| f.display_path.to_owned())
                .unwrap_or_default();
            *out.entry((file, d.code.to_owned())).or_default() += 1;
        }
        out
    }
}

fn every_edit(root: &Path) {
    let root = support::real_path(root);
    let project = Project::load(&root.join("ascribe.toml")).expect("the example loads");
    let model = project.model().clone();
    let build = model.editor_default_build().clone();
    let options = ascribe_fmt::options_from_model(&model);
    let content = root.join(project.content_root().as_str());
    let index = project.held_index();
    let reaches: Vec<Reach> = project
        .sources()
        .iter()
        .map(|s| Reach::of(&project, &index, &build, &s.path))
        .collect();
    // Every range of every page, in order: the threads take them in turn, so
    // one page's ranges are shared out rather than one page per thread.
    let items: Vec<(usize, (usize, usize))> = project
        .sources()
        .iter()
        .enumerate()
        .flat_map(|(i, s)| {
            let doc = ascribe_syntax::parse(&s.text, &options);
            ranges(&s.text, &doc.blocks)
                .into_iter()
                .map(move |range| (i, range))
        })
        .collect();
    let next = AtomicUsize::new(0);
    let checked: Mutex<BTreeSet<(String, String)>> = Mutex::new(BTreeSet::new());
    let edits = AtomicUsize::new(0);
    let threads = thread::available_parallelism()
        .map_or(4, NonZero::get)
        .clamp(1, items.len().max(1));
    thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                let mut client = Client::start(&root);
                // Each page's arguments, from its targets, once per thread.
                let mut arguments_of: BTreeMap<usize, Vec<(&str, Value)>> = BTreeMap::new();
                while let Some(&(i, (start, end))) =
                    items.get(next.fetch_add(1, Ordering::SeqCst))
                {
                    let source = &project.sources()[i];
                    let path = content.join(source.path.as_str());
                    let uri = support::uri(&path);
                    let text = &source.text;
                    let lines = LineIndex::new(text);
                    let arguments = arguments_of.entry(i).or_insert_with(|| {
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
                        arguments(&targets, &source.path.to_string())
                    });
                    for (action, args) in arguments.iter() {
                        // The page's own settings don't depend on where the cursor is.
                        if action.starts_with("setPage") && start > 0 {
                            continue;
                        }
                        let response = client.request(
                            "ascribe/edit",
                            json!({
                                "textDocument": { "uri": uri.as_str() },
                                "range": { "start": position(&lines, start), "end": position(&lines, end) },
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
                        let after = apply(text, &lines, changes);
                        if !checked
                            .lock()
                            .unwrap()
                            .insert((source.path.to_string(), after.clone()))
                        {
                            continue;
                        }
                        edits.fetch_add(1, Ordering::SeqCst);
                        let what = format!("{action} at {start}..{end} of {}", source.path);
                        assert_canonical(text, &after, &options, &model, &what);
                        let edited = project.with_source(&source.path, after.clone());
                        let reach = &reaches[i];
                        for (key, count) in reach.diagnostics(&edited, &build, &source.path) {
                            let before = reach.baseline.get(&key).copied().unwrap_or_default();
                            assert!(count <= before, "{what} adds {key:?}:\n{after}");
                        }
                    }
                }
                client.shutdown();
            });
        }
    });
    assert!(
        edits.load(Ordering::SeqCst) > 0,
        "no edit was made in {}",
        root.display()
    );
}

/// The ranges to try (see the module documentation): the start of each
/// line, just inside each `[`, `!`, and `{`, and each block whole and with
/// the block after it.
fn ranges(text: &str, blocks: &[Block]) -> Vec<(usize, usize)> {
    let mut out = BTreeSet::new();
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        out.insert((at, at));
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

fn position(lines: &LineIndex, offset: usize) -> Value {
    let p = lines
        .wide_line_col(WideEncoding::Utf16, offset)
        .expect("a position");
    json!({ "line": p.line, "character": p.col })
}

fn offset(lines: &LineIndex, position: &Value) -> usize {
    lines
        .wide_offset(
            WideEncoding::Utf16,
            WideLineCol {
                line: position["line"].as_u64().unwrap() as u32,
                col: position["character"].as_u64().unwrap() as u32,
            },
        )
        .expect("an offset")
}

fn apply(text: &str, lines: &LineIndex, edits: &[Value]) -> String {
    let edits: Vec<ascribe_core::TextEdit> = edits
        .iter()
        .map(|e| {
            ascribe_core::TextEdit::replace(
                Span::new(
                    offset(lines, &e["range"]["start"]),
                    offset(lines, &e["range"]["end"]),
                ),
                e["newText"].as_str().expect("text"),
            )
        })
        .collect();
    ascribe_core::apply_edits(text, &edits).expect("edits that apply")
}
