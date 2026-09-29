//! Ascribe's syntax tree.
//!
//! The tree is Ascribe's own. It doesn't expose comrak's types, so nothing
//! downstream depends on the parser behind it. Every node has a [`Span`]: a
//! range of UTF-8 byte offsets into the source file, counting from the start
//! of the file (frontmatter included). A span covers exactly the source text
//! of its node, from its first character through its last, without the line
//! ending after it. Ascribe nodes keep sub-spans for their parts (name,
//! attribute block, each pair, colon, primary), which the formatter (phase
//! 23) and the language server (phase 15) use to edit, highlight, and
//! complete them.
//!
//! # Shape
//!
//! [`ParsedDocument`] holds the file's [`Block`]s in source order. CommonMark
//! blocks nest as CommonMark nests them (block quotes and list items hold
//! blocks). The Ascribe nodes come in two layers:
//!
//! - **Phase 05 reads lines.** A directive line is a
//!   [`BlockKind::Directive`] and an end line a [`BlockKind::End`], each with
//!   its head parsed into parts. A container opener has [`Form::Container`].
//! - **Phase 06 (the structure pass, run by `parse`) gives them structure.**
//!   A container opener and the blocks up to its end line become a
//!   [`BlockKind::Container`]; a run of openers of a groupable directive
//!   becomes a [`BlockKind::Group`] of [`Arm`]s, closed by one end line. A
//!   title line is attached to the directive below it
//!   ([`DirectiveLine::title`]) and stops being a block of its own. Each
//!   line-form directive records what it binds ([`DirectiveLine::binding`]).
//!   What stays flat: line-form directives, which are siblings of the blocks
//!   around them (a following-block directive and its block are neighbors),
//!   and end lines that close nothing, which are reported.
//! - **Phase 07 fills in inline extensions**: [`InlineKind::Phrase`] for a
//!   `{key}` candidate in text (and [`Link::destination_phrases`],
//!   [`Image::destination_phrases`], and [`CodeBlock::phrases`] where a
//!   candidate isn't an inline node), and [`Image::attributes`] for the
//!   attribute block after an image.
//!
//! Those node kinds exist now, and are documented, so phases 06 and 07 can
//! run in parallel without both editing this file.
//!
//! # Text
//!
//! Nodes that hold text keep both the source span and, where CommonMark
//! decodes it (escapes, entities), the decoded value. To read *raw source*
//! text, slice the source with the span and use [`raw_text`] for spans that
//! cross lines inside block quotes or list items.

use tessera_core::{AttributeBlock, FileId, Issue, Span};

/// A parsed Ascribe source file.
#[derive(Clone, Debug, PartialEq)]
pub struct ParsedDocument {
    /// The file's id, as given in the parse options.
    pub file: FileId,
    /// The whole source, `0..len`.
    pub span: Span,
    /// The YAML frontmatter, if the file begins with one.
    pub frontmatter: Option<Frontmatter>,
    /// The file's blocks, after the frontmatter, in source order.
    pub blocks: Vec<Block>,
    /// Problems found while parsing, in source order: malformed attribute
    /// blocks, primaries the directive doesn't take (or lacks), and
    /// directive-shaped lines with an unknown name.
    pub issues: Vec<Issue>,
    /// The `\{key}` escapes (SPEC §2.3) outside code, in source order: each
    /// is a [`Phrase`] whose span runs from the backslash through the `}`.
    /// They are plain text in the tree, never candidates; they're kept so that
    /// tools can tell an author's escape from text that was never a phrase
    /// (the undeclared-phrase warning is silenced by one).
    pub escaped_phrases: Vec<Phrase>,
    /// The link reference definitions (`[ref]: {api}streaming "title"`), in
    /// source order, wherever they are: in the document, a list item, a block
    /// quote, or a directive container. comrak consumes them, so they aren't
    /// blocks; a paragraph that held only definitions isn't in
    /// [`ParsedDocument::blocks`] at all. Keeping them out of [`BlockKind`]
    /// is deliberate: code that matches on block kinds needn't handle them.
    ///
    /// A definition that isn't first for its label is included; it's never
    /// used (CommonMark: the first definition of a label wins).
    pub definitions: Vec<LinkDefinition>,
}

