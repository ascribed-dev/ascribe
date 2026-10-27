//! `template-placeholder-left`: what a draft of a type with a template still
//! has of the template's placeholders, never filled in, which is
//! `template-placeholder-published`, an error, in a page that publishes
//! (#1005); and
//! `template-section-empty`: a section the template gives only memos that
//! the page still leaves empty.
//!
//! The template and the page are each read as text, block by block: a
//! paragraph, a heading, or a table's cell is one piece of text, with
//! emphasis, links, and escapes reduced to the text they show and every run
//! of white space to one space, so how a line was wrapped doesn't matter. A
//! placeholder is a run of that text that a brace escaped in the template's
//! source opens and the next closing brace ends (README decisions 17 and 20
//! of the page templates plan): one that doesn't close in its block runs to
//! the block's end, takes each block after it whole, and ends in the block
//! with its closing brace. A page has a placeholder when it has each of its
//! pieces, in order, exactly, and its opening brace is escaped there too:
//! a phrase of the same name, or the placeholder quoted in code, is the
//! page's own text. A frontmatter field is left when its value is the
//! template's. A template's instructions to the writer are memos, which
//! aren't blocks with text, so a page that keeps them isn't reported: what
//! to do with a memo is the writer's to decide.
//!
//! A section of the template that has nothing under its heading, before the
//! next heading, but memos and directives bound to its heading, such as
//! `@id`, has no placeholder to leave, so a page that leaves it unfilled
//! builds a heading with nothing under it. A page's section with the same
//! heading, at the same level and with the same text, is empty when nothing
//! in its whole extent, its subsections included, is more than a memo, a
//! directive bound to a heading, or a subsection's heading: a `##` heading
//! followed directly by a `###` section with text in it is filled (decision
//! #558).
//!
//! The template is read once a project, through its [`FileSystem`]
//! (`Project::template`), and a template that can't be read is the content
//! model's error (`model-type-template`), so here it's only not looked at.
//!
//! [`FileSystem`]: ascribe_resolve::FileSystem

use ascribe_core::{DiagnosticSlug, Issue, Location, Span, diagnostics};
use ascribe_model::{ContentModel, TYPE_KEY};
use ascribe_resolve::DRAFT;
use ascribe_syntax::{Block, BlockKind, Bound, Heading, Inline, InlineKind, ParseOptions, parse};
use serde_yaml_ng::Value;

use super::Ctx;
use crate::yaml::YamlIndex;

/// The most words of a placeholder or a value a message quotes.
const OPENING_WORDS: usize = 8;

/// The directive whose blocks are memos.
const MEMO: &str = "memo";

/// A template, as the check compares a page with it.
#[derive(Debug, Default)]
pub(crate) struct Template {
    /// Each field of its frontmatter with a value, in order, apart from
    /// `type`, which a page of the type keeps as the template has it, and
    /// `draft`, which says whether the page publishes, not what it says.
    fields: Vec<(String, Value)>,
    /// Each distinct placeholder, as its pieces, in the order they're first
    /// found.
    placeholders: Vec<Vec<String>>,
    /// Each section with only memos under its heading, as the heading's
    /// level and text, in order.
    memo_sections: Vec<(u8, String)>,
}

