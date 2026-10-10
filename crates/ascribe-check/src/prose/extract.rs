//! A page's prose, as Markdown Vale's own reader understands, with a map from
//! each range of it back to the source.
//!
//! What's given: headings, paragraphs, list items, table cells, block
//! quotes' contents, directives' text primaries and titles, and HTML
//! comments (so Vale's own comments, `<!-- vale off -->`, still work), each
//! as a block of its own. Phrases are replaced by their values. Directive
//! lines without a text primary, end lines, attribute blocks, frontmatter,
//! link reference definitions, code blocks, and raw HTML other than comments
//! are left out, so nothing in them is reported. Every arm of a variant
//! group is given, since the source is checked, not a build.
//!
//! Inline markup is copied as written: emphasis, code spans, escapes, and
//! autolinks are Vale's to read. A link or image is given as its text with an
//! empty destination (`[text]()`), so its address, which may be a reference
//! whose definition isn't given, is never read as prose.

use ascribe_core::Span;
use ascribe_syntax::{
    Block, BlockKind, DirectiveLine, Inline, InlineKind, LinkForm, ParsedDocument, PrimaryValue,
    TableRow,
};

/// A page's prose, and where each part of it comes from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prose {
    /// The Markdown given to Vale.
    pub text: String,
    /// The parts of `text`, in order and touching: each range of `text` is
    /// in exactly one.
    pieces: Vec<Piece>,
}

/// A range of [`Prose::text`] and the source it stands for.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Piece {
    /// The range of the prose.
    start: usize,
    end: usize,
    /// The source it comes from.
    source: Span,
    kind: Kind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Kind {
    /// The source text itself, byte for byte.
    Copied,
    /// Text that stands for the source span as a whole: a heading's marker,
    /// a line break, a link's `]()`.
    Made,
    /// A phrase's value, standing for its `{key}`.
    Phrase(String),
}

/// Where a range of the prose is in the source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Located {
    /// The source range.
    pub span: Span,
    /// The key of the phrase whose text the range is in, if it's in one.
    pub phrase: Option<String>,
    /// Whether the source range holds exactly the text of the prose range,
    /// so an edit of one is an edit of the other.
    pub exact: bool,
}

impl Prose {
    /// The prose of a parsed page. `phrase` gives a phrase's value by its
    /// key; an undeclared phrase is given as written.
    pub fn of(
        source: &str,
        doc: &ParsedDocument,
        phrase: &dyn Fn(&str) -> Option<String>,
    ) -> Prose {
        let mut w = Writer {
            source,
            phrase,
            text: String::new(),
            pieces: Vec::new(),
            indent: false,
            in_table: false,
        };
        w.blocks(&doc.blocks);
        Prose {
            text: w.text,
            pieces: w.pieces,
        }
    }

    /// Where the prose's `start..end` (byte offsets, on character
    /// boundaries) is in the source. `None` for an empty prose.
    pub fn locate(&self, start: usize, end: usize, source: &str) -> Option<Located> {
        let end = end.max(start);
        let first = self.piece_at(start)?;
        let last = if end > start {
            self.piece_at(end - 1)?
        } else {
            first
        };
        let from = match first.kind {
            Kind::Copied => first.source.start() + (start - first.start),
            _ => first.source.start(),
        };
        let to = match last.kind {
            Kind::Copied => last.source.start() + (end - last.start),
            _ => last.source.end(),
        };
        let to = to.max(from);
        let phrase = self
            .pieces
            .iter()
            .filter(|p| p.end > start && p.start < end.max(start + 1))
            .find_map(|p| match &p.kind {
                Kind::Phrase(key) => Some(key.clone()),
                _ => None,
            });
        let exact = phrase.is_none()
            && source.get(from..to).is_some()
            && source.get(from..to) == self.text.get(start..end);
        Some(Located {
            span: Span::new(from, to),
            phrase,
            exact,
        })
    }

    fn piece_at(&self, offset: usize) -> Option<&Piece> {
        let i = self.pieces.partition_point(|p| p.end <= offset);
        self.pieces.get(i).or_else(|| self.pieces.last())
    }
}

