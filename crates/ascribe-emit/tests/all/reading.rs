//! A page's reading (`ascribe_resolve::reading`), what `ascribe search`
//! matches, says what the plain output says: for every project under
//! `examples/` and `tests/conformance/cases/`, in every build, each block of
//! each page's reading but its description, which the plain output doesn't
//! write, is found, in order, in the page's plain output read
//! the same way (parsed, markup removed, white space collapsed). A block the
//! reading has and the plain output doesn't is a bug in one of them.
//!
//! The reading's own map back to the source is held in `ascribe-resolve`'s
//! `tests/all/reading.rs`.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::Path;

use ascribe_emit::{EmitContext, PlainEmitter, emit_page};
use ascribe_resolve::reading::{In, Reading, html_text};
use ascribe_resolve::{AstroRouter, Project};
use ascribe_syntax::{Block, BlockKind, Inline, InlineKind, ParseOptions};

use crate::support::{cases, example, models, repository};

/// Inline content as plain text, as a reader reads it. The line a link
/// out of a line of a versioned build leads to, which the plain output
/// writes after the link in parentheses, is left out: the content model
/// writes it.
fn plain(list: &[Inline], out: &mut String) {
    let mut after_link = false;
    for node in list {
        if std::mem::take(&mut after_link)
            && let InlineKind::Text(v) = &node.kind
            && let Some(rest) = v.strip_prefix(" (")
            && let Some(close) = rest.find(')')
            && !rest[..close].contains(' ')
        {
            out.push_str(&rest[close + 1..]);
            continue;
        }
        after_link = matches!(node.kind, InlineKind::Link(_));
        match &node.kind {
            InlineKind::Text(v) | InlineKind::Code(v) => out.push_str(v),
            InlineKind::Html(v) => out.push_str(&html_text(v)),
            InlineKind::SoftBreak | InlineKind::HardBreak => out.push(' '),
            InlineKind::Emphasis(c) | InlineKind::Strong(c) => plain(c, out),
            InlineKind::Link(link) => plain(&link.children, out),
            InlineKind::Image(image) => plain(&image.children, out),
            InlineKind::Phrase(p) => out.push_str(&format!("{{{}}}", p.key)),
        }
    }
}

/// The plain output's blocks read as text, in order, each followed by a
/// space. A row's availability, which the content model writes, is left
/// out of its first cell. The plain output is CommonMark, so what reads as
/// a directive in it is its text as written.
fn read_plain(source: &str, blocks: &[Block], out: &mut String) {
    for block in blocks {
        match &block.kind {
            BlockKind::Heading(h) => plain(&h.inlines, out),
            BlockKind::Paragraph(p) => plain(&p.inlines, out),
            BlockKind::CodeBlock(c) => out.push_str(&c.literal),
            BlockKind::HtmlBlock(h) => out.push_str(&html_text(&h.literal)),
            BlockKind::Table(t) => {
                for row in &t.rows {
                    for cell in &row.cells {
                        let mut text = String::new();
                        plain(&cell.inlines, &mut text);
                        let text = match text.find("(Available: ") {
                            Some(at) => text[..at].to_owned(),
                            None => text,
                        };
                        out.push_str(&text);
                        out.push(' ');
                    }
                }
            }
            BlockKind::BlockQuote(q) => read_plain(source, &q.children, out),
            BlockKind::List(list) => {
                for item in &list.items {
                    read_plain(source, &item.children, out);
                }
            }
            BlockKind::Directive(_)
            | BlockKind::Container(_)
            | BlockKind::Group(_)
            | BlockKind::Title(_) => {
                let written = &source[block.span.range()];
                let mut chars = written.chars().peekable();
                while let Some(c) = chars.next() {
                    match chars.peek() {
                        Some(next) if c == '\\' && next.is_ascii_punctuation() => {}
                        _ => out.push(c),
                    }
                }
            }
            BlockKind::ThematicBreak | BlockKind::End(_) => {}
        }
        out.push(' ');
    }
}

fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Every block of every page's reading, in every build of `project`, in
/// its plain output, in order; the problems, one line each.
fn disagreements(root: &Path, name: &str, project: &Project) -> Vec<String> {
    let mut out = Vec::new();
    for build in &project.model().builds {
        let project = project.for_build(build);
        let project = project.as_ref();
        let router = AstroRouter::from_consumer(&project.model().consumer);
        let resolved = project.resolve_build(build, &router);
        let cx = EmitContext::new(project, root, build);
        let emitter = PlainEmitter::default();
        for page in &resolved.pages {
            let Ok(emitted) = emit_page(&emitter, &cx, page) else {
                continue;
            };
            let doc = ascribe_syntax::parse(&emitted.text, &ParseOptions::default());
            let mut text = String::new();
            read_plain(&emitted.text, &doc.blocks, &mut text);
            let text = collapse(&text);
            let mut at = 0;
            // The plain output writes no frontmatter, so the description
            // isn't in it; the title is its first heading.
            let blocks = Reading::of(project, page).blocks;
            for block in blocks
                .into_iter()
                .filter(|b| b.kind != In::Description && b.kind.searched())
            {
                match text[at..].find(&block.text) {
                    Some(found) => at += found + block.text.len(),
                    None => out.push(format!(
                        "{name}, build {}, {}: {:?} isn't in the plain output after {at}:\n{text}",
                        build.name, page.path, block.text
                    )),
                }
            }
        }
    }
    out
}

#[test]
fn every_example_reads_as_its_plain_output_says() {
    let mut found = Vec::new();
    models(&repository().join("examples"), &mut found);
    assert!(found.len() > 5, "the examples weren't found: {found:?}");
    let mut problems = Vec::new();
    for model in found {
        let (root, project) = example(&model).expect("an example loads");
        problems.extend(disagreements(&root, &model.display().to_string(), &project));
    }
    assert!(problems.is_empty(), "{}", problems.join("\n\n"));
}

#[test]
fn every_conformance_case_reads_as_its_plain_output_says() {
    let conformance = repository().join("tests/conformance");
    let mut found = Vec::new();
    cases(
        &conformance.join("cases"),
        &conformance.join("_model/ascribe.toml"),
        &mut found,
    );
    assert!(
        found.len() > 100,
        "the cases weren't found: {}",
        found.len()
    );
    let mut problems = Vec::new();
    for (dir, project) in found {
        problems.extend(disagreements(&dir, &dir.display().to_string(), &project));
    }
    assert!(problems.is_empty(), "{}", problems.join("\n\n"));
}
