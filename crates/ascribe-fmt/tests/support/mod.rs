//! Shared helpers: the shared fixture model, and an outline of a parsed
//! document that ignores spans, formatting, and attribute order.

#![allow(dead_code, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use ascribe_core::{AttributeValue, FileId};
use ascribe_model::{ContentModel, load_str};
use ascribe_syntax::*;

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the repository root exists")
}

pub fn shared_model_path() -> PathBuf {
    repo_root().join("tests/conformance/_model/ascribe.toml")
}

/// The model in the file at `path`.
pub fn model_at(path: &Path) -> ContentModel {
    let text = std::fs::read_to_string(path).expect("the model is readable");
    load_str(&text, FileId::new(0)).unwrap_or_else(|e| panic!("{}: {e:?}", path.display()))
}

/// The conformance suite's shared model.
pub fn shared_model() -> ContentModel {
    shared_model_ref().clone()
}

/// The shared model, loaded once.
pub fn shared_model_ref() -> &'static ContentModel {
    static MODEL: std::sync::OnceLock<ContentModel> = std::sync::OnceLock::new();
    MODEL.get_or_init(|| model_at(&shared_model_path()))
}

/// Parse options from a model, as a project would build them.
pub fn options(model: &ContentModel) -> ParseOptions {
    ascribe_fmt::options_from_model(model)
}

/// Formats `source` under the shared model.
pub fn fmt(source: &str) -> String {
    let model = shared_model_ref();
    ascribe_fmt::format_source(source, &options(model), model)
}

/// Every `.md` under `dir`, recursively, in path order.
pub fn markdown_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries {
        let path = entry.expect("readable entry").path();
        if path.is_dir() {
            out.extend(markdown_files(&path));
        } else if path.extension().is_some_and(|e| e == "md") {
            out.push(path);
        }
    }
    out.sort();
    out
}

/// What a document *means*: its blocks and inlines with their text and
/// values, and each directive's name, form, binding, attributes (as a
/// sorted list, so order doesn't matter), primary, and title. Spacing is
/// gone: two documents with the same outline differ only in formatting.
pub fn outline(source: &str, options: &ParseOptions) -> String {
    let doc = parse(source, options);
    let mut out = String::new();
    blocks(source, &doc.blocks, 0, &mut out);
    for d in &doc.definitions {
        out.push_str(&format!(
            "definition {:?} {:?} {:?}\n",
            d.normalized_label,
            d.url,
            d.title.as_ref().map(|t| &t.text)
        ));
    }
    for issue in &doc.issues {
        // The blank line the formatter removes is the one thing it fixes that
        // a diagnostic reports.
        if issue.slug.as_str() != "binding-blank-line" {
            out.push_str(&format!("issue {}\n", issue.slug));
        }
    }
    out
}

fn pad(depth: usize) -> String {
    "  ".repeat(depth)
}

fn blocks(source: &str, list: &[Block], depth: usize, out: &mut String) {
    for block in list {
        let p = pad(depth);
        match &block.kind {
            BlockKind::Heading(h) => {
                out.push_str(&format!("{p}heading {} setext={}\n", h.level, h.setext));
                inlines(&h.inlines, depth + 1, out);
            }
            BlockKind::Paragraph(para) => {
                out.push_str(&format!("{p}paragraph\n"));
                inlines(&para.inlines, depth + 1, out);
            }
            BlockKind::CodeBlock(c) => {
                out.push_str(&format!(
                    "{p}code fenced={} {:?} {:?}\n",
                    c.fenced, c.info, c.literal
                ));
            }
            BlockKind::BlockQuote(q) => {
                out.push_str(&format!("{p}quote\n"));
                blocks(source, &q.children, depth + 1, out);
            }
            BlockKind::List(l) => {
                out.push_str(&format!(
                    "{p}list ordered={} start={:?} tight={}\n",
                    l.ordered, l.start, l.tight
                ));
                for item in &l.items {
                    out.push_str(&format!("{p}  item\n"));
                    blocks(source, &item.children, depth + 2, out);
                }
            }
            BlockKind::HtmlBlock(h) => out.push_str(&format!("{p}html {:?}\n", h.literal)),
            BlockKind::ThematicBreak => out.push_str(&format!("{p}break\n")),
            BlockKind::Table(t) => {
                out.push_str(&format!("{p}table\n"));
                for row in &t.rows {
                    out.push_str(&format!("{p}  row header={}\n", row.header));
                    for cell in &row.cells {
                        out.push_str(&format!("{p}    cell\n"));
                        inlines(&cell.inlines, depth + 3, out);
                    }
                }
            }
            BlockKind::Directive(d) => directive(source, "directive", d, depth, out),
            BlockKind::End(_) => out.push_str(&format!("{p}end\n")),
            BlockKind::Container(c) => {
                directive(source, "container", &c.opener, depth, out);
                blocks(source, &c.children, depth + 1, out);
                out.push_str(&format!("{p}  closed={}\n", c.end.is_some()));
            }
            BlockKind::Group(g) => {
                out.push_str(&format!("{p}group {} closed={}\n", g.name, g.end.is_some()));
                for arm in &g.arms {
                    directive(source, "arm", &arm.opener, depth + 1, out);
                    blocks(source, &arm.children, depth + 2, out);
                }
            }
            BlockKind::Title(_) => out.push_str(&format!("{p}title\n")),
        }
    }
}