impl Template {
    /// Reads `text`, a template, as the project's pages are read.
    pub(crate) fn read(text: &str, model: &ContentModel) -> Template {
        let options = ParseOptions::new(model.directive_schemas())
            .with_note_types(model.notes.iter().map(|n| n.name.clone()).collect());
        let doc = parse(text, &options);
        let mut template = Template::default();
        if let Some(fm) = &doc.frontmatter
            && let Some(content) = text.get(fm.content.range())
            && let Ok(Value::Mapping(map)) = serde_yaml_ng::from_str::<Value>(content)
        {
            for (key, value) in map {
                let filled = match &value {
                    Value::Null => false,
                    Value::String(s) => !s.trim().is_empty(),
                    _ => true,
                };
                if let (Value::String(key), true) = (key, filled)
                    && key != TYPE_KEY
                    && key != DRAFT
                {
                    template.fields.push((key, value));
                }
            }
        }
        let mut units = Vec::new();
        collect(text, &doc.blocks, &mut units);
        let mut open: Option<Vec<String>> = None;
        for unit in &units {
            let mut at = 0;
            loop {
                if let Some(pieces) = &mut open {
                    let Some(close) = unit.find_char('}', at) else {
                        if unit.len() > at {
                            pieces.push(unit.slice(at, unit.len()));
                        }
                        break;
                    };
                    pieces.push(unit.slice(at, close + 1));
                    at = close + 1;
                    let pieces = open.take().unwrap_or_default();
                    template.add_placeholder(pieces);
                    continue;
                }
                let Some(start) = (at..unit.len()).find(|&i| unit.escaped[i]) else {
                    break;
                };
                match unit.find_char('}', start + 1) {
                    Some(close) => {
                        template.add_placeholder(vec![unit.slice(start, close + 1)]);
                        at = close + 1;
                    }
                    None => {
                        open = Some(vec![unit.slice(start, unit.len())]);
                        break;
                    }
                }
            }
        }
        // One that never closes runs to the end of the template.
        if let Some(pieces) = open {
            template.add_placeholder(pieces);
        }
        for (at, block) in doc.blocks.iter().enumerate() {
            let BlockKind::Heading(heading) = &block.kind else {
                continue;
            };
            let own: Vec<&Block> = doc.blocks[at + 1..]
                .iter()
                .take_while(|b| !matches!(b.kind, BlockKind::Heading(_)))
                .collect();
            let key = (heading.level, heading_text(text, heading));
            if own.iter().any(|b| is_memo(b))
                && own.iter().all(|b| is_nothing(b))
                && !template.memo_sections.contains(&key)
            {
                template.memo_sections.push(key);
            }
        }
        template
    }

    fn add_placeholder(&mut self, pieces: Vec<String>) {
        if !self.placeholders.contains(&pieces) {
            self.placeholders.push(pieces);
        }
    }
}

/// One block's text: each character shown, with where it is in the source
/// and whether it's a brace escaped there.
#[derive(Debug, Default)]
struct Unit {
    chars: Vec<char>,
    offsets: Vec<usize>,
    escaped: Vec<bool>,
}

impl Unit {
    fn len(&self) -> usize {
        self.chars.len()
    }

    fn slice(&self, start: usize, end: usize) -> String {
        self.chars[start..end].iter().collect()
    }

    fn find_char(&self, c: char, from: usize) -> Option<usize> {
        (from..self.len()).find(|&i| self.chars[i] == c)
    }

    /// Where `needle` is in this unit, from character `from`, as a
    /// character index.
    fn find(&self, needle: &[char], from: usize) -> Option<usize> {
        if needle.is_empty() || needle.len() > self.len() {
            return None;
        }
        (from..=self.len() - needle.len()).find(|&i| self.chars[i..i + needle.len()] == *needle)
    }

    /// The source from the character at `start` through the one before
    /// `end`.
    fn span(&self, start: usize, end: usize, source: &str) -> Span {
        let from = self.offsets[start];
        let last = self.offsets[end - 1];
        let width = source[last..].chars().next().map_or(0, char::len_utf8);
        Span::new(from, (last + width).max(from))
    }

    /// Ends the unit: white space trimmed at both ends, and each run of it
    /// inside made one space.
    fn finish(raw: Vec<(char, usize, bool)>) -> Unit {
        let mut unit = Unit::default();
        let mut space: Option<usize> = None;
        for (c, offset, escaped) in raw {
            if c.is_whitespace() {
                space.get_or_insert(offset);
                continue;
            }
            if let Some(at) = space.take()
                && !unit.chars.is_empty()
            {
                unit.chars.push(' ');
                unit.offsets.push(at);
                unit.escaped.push(false);
            }
            unit.chars.push(c);
            unit.offsets.push(offset);
            unit.escaped.push(escaped);
        }
        unit
    }
}