/// A link reference definition (CommonMark), `[label]: destination "title"`,
/// which can span several lines and sit in a list item or block quote. Its
/// destination is a link destination, so phrases apply in it (SPEC §5.1,
/// resolved Q43), and so do backslash escapes.
///
/// A definition is only recognized where CommonMark recognizes one: at the
/// start of a paragraph. A leading `[label]: /url` in a directive's text
/// primary is text, not a definition (SPEC §3.4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkDefinition {
    /// From the `[` through the last character of the title, or of the
    /// destination when there is no title. It doesn't include the line
    /// ending, or trailing whitespace.
    pub span: Span,
    /// The label between the brackets, without the brackets or the whitespace
    /// around it.
    pub label: Span,
    /// The label's text as written.
    pub label_text: String,
    /// The label as CommonMark compares labels: case folded, with runs of
    /// white space collapsed. A reference link matches the first definition
    /// with the same normalized label.
    pub normalized_label: String,
    /// The destination as written, including its `<` and `>` if it has them.
    pub destination: Span,
    /// The destination with `<>` removed and escapes and entities decoded;
    /// the same value as [`Link::destination`] for a link that uses this
    /// definition.
    pub url: String,
    /// The phrase candidates in the destination as written, in source order
    /// (`[ref]: {api}streaming`), as for [`Link::destination_phrases`]. A
    /// backslash escapes (`\{key}` is text, and is in
    /// [`ParsedDocument::escaped_phrases`]).
    pub destination_phrases: Vec<Phrase>,
    /// The title, if the definition has one.
    pub title: Option<DefinitionTitle>,
}

/// The title of a [`LinkDefinition`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefinitionTitle {
    /// The title as written, including its quotes or parentheses.
    pub span: Span,
    /// The title's text, with the delimiters removed and escapes and entities
    /// decoded.
    pub text: String,
}

/// YAML frontmatter (SPEC §2.1): a leading block delimited by `---` lines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frontmatter {
    /// From the opening `---` through the closing `---`.
    pub span: Span,
    /// The YAML between the delimiter lines, from the start of the first
    /// line after the opening delimiter to the start of the closing one.
    /// Empty when there is nothing between them.
    pub content: Span,
}

/// A block: its span and what it is.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    /// The block's source. For a container block (a block quote, a list,
    /// an item), it runs from the first character of the first line through
    /// the last character of the last line.
    pub span: Span,
    /// What kind of block it is.
    pub kind: BlockKind,
}

/// The kinds of block.
#[derive(Clone, Debug, PartialEq)]
pub enum BlockKind {
    /// An ATX or setext heading.
    Heading(Heading),
    /// A paragraph.
    Paragraph(Paragraph),
    /// A fenced or indented code block.
    CodeBlock(CodeBlock),
    /// A block quote.
    BlockQuote(BlockQuote),
    /// A list.
    List(List),
    /// A raw HTML block.
    HtmlBlock(HtmlBlock),
    /// A thematic break (`***`, `---`, `___`).
    ThematicBreak,
    /// A GFM table.
    Table(Table),
    /// A line-form directive line (SPEC §3.1). After the structure pass, the
    /// openers of containers and arms are inside [`Container`] and [`Arm`]
    /// instead, so a `Directive` block is always a line-form directive.
    Directive(DirectiveLine),
    /// An end line, `@end` (SPEC §3.1) that closes nothing. The structure
    /// pass moves every end line that closes a container into its
    /// [`Container::end`] or [`Group::end`], so one left here is reported
    /// (`end-unmatched` or `end-indent-mismatch`).
    End(EndLine),
    /// A container: an opener, the blocks up to its end line, and the end
    /// line (SPEC §3.5). Its span starts at its title line, if it has one.
    Container(Container),
    /// A group of arms of one groupable directive (SPEC §3.6).
    Group(Group),
    /// Reserved; the structure pass never produces it. A title line
    /// (SPEC §3.7) is attached to the directive below it
    /// ([`DirectiveLine::title`]), and one that can't be attached stays a
    /// [`BlockKind::Paragraph`].
    Title(TitleLine),
}

/// A heading.
#[derive(Clone, Debug, PartialEq)]
pub struct Heading {
    /// 1 to 6.
    pub level: u8,
    /// Whether it's an underlined (setext) heading.
    pub setext: bool,
    /// The heading's text without the markers (`#`s, closing `#`s, the
    /// underline): from its first inline through its last. Empty, at the end
    /// of the heading's opening marker, when there is no text.
    pub content: Span,
    /// The heading's inline content.
    pub inlines: Vec<Inline>,
}

