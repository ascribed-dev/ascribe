//! Shared test support: a checker that every span in a tree covers exactly
//! its source text, and helpers to find inputs.

#![allow(dead_code, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use tessera_core::Span;
use tessera_syntax::*;

/// Checks every span in `doc` against `source`. Returns the problems found.
pub fn check_tree(source: &str, doc: &ParsedDocument) -> Vec<String> {
    let mut c = Checker {
        source,
        problems: Vec::new(),
    };
    c.blocks(&doc.blocks, doc.span, "document");
    if let Some(f) = &doc.frontmatter {
        c.within(f.content, f.span, "frontmatter content");
        c.text_starts(f.span, "---", "frontmatter");
    }
    for issue in &doc.issues {
        c.valid(issue.location.span, "issue");
    }
    for escaped in &doc.escaped_phrases {
        c.phrase(escaped, "escaped phrase");
        if !c.text(escaped.span).starts_with("\\{") {
            c.problem("escaped phrase", escaped.span, "doesn't start with `\\{`");
        }
    }
    c.problems
}

struct Checker<'a> {
    source: &'a str,
    problems: Vec<String>,
}

impl Checker<'_> {
    fn text(&self, span: Span) -> &str {
        self.source.get(span.start()..span.end()).unwrap_or("")
    }

    fn problem(&mut self, what: &str, span: Span, why: &str) {
        self.problems
            .push(format!("{what} {span:?} {:?}: {why}", self.text(span)));
    }

    /// In range and on character boundaries.
    fn valid(&mut self, span: Span, what: &str) -> bool {
        let ok = span.end() <= self.source.len()
            && self.source.is_char_boundary(span.start())
            && self.source.is_char_boundary(span.end());
        if !ok {
            self.problem(what, span, "not a valid range of the source");
        }
        ok
    }

    /// A phrase's spans: the candidate is `{key}` (or `\{key}`) in the source.
    fn phrase(&mut self, phrase: &Phrase, what: &str) {
        let (span, key) = (phrase.span, phrase.key_span);
        if !self.valid(span, what) || !self.valid(key, what) {
            return;
        }
        self.within(key, span, "phrase key");
        if self.text(key) != phrase.key {
            self.problem("phrase key", key, &format!("isn't {:?}", phrase.key));
        }
        if !self.text(span).ends_with(&format!("{{{}}}", phrase.key)) || key.end() + 1 != span.end()
        {
            self.problem(what, span, "isn't `{key}` around its key");
        }
    }

    fn within(&mut self, inner: Span, outer: Span, what: &str) {
        if !self.valid(inner, what) {
            return;
        }
        if inner.start() < outer.start() || inner.end() > outer.end() {
            self.problem(what, inner, &format!("outside its parent {outer:?}"));
        }
    }

    fn text_starts(&mut self, span: Span, prefix: &str, what: &str) {
        if !self.text(span).starts_with(prefix) {
            self.problem(what, span, &format!("doesn't start with {prefix:?}"));
        }
    }

    /// A span that holds a line's worth of content: it doesn't begin with
    /// whitespace or end with whitespace or a line ending.
    fn tight(&mut self, span: Span, what: &str) {
        let text = self.text(span);
        if text.is_empty() {
            self.problem(what, span, "empty");
        } else if text.starts_with([' ', '\t', '\n', '\r']) {
            self.problem(what, span, "starts with whitespace");
        } else if text.ends_with([' ', '\t', '\n', '\r']) {
            self.problem(what, span, "ends with whitespace");
        }
    }

    fn blocks(&mut self, blocks: &[Block], parent: Span, what: &str) {
        let mut previous_end = 0;
        for block in blocks {
            self.block(block, parent, what);
            if block.span.start() < previous_end {
                self.problem("block", block.span, "overlaps the block before it");
            }
            previous_end = block.span.end();
        }
    }

    fn block(&mut self, block: &Block, parent: Span, what: &str) {
        let span = block.span;
        self.within(span, parent, what);
        if !self.valid(span, "block") {
            return;
        }
        // No block ends in a line ending.
        if self.text(span).ends_with(['\n', '\r']) {
            self.problem("block", span, "ends with a line ending");
        }
        match &block.kind {
            BlockKind::Heading(h) => {
                self.tight(span, "heading");
                self.within(h.content, span, "heading content");
                self.inlines(&h.inlines, span);
            }
            BlockKind::Paragraph(p) => {
                self.tight(span, "paragraph");
                self.inlines(&p.inlines, span);
            }
            BlockKind::CodeBlock(c) => {
                for phrase in c.phrases.iter().flatten() {
                    self.within(phrase.span, span, "code phrase");
                    self.phrase(phrase, "code phrase");
                }
                if c.phrases.is_some() && !c.fenced {
                    self.problem("code", span, "an indented block with phrases");
                }
                if let Some(info) = c.info_span {
                    self.within(info, span, "info");
                    if self.text(info) != c.info && !self.text(info).contains(['\\', '&']) {
                        self.problem("info", info, "isn't the info string");
                    }
                }
            }
            BlockKind::BlockQuote(q) => {
                self.text_starts(span, ">", "block quote");
                self.blocks(&q.children, span, "block quote child");
            }
            BlockKind::List(l) => {
                let mut previous_end = 0;
                for item in &l.items {
                    self.within(item.span, span, "list item");
                    self.within(item.marker, item.span, "marker");
                    if item.marker.start() != item.span.start() || item.marker.is_empty() {
                        self.problem("marker", item.marker, "isn't at the item's start");
                    }
                    if item.span.start() < previous_end {
                        self.problem("item", item.span, "overlaps the item before it");
                    }
                    previous_end = item.span.end();
                    self.blocks(&item.children, item.span, "item child");
                }
            }
            BlockKind::HtmlBlock(_) | BlockKind::ThematicBreak => {}
            BlockKind::Table(t) => {
                for row in &t.rows {
                    self.within(row.span, span, "table row");
                    for cell in &row.cells {
                        self.within(cell.span, row.span, "table cell");
                        self.inlines(&cell.inlines, cell.span);
                    }
                }
            }
            BlockKind::Directive(d) => self.directive(d, span),
            BlockKind::End(e) => {
                self.text_starts(e.span, "@end", "end line");
                if self.text(e.span) != "@end" {
                    self.problem("end line", e.span, "isn't `@end`");
                }
                if let Some(extra) = e.extra {
                    self.tight(extra, "end line extra");
                }
            }
            BlockKind::Container(_) | BlockKind::Group(_) | BlockKind::Title(_) => {
                self.problem("block", span, "a node phase 05 doesn't produce");
            }
        }
    }

    fn directive(&mut self, d: &DirectiveLine, span: Span) {
        if d.span != span {
            self.problem("directive", d.span, "differs from its block's span");
        }
        self.tight(span, "directive line");
        self.text_starts(span, "@", "directive line");
        if self.text(d.name_span) != format!("@{}", d.name) {
            self.problem("name", d.name_span, "isn't `@name`");
        }
        if d.name_span.start() != span.start() {
            self.problem("name", d.name_span, "isn't at the line's start");
        }
        if let Some(attrs) = &d.attributes {
            self.within(attrs.span, span, "attribute block");
            self.text_starts(attrs.span, "{", "attribute block");
            if d.attributes_closed && !self.text(attrs.span).ends_with('}') {
                self.problem("attribute block", attrs.span, "doesn't end with `}`");
            }
            for a in &attrs.attributes {
                self.within(a.span, attrs.span, "attribute");
                self.within(a.key_span, a.span, "key");
                if self.text(a.key_span) != a.key {
                    self.problem("key", a.key_span, "isn't the key");
                }
                if let Some(v) = &a.value {
                    self.within(v.span(), a.span, "value");
                    match v {
                        tessera_core::AttributeValue::Token(t) => {
                            if self.text(t.span) != t.text {
                                self.problem("token", t.span, "isn't the token's text");
                            }
                        }
                        tessera_core::AttributeValue::Quoted { span, .. } => {
                            self.text_starts(*span, "\"", "quoted value");
                        }
                        tessera_core::AttributeValue::Set { members, span } => {
                            for m in members {
                                self.within(m.span, *span, "set member");
                                if self.text(m.span) != m.text {
                                    self.problem("member", m.span, "isn't the member's text");
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Some(colon) = d.colon {
            self.within(colon, span, "colon");
            if self.text(colon) != ":" {
                self.problem("colon", colon, "isn't `:`");
            }
        }
        if let Some(u) = d.unexpected {
            self.within(u, span, "unexpected text");
            self.tight(u, "unexpected text");
        }
        match &d.primary {
            None => {}
            Some(PrimaryValue::Identifier(p)) => {
                self.within(p.span, span, "identifier primary");
                if self.text(p.span) != p.text || p.text.contains([' ', '\t']) {
                    self.problem("identifier", p.span, "isn't the token");
                }
                if let Some(t) = p.trailing {
                    self.within(t, span, "trailing text");
                    self.tight(t, "trailing text");
                }
            }
            Some(PrimaryValue::Line(p)) => {
                self.within(p.span, span, "line primary");
                self.tight(p.span, "line primary");
                if self.text(p.span) != p.text {
                    self.problem("line primary", p.span, "isn't its text");
                }
            }
            Some(PrimaryValue::Unexpected(s)) => {
                self.within(*s, span, "unexpected primary");
                self.tight(*s, "unexpected primary");
            }
            Some(PrimaryValue::Text(p)) => {
                self.within(p.span, span, "text primary");
                self.tight(p.span, "text primary");
                if p.span.end() != span.end() {
                    self.problem("text primary", p.span, "doesn't end the directive line");
                }
                if p.lines.first().map(|l| l.start()) != Some(p.span.start()) {
                    self.problem("text primary", p.span, "first line isn't at its start");
                }
                for l in &p.lines {
                    self.within(*l, p.span, "primary line");
                    self.tight(*l, "primary line");
                }
                self.inlines(&p.inlines, p.span);
            }
        }
        let expected = match (d.colon, &d.primary, d.unexpected) {
            (Some(_), None, None) => Form::Container,
            _ => Form::Line,
        };
        if d.form != expected {
            self.problem("directive", span, "has the wrong form");
        }
    }

    fn inlines(&mut self, inlines: &[Inline], parent: Span) {
        let mut previous_end = parent.start();
        for inline in inlines {
            let span = inline.span;
            self.within(span, parent, "inline");
            if !self.valid(span, "inline") {
                continue;
            }
            if span.start() < previous_end {
                self.problem("inline", span, "overlaps the inline before it");
            }
            previous_end = span.end();
            let text = self.text(span).to_owned();
            match &inline.kind {
                InlineKind::Text(value) => {
                    if span.is_empty() {
                        self.problem("text", span, "empty");
                    }
                    // Text with nothing to decode is its source.
                    if !text.contains(['\\', '&', '\n', '\r']) && !value.contains('\u{fffd}') {
                        let plain = !text.contains(['\'', '"', '-', '.']);
                        if plain && *value != text {
                            self.problem("text", span, &format!("value is {value:?}"));
                        }
                    }
                }
                InlineKind::Code(_) => self.text_starts(span, "`", "code span"),
                InlineKind::SoftBreak | InlineKind::HardBreak => {
                    if !text.contains(['\n', '\r']) {
                        self.problem("break", span, "has no line ending");
                    }
                }
                InlineKind::Html(_) => self.text_starts(span, "<", "inline html"),
                InlineKind::Emphasis(children) => {
                    if !(text.starts_with('*') || text.starts_with('_')) {
                        self.problem("emphasis", span, "doesn't start with a marker");
                    }
                    self.inlines(children, span);
                }
                InlineKind::Strong(children) => {
                    if !(text.starts_with("**") || text.starts_with("__")) {
                        self.problem("strong", span, "doesn't start with markers");
                    }
                    self.inlines(children, span);
                }
                InlineKind::Link(l) => {
                    match l.form {
                        LinkForm::Autolink => self.text_starts(span, "<", "autolink"),
                        _ => self.text_starts(span, "[", "link"),
                    }
                    if let Some(label) = l.label {
                        self.within(label, span, "link label");
                    }
                    for phrase in &l.destination_phrases {
                        self.within(phrase.span, span, "destination phrase");
                        self.phrase(phrase, "destination phrase");
                    }
                    self.inlines(&l.children, span);
                }
                InlineKind::Image(i) => {
                    self.text_starts(span, "![", "image");
                    self.within(i.alt, span, "alt");
                    if let Some(label) = i.label {
                        self.within(label, span, "image label");
                    }
                    for phrase in &i.destination_phrases {
                        self.within(phrase.span, span, "destination phrase");
                        self.phrase(phrase, "destination phrase");
                    }
                    // The attribute block is the end of the image.
                    if let Some(attributes) = &i.attributes {
                        let block = attributes.block.span;
                        self.within(block, span, "image attributes");
                        if block.end() != span.end() {
                            self.problem("image attributes", block, "isn't at the image's end");
                        }
                        if !self.text(block).starts_with('{') || !self.text(block).ends_with('}') {
                            self.problem("image attributes", block, "isn't `{…}`");
                        }
                    }
                    self.inlines(&i.children, i.alt);
                }
                InlineKind::Phrase(phrase) => {
                    self.phrase(phrase, "phrase");
                    if phrase.span != span {
                        self.problem("phrase", span, "isn't the phrase's span");
                    }
                }
            }
        }
    }
}

/// The repository root.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the repository root exists")
}

/// Every `.md` file under `dir`, recursively.
pub fn markdown_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(markdown_files(&path));
        } else if path.extension().is_some_and(|e| e == "md") {
            out.push(path);
        }
    }
    out.sort();
    out
}