/// Builds the prose.
struct Writer<'a> {
    source: &'a str,
    phrase: &'a dyn Fn(&str) -> Option<String>,
    text: String,
    pieces: Vec<Piece>,
    /// Whether a line break in the current block is followed by a list
    /// item's indentation.
    indent: bool,
    /// Whether the inlines are a table cell's, where a pipe would end the
    /// cell.
    in_table: bool,
}

impl Writer<'_> {
    fn push(&mut self, text: &str, source: Span, kind: Kind) {
        if text.is_empty() {
            return;
        }
        let start = self.text.len();
        self.text.push_str(text);
        self.pieces.push(Piece {
            start,
            end: self.text.len(),
            source,
            kind,
        });
    }

    /// Copies the source of `span`, which must be on one line.
    fn copy(&mut self, span: Span) {
        let Some(text) = self.source.get(span.range()) else {
            return;
        };
        if text.contains(['\n', '\r']) {
            // A span that crosses lines (raw inline HTML, say) is copied a
            // line at a time, without the container prefixes after the
            // first.
            let mut at = span.start();
            for (n, line) in text.split_inclusive('\n').enumerate() {
                let body = line.trim_end_matches(['\n', '\r']);
                let lead = if n == 0 {
                    0
                } else {
                    body.len() - strip_prefix(body).len()
                };
                let piece = Span::new(at + lead, at + body.len());
                if let Some(t) = self.source.get(piece.range()) {
                    let t = t.to_owned();
                    self.push(&t, piece, Kind::Copied);
                }
                if line.len() > body.len() {
                    self.line_break(Span::new(at + body.len(), at + line.len()));
                }
                at += line.len();
            }
            return;
        }
        let text = text.to_owned();
        self.push(&text, span, Kind::Copied);
    }

    fn made(&mut self, text: &str, source: Span) {
        self.push(text, source, Kind::Made);
    }

    fn line_break(&mut self, source: Span) {
        let text = if self.indent { "\n  " } else { "\n" };
        self.made(text, source);
    }

    /// Ends a block: a blank line after it.
    fn end_block(&mut self, at: usize) {
        if !self.text.is_empty() && !self.text.ends_with("\n\n") {
            let ending = if self.text.ends_with('\n') {
                "\n"
            } else {
                "\n\n"
            };
            self.made(ending, Span::new(at, at));
        }
        self.indent = false;
    }

    fn blocks(&mut self, blocks: &[Block]) {
        for block in blocks {
            self.block(block);
        }
    }

    fn block(&mut self, block: &Block) {
        match &block.kind {
            BlockKind::Heading(h) => {
                let marker = "#".repeat(usize::from(h.level.clamp(1, 6)));
                self.made(
                    &format!("{marker} "),
                    Span::new(block.span.start(), h.content.start()),
                );
                self.inlines(&h.inlines);
                self.end_block(block.span.end());
            }
            BlockKind::Paragraph(p) => self.paragraph(&p.inlines, block.span.end()),
            BlockKind::BlockQuote(q) => self.blocks(&q.children),
            BlockKind::List(list) => {
                for item in &list.items {
                    let mut children = item.children.iter();
                    match children.next() {
                        Some(Block {
                            span,
                            kind: BlockKind::Paragraph(p),
                        }) => {
                            self.made("- ", item.marker);
                            self.indent = true;
                            self.inlines(&p.inlines);
                            self.end_block(span.end());
                        }
                        Some(first) => self.block(first),
                        None => {}
                    }
                    for child in children {
                        self.block(child);
                    }
                }
            }
            BlockKind::HtmlBlock(html) => {
                if html.literal.trim_start().starts_with("<!--") {
                    self.copy(block.span);
                    self.end_block(block.span.end());
                }
            }
            BlockKind::Table(table) => {
                let columns = table.alignments.len().max(1);
                for row in &table.rows {
                    self.row(row);
                    if row.header {
                        let delimiter = format!("|{}\n", "---|".repeat(columns));
                        self.made(&delimiter, Span::new(row.span.end(), row.span.end()));
                    }
                }
                self.end_block(block.span.end());
            }
            BlockKind::Directive(line) => self.directive(line, block.span.end()),
            BlockKind::Container(c) => {
                self.directive(&c.opener, c.opener.span.end());
                self.blocks(&c.children);
            }
            BlockKind::Group(g) => {
                for arm in &g.arms {
                    self.directive(&arm.opener, arm.opener.span.end());
                    self.blocks(&arm.children);
                }
            }
            BlockKind::Title(t) => self.paragraph(&t.inlines, t.span.end()),
            BlockKind::CodeBlock(_) | BlockKind::ThematicBreak | BlockKind::End(_) => {}
        }
    }

    fn paragraph(&mut self, inlines: &[Inline], end: usize) {
        self.inlines(inlines);
        self.end_block(end);
    }

    /// A directive's title, and its primary when that's text.
    fn directive(&mut self, line: &DirectiveLine, end: usize) {
        if let Some(title) = &line.title {
            self.paragraph(&title.inlines, title.span.end());
        }
        if let Some(PrimaryValue::Text(primary)) = &line.primary {
            self.paragraph(&primary.inlines, end);
        }
    }

    fn row(&mut self, row: &TableRow) {
        self.made("|", Span::new(row.span.start(), row.span.start()));
        for cell in &row.cells {
            self.made(" ", Span::new(cell.span.start(), cell.span.start()));
            self.in_table = true;
            self.inlines(&cell.inlines);
            self.in_table = false;
            self.made(" |", Span::new(cell.span.end(), cell.span.end()));
        }
        self.made("\n", Span::new(row.span.end(), row.span.end()));
    }

    fn inlines(&mut self, inlines: &[Inline]) {
        for inline in inlines {
            self.inline(inline);
        }
    }

    fn inline(&mut self, inline: &Inline) {
        let span = inline.span;
        match &inline.kind {
            InlineKind::Text(_) | InlineKind::Code(_) | InlineKind::Html(_) => self.copy(span),
            InlineKind::SoftBreak | InlineKind::HardBreak => self.line_break(span),
            InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                match (children.first(), children.last()) {
                    (Some(first), Some(last)) => {
                        self.copy(Span::new(span.start(), first.span.start()));
                        self.inlines(children);
                        self.copy(Span::new(last.span.end(), span.end()));
                    }
                    _ => self.copy(span),
                }
            }
            InlineKind::Link(link) if link.form == LinkForm::Autolink => self.copy(span),
            // A link without text takes its target's title when it's built,
            // which isn't known here: it reads as a word made of the target,
            // so the words around it aren't taken for neighbors.
            InlineKind::Link(link) if link.children.is_empty() => {
                self.made(&format!("[{}]()", stand_in(&link.destination)), span);
            }
            InlineKind::Link(link) => self.bracketed("[", span, &link.children),
            InlineKind::Image(image) => self.bracketed("![", span, &image.children),
            InlineKind::Phrase(phrase) => match (self.phrase)(&phrase.key) {
                Some(value) => {
                    let value = escape(&value, self.in_table);
                    self.push(&value, phrase.span, Kind::Phrase(phrase.key.clone()));
                }
                None => self.copy(span),
            },
        }
    }

    /// A link's or image's text, as `[text]()` or `![text]()`.
    fn bracketed(&mut self, open: &str, span: Span, children: &[Inline]) {
        let open_end = (span.start() + open.len()).min(span.end());
        self.made(open, Span::new(span.start(), open_end));
        self.inlines(children);
        let tail = children.last().map_or(open_end, |c| c.span.end());
        self.made("]()", Span::new(tail.max(open_end), span.end()));
    }
}