/// A paragraph.
#[derive(Clone, Debug, PartialEq)]
pub struct Paragraph {
    /// The paragraph's inline content.
    pub inlines: Vec<Inline>,
}

/// A code block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeBlock {
    /// Whether the block is fenced.
    pub fenced: bool,
    /// The fence's info string, trimmed. Empty for an indented block or a
    /// fence without one.
    pub info: String,
    /// The info string's span; `None` when `info` is empty.
    pub info_span: Option<Span>,
    /// The block's literal text, as the code it holds (fences and indentation
    /// removed).
    pub literal: String,
    /// The phrase candidates in the block, in source order, when it's a fence
    /// whose info string contains the word `phrases=true` (SPEC §5.1); `None`
    /// for every other code block, where `{key}` is never a phrase. `Some`
    /// with no candidates is a fence that opted in and has none.
    pub phrases: Option<Vec<Phrase>>,
}

/// A block quote.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockQuote {
    /// The blocks in it.
    pub children: Vec<Block>,
}

/// A list.
#[derive(Clone, Debug, PartialEq)]
pub struct List {
    /// Whether the list is ordered.
    pub ordered: bool,
    /// The first number of an ordered list.
    pub start: Option<u64>,
    /// Whether the list is tight (no blank lines between items or blocks).
    pub tight: bool,
    /// The items.
    pub items: Vec<ListItem>,
}

/// A list item.
#[derive(Clone, Debug, PartialEq)]
pub struct ListItem {
    /// From the marker through the end of the item's last line.
    pub span: Span,
    /// The marker: `-`, `+`, `*`, or digits with `.` or `)`.
    pub marker: Span,
    /// The blocks in it.
    pub children: Vec<Block>,
}

/// A raw HTML block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HtmlBlock {
    /// The block's literal text.
    pub literal: String,
}

/// A GFM table. Its rows and cells are kept so that inline content in cells
/// (phrases, links) is reachable; the delimiter row isn't a node, but the
/// alignment it gives each column is kept.
#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    /// The header row, then the body rows.
    pub rows: Vec<TableRow>,
    /// Each column's alignment, from the delimiter row (`:---`, `:---:`,
    /// `---:`), one per column.
    pub alignments: Vec<Alignment>,
}

/// A table column's alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alignment {
    /// `---`: none given.
    None,
    /// `:---`.
    Left,
    /// `:---:`.
    Center,
    /// `---:`.
    Right,
}

/// A table row.
#[derive(Clone, Debug, PartialEq)]
pub struct TableRow {
    /// The row, from its first character through its last, pipes included.
    pub span: Span,
    /// Whether it's the header row.
    pub header: bool,
    /// The row's cells.
    pub cells: Vec<TableCell>,
}

/// A table cell.
#[derive(Clone, Debug, PartialEq)]
pub struct TableCell {
    /// The cell's text between its pipes, including surrounding spaces.
    pub span: Span,
    /// The cell's inline content.
    pub inlines: Vec<Inline>,
}

// ---------------------------------------------------------------------------
// Ascribe lines

/// Whether a directive line opens a container or stands alone (SPEC §3.5).
///
/// The form is decided by the directive line alone: an empty primary after a
/// colon opens a container, and anything else is line form. Whether the
/// directive's schema permits that form is a structure check (phase 06).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Form {
    /// A single line.
    Line,
    /// A colon with nothing after it: opens a container.
    Container,
}