fn directive(source: &str, kind: &str, d: &DirectiveLine, depth: usize, out: &mut String) {
    let p = pad(depth);
    let mut attributes: Vec<String> = d
        .attributes
        .iter()
        .flat_map(|b| &b.attributes)
        .map(|a| {
            let value = match &a.value {
                Some(AttributeValue::Set { members, .. }) => {
                    format!(
                        "set{:?}",
                        members.iter().map(|m| &m.text).collect::<Vec<_>>()
                    )
                }
                Some(v) => format!("{:?}", v.as_text()),
                None => "none".to_owned(),
            };
            format!("{}={value}", a.key)
        })
        .collect();
    attributes.sort();
    out.push_str(&format!(
        "{p}{kind} @{} form={:?} binding={:?} attrs={attributes:?} closed={}\n",
        d.name, d.form, d.binding, d.attributes_closed
    ));
    match &d.primary {
        Some(PrimaryValue::Text(t)) => {
            out.push_str(&format!("{p}  primary text\n"));
            inlines(&t.inlines, depth + 2, out);
        }
        Some(PrimaryValue::Identifier(i)) => out.push_str(&format!(
            "{p}  primary id {:?} {:?}\n",
            i.text,
            i.trailing.map(|s| raw_text(source, s))
        )),
        Some(PrimaryValue::Line(l)) => out.push_str(&format!("{p}  primary line {:?}\n", l.text)),
        Some(PrimaryValue::Unexpected(s)) => out.push_str(&format!(
            "{p}  primary unexpected {:?}\n",
            raw_text(source, *s)
        )),
        None => {}
    }
    if let Some(s) = d.unexpected {
        out.push_str(&format!("{p}  unexpected {:?}\n", raw_text(source, s)));
    }
    if let Some(t) = &d.title {
        out.push_str(&format!("{p}  title\n"));
        inlines(&t.inlines, depth + 2, out);
    }
}

fn inlines(list: &[Inline], depth: usize, out: &mut String) {
    let p = pad(depth);
    for inline in list {
        match &inline.kind {
            InlineKind::Text(t) => out.push_str(&format!("{p}text {t:?}\n")),
            InlineKind::Code(c) => out.push_str(&format!("{p}code {c:?}\n")),
            InlineKind::SoftBreak => out.push_str(&format!("{p}softbreak\n")),
            InlineKind::HardBreak => out.push_str(&format!("{p}hardbreak\n")),
            InlineKind::Html(h) => out.push_str(&format!("{p}html {h:?}\n")),
            InlineKind::Emphasis(c) => {
                out.push_str(&format!("{p}em\n"));
                inlines(c, depth + 1, out);
            }
            InlineKind::Strong(c) => {
                out.push_str(&format!("{p}strong\n"));
                inlines(c, depth + 1, out);
            }
            InlineKind::Link(l) => {
                out.push_str(&format!(
                    "{p}link {:?} {:?} {:?}\n",
                    l.form, l.destination, l.title
                ));
                inlines(&l.children, depth + 1, out);
            }
            InlineKind::Image(i) => {
                let mut attributes: Vec<String> = i
                    .attributes
                    .iter()
                    .flat_map(|a| &a.block.attributes)
                    .map(|a| format!("{}={:?}", a.key, a.value.as_ref().map(|v| v.members())))
                    .collect();
                attributes.sort();
                out.push_str(&format!(
                    "{p}image {:?} {:?} {:?} attrs={attributes:?}\n",
                    i.form, i.destination, i.title
                ));
                inlines(&i.children, depth + 1, out);
            }
            InlineKind::Phrase(ph) => out.push_str(&format!("{p}phrase {}\n", ph.key)),
        }
    }
}