/// What an empty link's text stands in for: its target's file name without
/// the extension, or its fragment, or `link`.
fn stand_in(destination: &str) -> String {
    let (path, fragment) = destination.split_once('#').unwrap_or((destination, ""));
    let file = path.rsplit('/').next().unwrap_or_default();
    let stem = file.split('.').next().unwrap_or_default();
    let word = if stem.is_empty() { fragment } else { stem };
    let word: String = word
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect();
    if word.is_empty() {
        "link".to_owned()
    } else {
        word
    }
}

/// A phrase's value as Markdown text: the characters that would start markup
/// are escaped, and in a table cell a pipe too, so it reads as the literal
/// text it is.
fn escape(value: &str, in_table: bool) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        if matches!(c, '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '>') || (in_table && c == '|') {
            out.push('\\');
        }
        match c {
            '\n' | '\r' => out.push(' '),
            c => out.push(c),
        }
    }
    out
}

/// A continuation line without its container prefix: leading spaces and
/// tabs, and block quote markers.
fn strip_prefix(line: &str) -> &str {
    let mut rest = line;
    loop {
        let trimmed = rest.trim_start_matches([' ', '\t']);
        match trimmed.strip_prefix('>') {
            Some(after) => rest = after,
            None => return trimmed,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use ascribe_syntax::{ParseOptions, parse};
    use proptest::prelude::*;

    fn prose(source: &str) -> Prose {
        let doc = parse(source, &ParseOptions::default());
        let phrase = |key: &str| match key {
            "product" => Some("Acme Cloud".to_owned()),
            "pipe" => Some("a | b *c*".to_owned()),
            _ => None,
        };
        Prose::of(source, &doc, &phrase)
    }

    /// Every character of the prose maps to a range of the source; the
    /// copied ones to exactly their own text.
    fn every_character_maps(source: &str) {
        let p = prose(source);
        for (i, c) in p.text.char_indices() {
            let at = p
                .locate(i, i + c.len_utf8(), source)
                .unwrap_or_else(|| panic!("{i} of {:?} isn't placed", p.text));
            assert!(at.span.end() <= source.len(), "{at:?} in {source:?}");
            assert!(source.is_char_boundary(at.span.start()));
            assert!(source.is_char_boundary(at.span.end()));
            if at.exact {
                assert_eq!(&source[at.span.range()], &p.text[i..i + c.len_utf8()]);
            }
        }
    }

    /// Where the first match of `needle` in the prose is in the source.
    fn place(source: &str, needle: &str) -> Located {
        let p = prose(source);
        let i = p
            .text
            .find(needle)
            .unwrap_or_else(|| panic!("{needle:?} not in {:?}", p.text));
        p.locate(i, i + needle.len(), source).unwrap()
    }

    #[test]
    fn headings_paragraphs_and_lists_are_given() {
        let source = "# Title\n\nSome *text* here,\nover two lines.\n\n- An item\n  that wraps.\n- Another.\n";
        assert_eq!(
            prose(source).text,
            "# Title\n\nSome *text* here,\nover two lines.\n\n- An item\n  that wraps.\n\n- Another.\n\n"
        );
        every_character_maps(source);
        let at = place(source, "wraps");
        assert_eq!(&source[at.span.range()], "wraps");
        assert!(at.exact);
    }

    #[test]
    fn a_phrase_is_its_value_at_its_key() {
        let source = "Try {product} today, or {unknown}.\n";
        let p = prose(source);
        assert_eq!(p.text, "Try Acme Cloud today, or {unknown}.\n\n");
        every_character_maps(source);
        let at = place(source, "Cloud");
        assert_eq!(&source[at.span.range()], "{product}");
        assert_eq!(at.phrase.as_deref(), Some("product"));
        assert!(!at.exact);
        let after = place(source, "today");
        assert_eq!(&source[after.span.range()], "today");
    }

    #[test]
    fn directives_inside_list_items_give_only_their_text() {
        let source = "- Step one.\n\n  @note {type=caution}: Back up\n  first.\n\n  @include: part.md\n- Step two.\n";
        let p = prose(source);
        assert!(p.text.contains("Back up\nfirst."), "{:?}", p.text);
        assert!(!p.text.contains("caution"), "{:?}", p.text);
        assert!(!p.text.contains("part.md"), "{:?}", p.text);
        every_character_maps(source);
        let at = place(source, "first");
        assert_eq!(&source[at.span.range()], "first");
    }

    #[test]
    fn a_table_is_given_as_a_table() {
        let source = "| Name | Says |\n|---|---|\n| {pipe} {available=beta} | **Hi** |\n";
        let p = prose(source);
        assert_eq!(
            p.text,
            "| Name | Says |\n|---|---|\n| a \\| b \\*c\\* | **Hi** |\n\n"
        );
        every_character_maps(source);
        let at = place(source, "Hi");
        assert_eq!(&source[at.span.range()], "Hi");
    }

    #[test]
    fn non_ascii_text_maps_exactly() {
        let source = "> Ça va — très *bien*,\n> merci.\n";
        every_character_maps(source);
        let at = place(source, "merci");
        assert_eq!(&source[at.span.range()], "merci");
        let at = place(source, "très");
        assert_eq!(&source[at.span.range()], "très");
    }

    #[test]
    fn containers_titles_and_arms_give_their_text() {
        let source = ".Before you start\n@details:\nInside the details.\n@end\n\n@variant {platform=linux}:\nOn Linux.\n@variant {platform=mac}:\nOn a Mac.\n@end\n\n<!-- vale off -->\n\n```sh\necho hi\n```\n";
        let p = prose(source);
        for part in [
            "Before you start",
            "Inside the details.",
            "On Linux.",
            "On a Mac.",
            "<!-- vale off -->",
        ] {
            assert!(p.text.contains(part), "{part:?} isn't in {:?}", p.text);
        }
        assert!(!p.text.contains("echo"), "{:?}", p.text);
        assert!(!p.text.contains("platform"), "{:?}", p.text);
        every_character_maps(source);
    }

    #[test]
    fn links_and_images_give_their_text_only() {
        let source = "See [the guide](guide.md \"Title\") and [ref][r], ![a diagram](d.png){width=50}, <https://example.com>.\n\n[r]: https://example.com/ref\n";
        let p = prose(source);
        assert_eq!(
            p.text,
            "See [the guide]() and [ref](), ![a diagram](), <https://example.com>.\n\n"
        );
        every_character_maps(source);
    }

    #[test]
    fn a_link_without_text_reads_as_a_word_made_of_its_target() {
        let source = "Refer to [](/explore/discover.md) to learn more, or to [](#setup) or []().\n";
        let p = prose(source);
        assert_eq!(
            p.text,
            "Refer to [discover]() to learn more, or to [setup]() or [link]().\n\n"
        );
        every_character_maps(source);
    }

    fn page() -> impl Strategy<Value = String> {
        let word = prop::sample::select(vec![
            "word",
            "teh",
            "café",
            "{product}",
            "{pipe}",
            "*em*",
            "`code`",
            "[link](x.md)",
            "[](x.md)",
            "ünï",
            "—",
            "a\\*b",
        ]);
        let line = prop::collection::vec(word, 1..6).prop_map(|w| w.join(" "));
        let block = prop_oneof![
            line.clone().prop_map(|l| format!("{l}\n")),
            line.clone().prop_map(|l| format!("## {l}\n")),
            (line.clone(), line.clone()).prop_map(|(a, b)| format!("- {a}\n  {b}\n")),
            (line.clone(), line.clone())
                .prop_map(|(a, b)| format!("| {a} | {b} |\n|---|---|\n| {b} | {a} |\n")),
            line.clone().prop_map(|l| format!("@note: {l}\n")),
            line.clone().prop_map(|l| format!("> {l}\n> {l}\n")),
            line.prop_map(|l| format!("- item\n\n  @note: {l}\n")),
        ];
        prop::collection::vec(block, 1..6).prop_map(|b| b.join("\n"))
    }

    proptest! {
        #[test]
        fn every_character_of_any_page_maps(source in page()) {
            every_character_maps(&source);
        }
    }
}