/// A directive line (SPEC §3.1): `@name {attributes}: primary`.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectiveLine {
    /// From the `@` through the last character of the line, or of the last
    /// line of a text primary. Trailing whitespace isn't included.
    pub span: Span,
    /// The keyword, without `@`: a built-in, a project widget, or another
    /// keyword the parse options declare.
    pub name: String,
    /// The `@` and the name.
    pub name_span: Span,
    /// The attribute block, if the line has one, even when it's malformed.
    /// (Issues about it are in [`ParsedDocument::issues`].)
    pub attributes: Option<AttributeBlock>,
    /// Whether the attribute block has its closing `}`. `true` when there is
    /// no block.
    pub attributes_closed: bool,
    /// The `:`, if the line has one.
    pub colon: Option<Span>,
    /// The primary, if the line has one.
    pub primary: Option<PrimaryValue>,
    /// Whether the line opens a container.
    pub form: Form,
    /// Text after the name (and attribute block) that fits no part of the
    /// head, such as the `hello: text` of `@note hello: text`. It's kept, not
    /// dropped, and reported as `directive-extra-text` (SPEC §3.1).
    pub unexpected: Option<Span>,
    /// The title line directly above the directive (SPEC §3.7), when the
    /// directive accepts one. **Set by the structure pass.** A directive
    /// with a title has a [`Block`] span that starts at the title's dot; this
    /// line's own `span` still starts at the `@`.
    pub title: Option<TitleLine>,
    /// What a line-form directive applies to (SPEC §3.8). **Set by the
    /// structure pass**; `None` on a container opener (and on a directive
    /// whose schema has no binding).
    pub binding: Option<Bound>,
}

/// What a line-form directive applies to (SPEC §3.8), as the structure pass
/// worked it out from the directive's schema and its position.
///
/// A [`Bound::Heading`] directive's heading is the nearest [`Heading`] before
/// it in the same list of blocks, and a [`Bound::FollowingBlock`] directive's
/// block is the next sibling that isn't itself a following-block directive
/// (stacked directives all describe the same block). [`crate::bound_heading`] and
/// [`crate::bound_block`] find them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Bound {
    /// The directive's own primary, or nothing (it stands alone).
    Own,
    /// The heading at the top of the section the directive sits in, and that
    /// section.
    Heading,
    /// The next block in the same container.
    FollowingBlock,
    /// Binding failed, and the structure pass reported why: a heading-bound
    /// directive not at the top of a section, or a following-block
    /// directive with no block, or a heading, after it.
    Unbound,
}

/// A directive's primary (SPEC §3.4), by what the directive's schema says.
#[derive(Clone, Debug, PartialEq)]
pub enum PrimaryValue {
    /// An identifier primary: a token that ends at the first whitespace.
    Identifier(IdentifierPrimary),
    /// A text primary: inline content that continues onto following lines.
    Text(TextPrimary),
    /// A line primary: the rest of the line, trimmed, not inline content
    /// (an availability spec).
    Line(LinePrimary),
    /// Text after the colon on a directive whose schema takes no primary.
    /// Kept for the language server; reported as an issue.
    Unexpected(Span),
}

impl PrimaryValue {
    /// The primary's span.
    pub fn span(&self) -> Span {
        match self {
            PrimaryValue::Identifier(p) => p.span,
            PrimaryValue::Text(p) => p.span,
            PrimaryValue::Line(p) => p.span,
            PrimaryValue::Unexpected(span) => *span,
        }
    }
}

/// An identifier primary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentifierPrimary {
    /// The token, up to the first whitespace.
    pub span: Span,
    /// The token's text.
    pub text: String,
    /// Text after the token on the same line, trimmed (`@include: my file.md`
    /// has ` file.md`). Kept, not dropped, and reported as
    /// `directive-extra-text` (SPEC §3.4, resolved Q15 and Q30).
    pub trailing: Option<Span>,
}

/// A text primary.
#[derive(Clone, Debug, PartialEq)]
pub struct TextPrimary {
    /// From the first character of the primary through the last character of
    /// its last line.
    pub span: Span,
    /// The primary's lines, one span each, trimmed of container indentation
    /// and blockquote markers, and of trailing whitespace. The first begins
    /// at the primary's first character.
    pub lines: Vec<Span>,
    /// The inline content.
    pub inlines: Vec<Inline>,
}

/// A line primary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinePrimary {
    /// The rest of the line after the colon, trimmed.
    pub span: Span,
    /// Its text.
    pub text: String,
}

/// An end line (SPEC §3.1): `@end`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndLine {
    /// From the `@` through the `d`.
    pub span: Span,
    /// The same as `span`; kept so end lines and directive lines both have a
    /// `name_span`.
    pub name_span: Span,
    /// Anything after `@end` other than whitespace, such as `: text`. It's
    /// reported as an issue.
    pub extra: Option<Span>,
}