/// Every block's text under `blocks`, in source order.
fn collect(source: &str, blocks: &[Block], out: &mut Vec<Unit>) {
    for block in blocks {
        match &block.kind {
            BlockKind::Heading(h) => out.push(Unit::finish(chars(source, &h.inlines))),
            BlockKind::Paragraph(p) => out.push(Unit::finish(chars(source, &p.inlines))),
            BlockKind::BlockQuote(q) => collect(source, &q.children, out),
            BlockKind::List(l) => {
                for item in &l.items {
                    collect(source, &item.children, out);
                }
            }
            BlockKind::Table(t) => {
                for row in &t.rows {
                    for cell in &row.cells {
                        out.push(Unit::finish(chars(source, &cell.inlines)));
                    }
                }
            }
            BlockKind::Container(c) => collect(source, &c.children, out),
            BlockKind::Group(g) => {
                for arm in &g.arms {
                    collect(source, &arm.children, out);
                }
            }
            BlockKind::CodeBlock(_)
            | BlockKind::Directive(_)
            | BlockKind::HtmlBlock(_)
            | BlockKind::ThematicBreak
            | BlockKind::End(_)
            | BlockKind::Title(_) => {}
        }
    }
}

/// The characters `inlines` show, each with its place in `source`.
fn chars(source: &str, inlines: &[Inline]) -> Vec<(char, usize, bool)> {
    let mut out = Vec::new();
    push_inlines(source, inlines, &mut out);
    out
}

fn push_inlines(source: &str, inlines: &[Inline], out: &mut Vec<(char, usize, bool)>) {
    for inline in inlines {
        let start = inline.span.start();
        match &inline.kind {
            InlineKind::Text(value) => push_text(source, inline.span, value, out),
            InlineKind::Code(value) => {
                out.push(('`', start, false));
                out.extend(value.chars().map(|c| (c, start, false)));
                out.push(('`', start, false));
            }
            InlineKind::Html(value) => out.extend(value.chars().map(|c| (c, start, false))),
            InlineKind::SoftBreak | InlineKind::HardBreak => out.push((' ', start, false)),
            InlineKind::Phrase(_) => {
                let text = source.get(inline.span.range()).unwrap_or_default();
                out.extend(text.char_indices().map(|(i, c)| (c, start + i, false)));
            }
            InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                push_inlines(source, children, out);
            }
            InlineKind::Link(link) => push_inlines(source, &link.children, out),
            InlineKind::Image(image) => push_inlines(source, &image.children, out),
        }
    }
}

/// A text node's characters, from its decoded `value`, each matched with
/// the source it was written as: itself, a backslash and itself, or an
/// entity.
fn push_text(source: &str, span: Span, value: &str, out: &mut Vec<(char, usize, bool)>) {
    let written = source.get(span.range()).unwrap_or_default();
    let mut at = 0;
    for c in value.chars() {
        let rest = &written[at.min(written.len())..];
        let offset = span.start() + at.min(written.len());
        let mut after_backslash = rest.chars();
        if c.is_ascii_punctuation()
            && after_backslash.next() == Some('\\')
            && after_backslash.next() == Some(c)
        {
            out.push((c, offset, c == '{'));
            at += 1 + c.len_utf8();
        } else if rest.starts_with(c) {
            out.push((c, offset, false));
            at += c.len_utf8();
        } else if rest.starts_with('&')
            && let Some(end) = rest.find(';')
        {
            out.push((c, offset, false));
            at += end + 1;
        } else {
            out.push((c, offset, false));
        }
    }
}

/// A heading's text, as a unit reads it.
fn heading_text(source: &str, heading: &Heading) -> String {
    Unit::finish(chars(source, &heading.inlines))
        .chars
        .iter()
        .collect()
}

/// Whether `block` is a memo, a line or a container.
fn is_memo(block: &Block) -> bool {
    match &block.kind {
        BlockKind::Directive(line) => line.name == MEMO,
        BlockKind::Container(container) => container.opener.name == MEMO,
        _ => false,
    }
}

/// Whether `block` gives a section nothing a reader sees: a memo, or a
/// directive bound to the section's heading, such as `@id`.
fn is_nothing(block: &Block) -> bool {
    is_memo(block)
        || matches!(&block.kind, BlockKind::Directive(_)) && block.binding() == Some(Bound::Heading)
}

/// The first words of `text`, with an ellipsis when there are more.
fn opening(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() <= OPENING_WORDS {
        return words.join(" ");
    }
    format!("{}…", words[..OPENING_WORDS].join(" "))
}

