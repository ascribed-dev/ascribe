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
    c.definitions(&doc.definitions);
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

    /// Every part of every link reference definition is where the source
    /// says: `[label]:` then a destination, then perhaps a title, and the
    /// definition ends with the last of them.
    fn definitions(&mut self, definitions: &[LinkDefinition]) {
        let mut previous_end = 0;
        for d in definitions {
            let what = "definition";
            if !self.valid(d.span, what) {
                continue;
            }
            if d.span.start() < previous_end {
                self.problem(what, d.span, "overlaps or precedes the one before");
            }
            previous_end = d.span.end();
            self.text_starts(d.span, "[", what);
            self.tight(d.span, what);
            // The label sits right inside the brackets, and `]:` follows it.
            self.within(d.label, d.span, "definition label");
            self.tight(d.label, "definition label");
            if self.text(d.label) != d.label_text {
                self.problem("definition label", d.label, "isn't its label_text");
            }
            if d.normalized_label.is_empty() {
                self.problem("definition label", d.label, "has an empty normalized label");
            }
            let between = self
                .source
                .get(d.span.start() + 1..d.label.start())
                .unwrap_or("x");
            if !between.chars().all(|c| c.is_whitespace() || c == '>') {
                self.problem("definition label", d.label, "isn't right after the `[`");
            }
            let after = self.source.get(d.label.end()..).unwrap_or("");
            let after = after.trim_start_matches(char::is_whitespace);
            if !after.starts_with("]:") {
                self.problem("definition label", d.label, "isn't followed by `]:`");
            }
            // The destination follows the colon.
            self.within(d.destination, d.span, "definition destination");
            self.tight(d.destination, "definition destination");
            if d.destination.start() < d.label.end() {
                self.problem(what, d.destination, "starts before the label ends");
            }
            let dest = self.text(d.destination);
            if dest.starts_with('<') != dest.ends_with('>') {
                self.problem(what, d.destination, "has one pointy bracket, not two");
            }
            let between = self
                .source
                .get(d.label.end()..d.destination.start())
                .unwrap_or("]:");
            if !between.trim_start().starts_with("]:")
                || !between[between.find(':').map_or(0, |i| i + 1)..]
                    .chars()
                    .all(|c| c.is_whitespace() || c == '>')
            {
                self.problem(what, d.destination, "isn't right after the `]:`");
            }
            for phrase in &d.destination_phrases {
                self.within(phrase.span, d.destination, "definition destination phrase");
                self.phrase(phrase, "definition destination phrase");
            }
            // A title ends the definition.
            match &d.title {
                Some(title) => {
                    self.within(title.span, d.span, "definition title");
                    if title.span.end() != d.span.end() {
                        self.problem(what, title.span, "isn't at the definition's end");
                    }
                    let text = self.text(title.span);
                    let closes = match text.chars().next() {
                        Some('"') => Some('"'),
                        Some('\'') => Some('\''),
                        Some('(') => Some(')'),
                        _ => None,
                    };
                    if closes.is_none() || text.chars().last() != closes || text.len() < 2 {
                        self.problem("definition title", title.span, "isn't delimited");
                    }
                    if title.span.start() < d.destination.end() {
                        self.problem(what, title.span, "starts before the destination ends");
                    }
                }
                None => {
                    if d.destination.end() != d.span.end() {
                        self.problem(what, d.span, "doesn't end with its destination");
                    }
                }
            }
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
            BlockKind::Container(c) => {
                self.tight(span, "container");
                self.opener(&c.opener, span, span.start(), "container opener");
                self.blocks(&c.children, span, "container child");
                self.closer(c.end.as_ref(), span);
                let last = c
                    .children
                    .last()
                    .map_or(c.opener.span.end(), |b| b.span.end());
                let want_end = c
                    .end
                    .as_ref()
                    .map_or(last.max(c.opener.span.end()), |e| e.span.end());
                if span.end() != want_end {
                    self.problem(
                        "container",
                        span,
                        "doesn't end where its end line or last block does",
                    );
                }
            }
            BlockKind::Group(g) => {
                self.tight(span, "group");
                if g.arms.is_empty() {
                    self.problem("group", span, "has no arms");
                }
                let mut previous_end = 0;
                for arm in &g.arms {
                    self.within(arm.span, span, "arm");
                    self.opener(&arm.opener, arm.span, arm.span.start(), "arm opener");
                    if arm.title != arm.opener.title {
                        self.problem("arm", arm.span, "title differs from the opener's");
                    }
                    self.blocks(&arm.children, arm.span, "arm child");
                    if arm.span.start() < previous_end {
                        self.problem("arm", arm.span, "overlaps the arm before it");
                    }
                    previous_end = arm.span.end();
                    let last = arm.children.last().map_or(0, |b| b.span.end());
                    if arm.span.end() != last.max(arm.opener.span.end()) {
                        self.problem("arm", arm.span, "doesn't end at its last block");
                    }
                }
                self.closer(g.end.as_ref(), span);
                if g.arms.first().map(|a| a.span.start()) != Some(span.start()) {
                    self.problem("group", span, "doesn't start where its first arm does");
                }
                let last = g.arms.last().map_or(0, |a| a.span.end());
                let want_end = g.end.as_ref().map_or(last, |e| e.span.end());
                if span.end() != want_end {
                    self.problem("group", span, "doesn't end where its end line does");
                }
            }
            BlockKind::Title(_) => {
                self.problem("block", span, "a node the structure pass doesn't produce");
            }
        }
    }

    /// The end line of a container or group, when it has one.
    fn closer(&mut self, end: Option<&EndLine>, parent: Span) {
        if let Some(e) = end {
            self.within(e.span, parent, "end line");
            if self.text(e.span) != "@end" {
                self.problem("end line", e.span, "isn't `@end`");
            }
        }
    }

    /// A directive line inside a container or an arm, whose block starts at
    /// `start` (its title line, or the `@`).
    fn opener(&mut self, d: &DirectiveLine, parent: Span, start: usize, what: &str) {
        self.within(d.span, parent, what);
        self.directive_parts(d, d.span);
        self.title(d, start);
    }

    fn title(&mut self, d: &DirectiveLine, start: usize) {
        let Some(t) = &d.title else {
            if start != d.span.start() {
                self.problem(
                    "directive",
                    d.span,
                    "its block starts before it, with no title",
                );
            }
            return;
        };
        if t.span.start() != start || t.span.end() > d.span.start() {
            self.problem(
                "title",
                t.span,
                "isn't at the start of its directive's block",
            );
        }
        self.tight(t.span, "title line");
        self.text_starts(t.dot, ".", "title dot");
        if self.text(t.dot) != "." || t.dot.end() != t.content.start() {
            self.problem("title", t.dot, "isn't the dot before the content");
        }
        self.within(t.content, t.span, "title content");
        if t.content.end() != t.span.end() {
            self.problem("title", t.content, "doesn't end the title line");
        }
        self.inlines(&t.inlines, t.content);
    }

    fn directive(&mut self, d: &DirectiveLine, span: Span) {
        let start = d.title.as_ref().map_or(d.span.start(), |t| t.span.start());
        if span != Span::new(start, d.span.end()) {
            self.problem("directive", d.span, "differs from its block's span");
        }
        self.title(d, start);
        self.directive_parts(d, d.span);
    }

    fn directive_parts(&mut self, d: &DirectiveLine, span: Span) {
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