/// A container: an opener, its blocks, and its end line (SPEC §3.5).
///
/// The span of its [`Block`] runs from the opener's title line (or the
/// opener) through the end line, or, when it's unclosed, through its last
/// block.
#[derive(Clone, Debug, PartialEq)]
pub struct Container {
    /// The opener: a directive line that opens a container, and whose title
    /// is in [`DirectiveLine::title`]. Its form is [`Form::Container`], except
    /// for a container-only directive written without its colon, which is
    /// still an opener and is reported (SPEC §3.5, resolved Q16).
    pub opener: DirectiveLine,
    /// The blocks between the opener and the end line.
    pub children: Vec<Block>,
    /// The end line that closes it; `None` when the container is unclosed.
    pub end: Option<EndLine>,
}

/// A group of arms (SPEC §3.6): a run of openers of one groupable directive,
/// closed by one end line. It's one container level for nesting (SPEC §3.10, resolved Q17).
#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    /// The groupable directive's keyword.
    pub name: String,
    /// The arms, in source order. At least one.
    pub arms: Vec<Arm>,
    /// The group's end line; `None` when the group is unclosed. It belongs to
    /// the group, not to its last arm.
    pub end: Option<EndLine>,
}

/// One arm of a group.
#[derive(Clone, Debug, PartialEq)]
pub struct Arm {
    /// From the opener's title line (or the opener) through the last block of
    /// the arm, or through the opener when the arm is empty. It doesn't
    /// include the group's end line.
    pub span: Span,
    /// The opener, a directive line that opens a container.
    pub opener: DirectiveLine,
    /// The title line above the opener, if it has one; the same as
    /// `opener.title`.
    pub title: Option<TitleLine>,
    /// The blocks in the arm.
    pub children: Vec<Block>,
}

/// A title line (SPEC §3.7): `.Title text`, attached to the directive below
/// it as [`DirectiveLine::title`].
#[derive(Clone, Debug, PartialEq)]
pub struct TitleLine {
    /// The whole line, from the `.` through the end of the text.
    pub span: Span,
    /// The `.`.
    pub dot: Span,
    /// The title's text after the `.`, without trailing whitespace.
    pub content: Span,
    /// The title's inline content.
    pub inlines: Vec<Inline>,
}

// ---------------------------------------------------------------------------
// Inlines

/// An inline node.
#[derive(Clone, Debug, PartialEq)]
pub struct Inline {
    /// The node's source, including its markup (`*`, backticks, brackets).
    pub span: Span,
    /// What it is.
    pub kind: InlineKind,
}

/// The kinds of inline.
#[derive(Clone, Debug, PartialEq)]
pub enum InlineKind {
    /// Text. Neighboring text that touches in the source is one node. The
    /// value has escapes and entities decoded (`\@x` is `@x`); the span is
    /// the source.
    Text(String),
    /// A code span. The value is its content, without the backticks.
    Code(String),
    /// A soft line break.
    SoftBreak,
    /// A hard line break.
    HardBreak,
    /// Raw inline HTML.
    Html(String),
    /// Emphasis (`*x*`, `_x_`).
    Emphasis(Vec<Inline>),
    /// Strong emphasis (`**x**`, `__x__`).
    Strong(Vec<Inline>),
    /// A link.
    Link(Link),
    /// An image, in any of CommonMark's forms.
    Image(Image),
    /// A phrase candidate, `{key}` (SPEC §5.1). **Phase 07 fills this in.**
    Phrase(Phrase),
}

/// The form a link or image was written in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LinkForm {
    /// `[text](destination "title")`.
    Inline,
    /// `[text][label]`.
    Full,
    /// `[label][]`.
    Collapsed,
    /// `[label]`.
    Shortcut,
    /// `<https://example.com>`. Links only.
    Autolink,
}

/// A link.
#[derive(Clone, Debug, PartialEq)]
pub struct Link {
    /// How it was written.
    pub form: LinkForm,
    /// The destination, with escapes decoded and any `<>` removed. For a
    /// reference form, the destination of the definition it resolved to.
    pub destination: String,
    /// The title, if any.
    pub title: Option<String>,
    /// For a full reference, the label between its second pair of brackets.
    pub label: Option<Span>,
    /// The phrase candidates in the destination as written, in source order
    /// (`[text]({api}streaming)`, `<https://{host}/status>`). The destination
    /// itself is unchanged. Inline links and autolinks have them; reference
    /// forms don't, since their destination is in a definition, which has
    /// them ([`LinkDefinition::destination_phrases`], SPEC §5.1).
    pub destination_phrases: Vec<Phrase>,
    /// The link text.
    pub children: Vec<Inline>,
}