/// A frontmatter value's text, white space made single spaces, for
/// comparing and quoting.
fn value_text(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.split_whitespace().collect::<Vec<_>>().join(" ")),
        Value::Bool(b) => Some(b.to_string()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

impl Ctx<'_> {
    /// The diagnostic for what the page still has of its template's
    /// placeholders: advice in a draft, which no build publishes, so a page
    /// being written isn't covered in errors, and an error in a page that
    /// publishes, whose readers would read the template's words.
    fn placeholder_slug(&self) -> DiagnosticSlug {
        if self.draft {
            diagnostics::TEMPLATE_PLACEHOLDER_LEFT
        } else {
            diagnostics::TEMPLATE_PLACEHOLDER_PUBLISHED
        }
    }

    /// The template of the page's type, when it has one and `check` is on.
    fn page_template(&self, check: DiagnosticSlug) -> Option<std::sync::Arc<Template>> {
        if self.model.checks.is_off(check) || self.model.is_fragment(self.file.path.as_str()) {
            return None;
        }
        match self.page_type() {
            Some(t) if t.template.is_some() => self.project.template(&t.name),
            _ => None,
        }
    }

    /// The frontmatter fields whose values are still the template's.
    pub(super) fn check_template_fields(&mut self, value: &Value, index: &YamlIndex) {
        let slug = self.placeholder_slug();
        let Some(template) = self.page_template(slug) else {
            return;
        };
        for (field, expected) in &template.fields {
            let Some(found) = value.get(field.as_str()) else {
                continue;
            };
            let same = match (value_text(expected), value_text(found)) {
                (Some(a), Some(b)) => a == b,
                _ => expected == found,
            };
            let Some(node) = index.get(field).filter(|_| same) else {
                continue;
            };
            let shown = value_text(found).map_or_else(String::new, |t| opening(&t));
            let issue = Issue::new(slug, Location::new(self.id, node.value))
                .with_variant("field")
                .with_arg("field", field.clone())
                .with_arg("value", shown);
            self.report(issue);
        }
    }

    /// The template's placeholders the page's body still has.
    pub(super) fn check_template_body(&mut self, blocks: &[Block]) {
        self.check_template_sections(blocks);
        let slug = self.placeholder_slug();
        let Some(template) = self.page_template(slug) else {
            return;
        };
        let source = self.file.text.clone();
        let mut units = Vec::new();
        collect(&source, blocks, &mut units);
        for pieces in &template.placeholders {
            let pieces: Vec<Vec<char>> = pieces.iter().map(|p| p.chars().collect()).collect();
            for Occurrence { unit, start, .. } in occurrences(&units, &pieces) {
                let unit = &units[unit];
                let end = start + pieces[0].len();
                let text = unit.slice(start, end);
                let issue = Issue::new(slug, self.location(unit.span(start, end, &source)))
                    .with_arg("placeholder", opening(&text));
                self.report(issue);
            }
        }
    }
}

impl Ctx<'_> {
    /// The template's sections of only memos that the page has, with the
    /// same heading, and leaves empty.
    fn check_template_sections(&mut self, blocks: &[Block]) {
        let Some(template) = self.page_template(diagnostics::TEMPLATE_SECTION_EMPTY) else {
            return;
        };
        if template.memo_sections.is_empty() {
            return;
        }
        let source = self.file.text.clone();
        for (at, block) in blocks.iter().enumerate() {
            let BlockKind::Heading(heading) = &block.kind else {
                continue;
            };
            let text = heading_text(&source, heading);
            if !template
                .memo_sections
                .contains(&(heading.level, text.clone()))
            {
                continue;
            }
            let empty = blocks[at + 1..]
                .iter()
                .take_while(|b| match &b.kind {
                    BlockKind::Heading(h) => h.level > heading.level,
                    _ => true,
                })
                .all(|b| matches!(b.kind, BlockKind::Heading(_)) || is_nothing(b));
            if empty {
                let issue = Issue::new(
                    diagnostics::TEMPLATE_SECTION_EMPTY,
                    self.location(heading.content),
                )
                .with_arg("heading", opening(&text));
                self.report(issue);
            }
        }
    }
}