/// An image.
#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    /// How it was written: `![alt](src)`, `![alt][label]`, `![label][]`, or
    /// `![label]`.
    pub form: LinkForm,
    /// The source, with escapes decoded and any `<>` removed. For a
    /// reference form, the destination of the definition it resolved to.
    pub destination: String,
    /// The title, if any.
    pub title: Option<String>,
    /// For a full reference, the label between its second pair of brackets.
    pub label: Option<Span>,
    /// The alt text between the first pair of brackets, as source.
    pub alt: Span,
    /// The alt text as inline content.
    pub children: Vec<Inline>,
    /// The phrase candidates in an inline image's source, as written, in
    /// source order (`![alt]({assets}a.png)`). The destination itself is
    /// unchanged. Reference forms have none: their destination is in a
    /// definition, which has them ([`LinkDefinition::destination_phrases`],
    /// SPEC §5.1).
    pub destination_phrases: Vec<Phrase>,
    /// The attribute block directly after the image (SPEC §5.3). The image's
    /// span covers it.
    pub attributes: Option<ImageAttributes>,
}

/// The attribute block after an image (SPEC §5.3).
#[derive(Clone, Debug, PartialEq)]
pub struct ImageAttributes {
    /// The parsed block, from `tessera_core::parse_attribute_block`. Its
    /// problems are in [`ParsedDocument::issues`].
    pub block: AttributeBlock,
}

/// A phrase candidate: `{key}` (SPEC §5.1). Whether the key is declared is
/// decided later; the parser records every candidate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phrase {
    /// The key, between the braces.
    pub key: String,
    /// The key's span, without the braces.
    pub key_span: Span,
    /// The whole candidate, braces included.
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Reading text

/// The raw source text of `span`, with the container prefixes of the lines
/// after the first removed: leading spaces and tabs and blockquote `>`
/// markers. Line breaks stay as they are in the source (`\n` for each), and
/// trailing whitespace on each line is removed.
///
/// This is the text of a paragraph, a heading, a title, or a primary as the
/// author wrote it inside its container.
pub fn raw_text(source: &str, span: Span) -> String {
    let text = source.get(span.start()..span.end()).unwrap_or("");
    let mut out = String::new();
    for (n, line) in split_lines(text).enumerate() {
        if n > 0 {
            out.push('\n');
        }
        let line = if n == 0 { line } else { strip_prefix(line) };
        out.push_str(line.trim_end_matches([' ', '\t']));
    }
    out
}

/// Strips container indentation and blockquote markers from the start of a
/// continuation line.
pub(crate) fn strip_prefix(line: &str) -> &str {
    let mut rest = line;
    loop {
        let trimmed = rest.trim_start_matches([' ', '\t']);
        match trimmed.strip_prefix('>') {
            Some(after) => rest = after,
            None => return trimmed,
        }
    }
}

/// Splits on `\n`, `\r\n`, and a lone `\r`, without the endings.
pub(crate) fn split_lines(text: &str) -> impl Iterator<Item = &str> {
    let mut rest = Some(text);
    std::iter::from_fn(move || {
        let r = rest?;
        match r.find(['\n', '\r']) {
            Some(i) => {
                let after = &r[i..];
                let skip = if after.starts_with("\r\n") { 2 } else { 1 };
                rest = Some(&r[i + skip..]);
                Some(&r[..i])
            }
            None => {
                rest = None;
                Some(r)
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_text_strips_container_prefixes() {
        let source = "> a  \n> > b\n>   c";
        assert_eq!(raw_text(source, Span::new(2, source.len())), "a\nb\nc");
        assert_eq!(raw_text("x\r\n  y\rz", Span::new(0, 8)), "x\ny\nz");
        assert_eq!(raw_text("abc", Span::new(1, 2)), "b");
    }

    #[test]
    fn splits_lines_like_commonmark() {
        let lines: Vec<_> = split_lines("a\r\nb\rc\nd").collect();
        assert_eq!(lines, ["a", "b", "c", "d"]);
        assert_eq!(split_lines("").count(), 1);
    }
}