/// Where `text`, a page whose type's template is `template`, still has
/// what the template left to fill in, in source order: each frontmatter
/// field whose value is the template's, inside its quotes when it has them,
/// and each placeholder of the body, from its first piece to its last, so
/// one that runs across blocks is one place. They're found as the check
/// finds them; a memo isn't among them, since it's read, not filled in.
pub(crate) fn places(template: &Template, text: &str, model: &ContentModel) -> Vec<Span> {
    let options = ParseOptions::new(model.directive_schemas())
        .with_note_types(model.notes.iter().map(|n| n.name.clone()).collect());
    let doc = parse(text, &options);
    let mut out = Vec::new();
    if let Some(fm) = &doc.frontmatter
        && let Some(content) = text.get(fm.content.range())
        && let Ok(value) = serde_yaml_ng::from_str::<Value>(content)
    {
        let index = YamlIndex::build(content, fm.content.start());
        for (field, expected) in &template.fields {
            let Some(found) = value.get(field.as_str()) else {
                continue;
            };
            let same = match (value_text(expected), value_text(found)) {
                (Some(a), Some(b)) => a == b,
                _ => expected == found,
            };
            if let Some(node) = index.get(field).filter(|_| same) {
                out.push(unquoted(text, node.value));
            }
        }
    }
    let mut units = Vec::new();
    collect(text, &doc.blocks, &mut units);
    for pieces in &template.placeholders {
        let pieces: Vec<Vec<char>> = pieces.iter().map(|p| p.chars().collect()).collect();
        let Some(last) = pieces.last() else {
            continue;
        };
        for found in occurrences(&units, &pieces) {
            let first = units[found.unit].span(found.start, found.start + pieces[0].len(), text);
            let end = found.end_start + last.len();
            let last = units[found.end_unit].span(found.end_start, end, text);
            out.push(Span::new(first.start(), last.end().max(first.start())));
        }
    }
    out.sort_by_key(|s| (s.start(), s.end()));
    out.dedup();
    out
}

/// A quoted scalar's span without its quotes.
fn unquoted(text: &str, span: Span) -> Span {
    let written = text.get(span.range()).unwrap_or_default();
    let quoted = written.len() >= 2
        && ((written.starts_with('"') && written.ends_with('"'))
            || (written.starts_with('\'') && written.ends_with('\'')));
    if quoted {
        Span::new(span.start() + 1, span.end() - 1)
    } else {
        span
    }
}

/// Where a page has a placeholder: the unit and the character its first
/// piece starts at, and the unit and the character its last piece starts
/// at, the same for a placeholder of one piece.
struct Occurrence {
    unit: usize,
    start: usize,
    end_unit: usize,
    end_start: usize,
}

/// Where the page's `units` have every piece of a placeholder, in order,
/// for each time. A placeholder of one piece is found inside a block; one of
/// several has its first piece in one block and each piece after it in a
/// later block.
fn occurrences(units: &[Unit], pieces: &[Vec<char>]) -> Vec<Occurrence> {
    let mut found = Vec::new();
    let Some((first, rest)) = pieces.split_first() else {
        return found;
    };
    let (mut unit, mut from) = (0, 0);
    while unit < units.len() {
        let Some(start) = units[unit].find(first, from) else {
            unit += 1;
            from = 0;
            continue;
        };
        // Only a brace escaped in the page's source opens the template's
        // placeholder: a phrase of the same name is the page's own, and so
        // is text in code, which keeps its backslash.
        if !units[unit].escaped[start] {
            from = start + 1;
            continue;
        }
        // Each later piece, in a later block than the one before it.
        let mut at = unit;
        let mut at_start = start;
        let whole = rest.iter().all(|piece| {
            let next = (at + 1..units.len()).find_map(|u| units[u].find(piece, 0).map(|s| (u, s)));
            next.inspect(|&(u, s)| (at, at_start) = (u, s)).is_some()
        });
        if whole {
            found.push(Occurrence {
                unit,
                start,
                end_unit: at,
                end_start: at_start,
            });
        }
        if rest.is_empty() || !whole {
            from = start + first.len();
        } else {
            unit = at + 1;
            from = 0;
        }
    }
    found
}
