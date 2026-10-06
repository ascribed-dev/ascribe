//! Converting comrak's tree to Ascribe's.
//!
//! comrak reports positions as a 1-based line and a 1-based byte column, with
//! the *last* character's position as a node's end. This module turns them
//! into byte offsets with `tessera_core::LineIndex`, and builds every span
//! from them (Ascribe lines' sub-spans come from the head parser instead,
//! anchored at the `@`).

use comrak_tessera::nodes::{LineColumn, Node, NodeLink, NodeValue};
use comrak_tessera::{Arena, Options, parse_document_with_definitions};
use tessera_core::{DirectiveSchema, Issue, LineIndex, Location, Primary, Span, diagnostics};

use crate::head::{Head, parse_head};
use crate::options::ParseOptions;
use crate::tree::*;
use crate::unknown::{directive_shape, is_known, suggest};

pub(crate) fn convert(source: &str, options: &ParseOptions) -> ParsedDocument {
    let mut comrak_options = Options::default();
    comrak_options.extension.tessera = Some(options.keywords());
    comrak_options.extension.table = true;
    comrak_options.extension.front_matter_delimiter = Some("---".to_owned());
    // Keep each escape as a node, so a text's span includes its backslash.
    comrak_options.parse.escaped_char_spans = true;

    let arena = Arena::new();
    let (root, raw_definitions) = parse_document_with_definitions(&arena, source, &comrak_options);
    let mut converter = Converter {
        source,
        index: LineIndex::new(source),
        options,
        issues: Vec::new(),
    };
    let frontmatter = root
        .first_child()
        .and_then(|first| converter.frontmatter(first));
    let blocks = converter.blocks(root);
    let mut blocks = crate::structure::run(source, options, blocks, &mut converter.issues);
    // After the structure pass, so the inline pass sees the final tree.
    let mut definitions: Vec<LinkDefinition> = raw_definitions
        .iter()
        .map(|d| converter.definition(d))
        .collect();
    let escaped_phrases = crate::inline::extend(
        source,
        options.file,
        &mut blocks,
        &mut definitions,
        &mut converter.issues,
    );
    let mut issues = converter.issues;
    issues.sort_by_key(|i| i.location.span.start());
    ParsedDocument {
        file: options.file,
        span: Span::new(0, source.len()),
        frontmatter,
        blocks,
        issues,
        escaped_phrases,
        definitions,
    }
}

struct Converter<'a> {
    source: &'a str,
    index: LineIndex,
    options: &'a ParseOptions,
    issues: Vec<Issue>,
}

impl<'a> Converter<'a> {
    // -- positions ----------------------------------------------------------

    fn line_start(&self, line: usize) -> usize {
        u32::try_from(line.saturating_sub(1))
            .ok()
            .and_then(|l| self.index.line_span(l))
            .map_or(self.source.len(), Span::start)
    }

    /// The offset of the first byte of the character at `pos`.
    fn start_of(&self, pos: LineColumn) -> usize {
        let offset = self.line_start(pos.line) + pos.column.saturating_sub(1);
        self.floor(offset)
    }

    /// The offset just past the character at `pos`.
    fn end_of(&self, pos: LineColumn) -> usize {
        let offset = self.line_start(pos.line) + pos.column;
        self.ceil(offset)
    }

    fn floor(&self, mut offset: usize) -> usize {
        offset = offset.min(self.source.len());
        while !self.source.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    }

    fn ceil(&self, mut offset: usize) -> usize {
        offset = offset.min(self.source.len());
        while !self.source.is_char_boundary(offset) {
            offset += 1;
        }
        offset
    }

    /// A span from comrak's positions, the end being the last byte.
    fn sourcepos_span(&self, pos: comrak_tessera::nodes::Sourcepos) -> Span {
        let start = self.start_of(pos.start);
        Span::new(start, self.end_of(pos.end).max(start))
    }

    /// A link reference definition, as the fork reports it.
    fn definition(&self, d: &comrak_tessera::tessera::LinkDefinition) -> LinkDefinition {
        let label = self.sourcepos_span(d.label);
        LinkDefinition {
            span: self.sourcepos_span(d.sourcepos),
            label,
            label_text: self.text(label).to_owned(),
            normalized_label: d.normalized_label.clone(),
            destination: self.sourcepos_span(d.destination),
            url: d.url.clone(),
            destination_phrases: Vec::new(),
            title: d.title.as_ref().map(|(pos, text)| DefinitionTitle {
                span: self.sourcepos_span(*pos),
                text: text.clone(),
            }),
        }
    }

    /// A node's span exactly as comrak reports it.
    fn raw_span(&self, node: Node<'_>) -> Span {
        let sourcepos = node.data().sourcepos;
        let start = self.start_of(sourcepos.start);
        Span::new(start, self.end_of(sourcepos.end).max(start))
    }

    /// A node's span, without the line ending its end position may include.
    fn span(&self, node: Node<'_>) -> Span {
        let raw = self.raw_span(node);
        let start = raw.start();
        let mut end = raw.end();
        while end > start && matches!(self.source.as_bytes().get(end - 1), Some(b'\n' | b'\r')) {
            end -= 1;
        }
        Span::new(start, end)
    }

    fn text(&self, span: Span) -> &'a str {
        self.source.get(span.start()..span.end()).unwrap_or("")
    }

    fn location(&self, span: Span) -> Location {
        Location::new(self.options.file, span)
    }

    fn report(&mut self, issue: Issue) {
        self.issues.push(issue);
    }

    // -- blocks -------------------------------------------------------------

    fn frontmatter(&self, node: Node<'_>) -> Option<Frontmatter> {
        if !matches!(node.data().value, NodeValue::FrontMatter(_)) {
            return None;
        }
        let span = self.span(node);
        let sourcepos = node.data().sourcepos;
        let open_line_end = self
            .index
            .line_span(u32::try_from(sourcepos.start.line.saturating_sub(1)).ok()?)?
            .end();
        let close_line_start = self.line_start(sourcepos.end.line);
        // The first line after the opening delimiter, through the line before
        // the closing one.
        let content_start = self.after_line_ending(open_line_end).min(close_line_start);
        Some(Frontmatter {
            span,
            content: Span::new(content_start, close_line_start.max(content_start)),
        })
    }

    /// The offset after the line ending that starts at `offset`.
    fn after_line_ending(&self, offset: usize) -> usize {
        let rest = self.source.get(offset..).unwrap_or("");
        if rest.starts_with("\r\n") {
            offset + 2
        } else if rest.starts_with(['\n', '\r']) {
            offset + 1
        } else {
            offset
        }
    }

    fn blocks(&mut self, parent: Node<'_>) -> Vec<Block> {
        let mut blocks = Vec::new();
        for child in parent.children() {
            self.block(child, &mut blocks);
        }
        blocks
    }

    fn block(&mut self, node: Node<'_>, out: &mut Vec<Block>) {
        let mut span = self.span(node);
        let ast = node.data();
        let kind = match &ast.value {
            NodeValue::FrontMatter(_) => return,
            NodeValue::TesseraLine(line) => {
                let (span, kind) = self.tessera_line(node, line);
                out.push(Block { span, kind });
                return;
            }
            NodeValue::Heading(h) => {
                // An ATX heading's end includes its trailing spaces.
                span = self.trimmed(span);
                let mut inlines = self.inlines(node);
                clamp_inlines(&mut inlines, span.end());
                let content = match (inlines.first(), inlines.last()) {
                    (Some(first), Some(last)) => Span::new(first.span.start(), last.span.end()),
                    _ => Span::empty(span.end()),
                };
                BlockKind::Heading(Heading {
                    level: h.level,
                    setext: h.setext,
                    content,
                    inlines,
                })
            }
            NodeValue::Paragraph => {
                // The last line's trailing spaces aren't content.
                span = self.trimmed(span);
                self.scan_unknown(span, false);
                let mut inlines = self.inlines(node);
                clamp_inlines(&mut inlines, span.end());
                BlockKind::Paragraph(Paragraph { inlines })
            }
            NodeValue::CodeBlock(code) => {
                let info = code.info.trim().to_owned();
                let info_span = (code.fenced && !info.is_empty())
                    .then(|| self.info_span(span))
                    .flatten();
                BlockKind::CodeBlock(CodeBlock {
                    fenced: code.fenced,
                    info,
                    info_span,
                    literal: code.literal.clone(),
                    phrases: None,
                })
            }
            NodeValue::BlockQuote => {
                let children = self.blocks(node);
                span = hull(span, children.iter().map(|b| b.span));
                BlockKind::BlockQuote(BlockQuote { children })
            }
            NodeValue::List(list) => {
                let items = node
                    .children()
                    .filter(|c| matches!(c.data().value, NodeValue::Item(_)))
                    .map(|item| {
                        let children = self.blocks(item);
                        let item_span = hull(self.span(item), children.iter().map(|b| b.span));
                        ListItem {
                            span: item_span,
                            marker: self.marker(item_span),
                            children,
                        }
                    })
                    .collect::<Vec<_>>();
                span = hull(span, items.iter().map(|i: &ListItem| i.span));
                let ordered = matches!(list.list_type, comrak_tessera::nodes::ListType::Ordered);
                BlockKind::List(List {
                    ordered,
                    start: ordered.then(|| u64::try_from(list.start).unwrap_or(u64::MAX)),
                    tight: list.tight,
                    items,
                })
            }
            NodeValue::HtmlBlock(html) => BlockKind::HtmlBlock(HtmlBlock {
                literal: html.literal.clone(),
            }),
            NodeValue::ThematicBreak => BlockKind::ThematicBreak,
            NodeValue::Table(table) => {
                let alignments = table
                    .alignments
                    .iter()
                    .map(|a| match a {
                        comrak_tessera::nodes::TableAlignment::Left => Alignment::Left,
                        comrak_tessera::nodes::TableAlignment::Center => Alignment::Center,
                        comrak_tessera::nodes::TableAlignment::Right => Alignment::Right,
                        comrak_tessera::nodes::TableAlignment::None => Alignment::None,
                    })
                    .collect();
                let rows = node
                    .children()
                    .map(|row| {
                        let header = matches!(row.data().value, NodeValue::TableRow(true));
                        TableRow {
                            span: self.span(row),
                            header,
                            cells: row
                                .children()
                                .map(|cell| {
                                    let span = self.span(cell);
                                    let mut inlines = self.inlines(cell);
                                    self.correct_cell_offsets(span, &mut inlines);
                                    TableCell { span, inlines }
                                })
                                .collect(),
                            attributes: None,
                        }
                    })
                    .collect::<Vec<_>>();
                span = hull(span, rows.iter().map(|r| r.span));
                BlockKind::Table(Table { rows, alignments })
            }
            // Nodes from extensions Ascribe doesn't enable never appear; if
            // one did, its blocks are kept and the wrapper dropped.
            _ => {
                for child in node.children() {
                    self.block(child, out);
                }
                return;
            }
        };
        out.push(Block { span, kind });
    }

    /// comrak parses a table cell after replacing each `\|` with `|`, so the
    /// positions it reports for the cell's inlines fall one byte short for
    /// every `\|` before them. This puts them back.
    fn correct_cell_offsets(&self, cell: Span, inlines: &mut [Inline]) {
        let source = self.text(cell);
        if !source.contains("\\|") {
            return;
        }
        // `map[u]` is the offset in the source of offset `u` in the unescaped text.
        let mut map = Vec::with_capacity(source.len() + 1);
        let bytes = source.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'\\' && bytes.get(i + 1) == Some(&b'|') {
                map.push(i);
                i += 2;
            } else {
                map.push(i);
                i += 1;
            }
        }
        map.push(source.len());
        let fix = |offset: usize| -> usize {
            let relative = offset.saturating_sub(cell.start());
            let mapped = map.get(relative).copied().unwrap_or(source.len());
            self.ceil(cell.start() + mapped)
        };
        fix_offsets(inlines, &fix);
    }

    /// The info string's span on a fence's opening line.
    fn info_span(&self, span: Span) -> Option<Span> {
        let text = self.text(span);
        let line = text.split(['\n', '\r']).next()?;
        let after_indent = line.trim_start_matches(' ');
        let fence = after_indent.chars().next()?;
        let after_fence = after_indent.trim_start_matches(fence);
        let trimmed_start = after_fence.trim_start_matches([' ', '\t']);
        let info = trimmed_start.trim_end_matches([' ', '\t']);
        if info.is_empty() {
            return None;
        }
        let start = span.start() + (line.len() - trimmed_start.len());
        Some(Span::new(start, start + info.len()))
    }

    /// The marker at the start of a list item.
    fn marker(&self, item: Span) -> Span {
        let text = self.text(item);
        let bytes = text.as_bytes();
        let digits = bytes.iter().take_while(|b| b.is_ascii_digit()).count();
        let len = if digits > 0 {
            digits + usize::from(matches!(bytes.get(digits), Some(b'.' | b')')))
        } else {
            usize::from(!bytes.is_empty())
        };
        Span::new(item.start(), item.start() + len)
    }

    // -- Ascribe lines ------------------------------------------------------

    fn tessera_line(
        &mut self,
        node: Node<'_>,
        line: &comrak_tessera::tessera::NodeTesseraLine,
    ) -> (Span, BlockKind) {
        let at = self.start_of(node.data().sourcepos.start);
        let head = parse_head(&line.raw, at, self.options.file);
        let name_span = Span::new(at, at + head.name_end);
        if line.name == tessera_core::END_KEYWORD {
            let end = self.end_line(&head, name_span, at, &line.name);
            return (end.span, BlockKind::End(end));
        }
        let directive = self.directive_line(node, line, head, name_span, at);
        (directive.span, BlockKind::Directive(directive))
    }

    fn end_line(&mut self, head: &Head, name_span: Span, at: usize, name: &str) -> EndLine {
        let first = head
            .attributes_range
            .map(|(start, _)| start)
            .or(head.colon)
            .or(head.unexpected_start);
        let extra = first.map(|start| Span::new(at + start, at + head.content_end));
        if let Some(extra) = extra {
            // SPEC §3.1: an end line is `@end` alone.
            let issue = Issue::new(diagnostics::DIRECTIVE_EXTRA_TEXT, self.location(extra))
                .with_variant("end")
                .with_arg("name", name)
                .with_arg("extra", self.text(extra));
            self.report(issue);
        }
        EndLine {
            span: name_span,
            name_span,
            extra,
        }
    }

    fn directive_line(
        &mut self,
        node: Node<'_>,
        line: &comrak_tessera::tessera::NodeTesseraLine,
        head: Head,
        name_span: Span,
        at: usize,
    ) -> DirectiveLine {
        let schema = self.options.schema(&line.name).cloned();
        let attributes_closed = head.attributes.as_ref().is_none_or(|a| a.closed);
        let colon = head.colon.map(|c| Span::new(at + c, at + c + 1));
        let unexpected = head
            .unexpected_start
            .map(|start| Span::new(at + start, at + head.content_end));
        if let Some(span) = unexpected {
            // SPEC §3.1: nothing else may appear on a directive line.
            self.report(
                Issue::new(diagnostics::DIRECTIVE_EXTRA_TEXT, self.location(span))
                    .with_variant("head")
                    .with_arg("name", line.name.clone())
                    .with_arg("extra", self.text(span)),
            );
        }
        let attributes = match head.attributes.clone() {
            Some(parsed) => {
                self.issues.extend(parsed.issues);
                Some(parsed.block)
            }
            None => None,
        };

        let primary = self.primary(node, line, &head, at, schema.as_ref());
        if let (Some(schema), None) = (&schema, &primary)
            && schema.primary.is_required()
        {
            self.report_missing(schema, name_span);
        }

        let form = if colon.is_some() && head.primary_start.is_none() && unexpected.is_none() {
            Form::Container
        } else {
            Form::Line
        };
        let end = match &primary {
            Some(PrimaryValue::Text(text)) => text.span.end(),
            _ => at + head.content_end,
        };
        DirectiveLine {
            span: Span::new(at, end),
            name: line.name.clone(),
            name_span,
            attributes,
            attributes_closed,
            colon,
            primary,
            form,
            unexpected,
            title: None,
            binding: None,
        }
    }

    fn report_missing(&mut self, schema: &DirectiveSchema, name_span: Span) {
        let (kind, example) = match schema.primary {
            Primary::Identifier { .. } => (
                "an identifier",
                match schema.name.as_str() {
                    "include" => "guides/setup.md",
                    "id" => "install",
                    _ => "name",
                },
            ),
            Primary::Availability { .. } => ("an availability spec", "cloud"),
            Primary::Text { .. } | Primary::None => ("text", "Text that fits here."),
        };
        self.report(
            Issue::new(diagnostics::DIRECTIVE_PRIMARY, self.location(name_span))
                .with_variant("missing")
                .with_arg("name", schema.name.clone())
                .with_arg("kind", kind)
                .with_arg("example", example),
        );
    }

    fn primary(
        &mut self,
        node: Node<'_>,
        line: &comrak_tessera::tessera::NodeTesseraLine,
        head: &Head,
        at: usize,
        schema: Option<&DirectiveSchema>,
    ) -> Option<PrimaryValue> {
        // A text primary is the child paragraph the block parser made.
        if line.text_primary.is_some()
            && let Some(paragraph) = node.first_child()
        {
            return Some(PrimaryValue::Text(self.text_primary(paragraph)));
        }
        let start = at + head.primary_start?;
        let end = at + head.content_end;
        let rest = self.text(Span::new(start, end));
        match schema.map(|s| s.primary) {
            Some(Primary::Identifier { .. }) => {
                // SPEC §3.4: text after the token is kept in `trailing` and
                // reported.
                let token_len = rest.find([' ', '\t']).unwrap_or(rest.len());
                let after = &rest[token_len..];
                let trailing = after.trim_start_matches([' ', '\t']);
                let trailing = (!trailing.is_empty())
                    .then(|| Span::new(start + token_len + (after.len() - trailing.len()), end));
                if let Some(span) = trailing {
                    self.report(
                        Issue::new(diagnostics::DIRECTIVE_EXTRA_TEXT, self.location(span))
                            .with_arg("name", line.name.clone())
                            .with_arg("extra", self.text(span)),
                    );
                }
                Some(PrimaryValue::Identifier(IdentifierPrimary {
                    span: Span::new(start, start + token_len),
                    text: rest[..token_len].to_owned(),
                    trailing,
                }))
            }
            Some(Primary::Availability { .. }) => Some(PrimaryValue::Line(LinePrimary {
                span: Span::new(start, end),
                text: rest.to_owned(),
            })),
            Some(Primary::Text { .. }) => {
                // The head parser and the block parser disagree about where a
                // text primary starts. That's a bug in one of them, which the
                // agreement test looks for; keep the line rather than drop it.
                Some(PrimaryValue::Text(TextPrimary {
                    span: Span::new(start, end),
                    lines: vec![Span::new(start, end)],
                    inlines: Vec::new(),
                }))
            }
            Some(Primary::None) | None => {
                let span = Span::new(start, end);
                self.report(
                    Issue::new(diagnostics::DIRECTIVE_PRIMARY, self.location(span))
                        .with_arg("name", line.name.clone()),
                );
                Some(PrimaryValue::Unexpected(span))
            }
        }
    }

    fn text_primary(&mut self, paragraph: Node<'_>) -> TextPrimary {
        let span = self.trimmed(self.span(paragraph));
        let lines = self.content_lines(span);
        self.scan_unknown(span, true);
        TextPrimary {
            span,
            lines,
            inlines: self.inlines(paragraph),
        }
    }

    /// `span` without trailing spaces and tabs.
    fn trimmed(&self, span: Span) -> Span {
        let text = self.text(span);
        Span::new(
            span.start(),
            span.start() + text.trim_end_matches([' ', '\t']).len(),
        )
    }

    /// The lines of `span` with container prefixes and trailing whitespace
    /// removed.
    fn content_lines(&self, span: Span) -> Vec<Span> {
        let text = self.text(span);
        let mut lines = Vec::new();
        let mut offset = 0;
        for (n, line) in split_lines(text).enumerate() {
            let start = if n == 0 {
                0
            } else {
                offset + (line.len() - strip_prefix(line).len())
            };
            let content = line.trim_end_matches([' ', '\t']);
            let end = (offset + content.len()).max(start);
            lines.push(Span::new(span.start() + start, span.start() + end));
            // Advance past this line and its ending.
            let after = offset + line.len();
            offset = after
                + if text[after..].starts_with("\r\n") {
                    2
                } else {
                    usize::from(text[after..].starts_with(['\n', '\r']))
                };
        }
        lines
    }

    // -- unknown directives -------------------------------------------------

    /// Warns about each line of a paragraph that is shaped like a directive
    /// with an unknown name (SPEC §3.2). `skip_first` skips the first line,
    /// which doesn't start at line start (it follows a directive's colon).
    fn scan_unknown(&mut self, span: Span, skip_first: bool) {
        let text = self.text(span);
        let mut offset = 0;
        for (n, line) in split_lines(text).enumerate() {
            let start = if n == 0 {
                0
            } else {
                offset + (line.len() - strip_prefix(line).len())
            };
            if !(n == 0 && skip_first) {
                let candidate = &text[start..offset + line.len()];
                if let Some(len) = directive_shape(candidate) {
                    let name = &candidate[1..=len];
                    if !is_known(name, self.options) {
                        let at = span.start() + start;
                        self.report_unknown(name, Span::new(at, at + 1 + len));
                    }
                }
            }
            let after = offset + line.len();
            offset = after
                + if text[after..].starts_with("\r\n") {
                    2
                } else {
                    usize::from(text[after..].starts_with(['\n', '\r']))
                };
        }
    }

    fn report_unknown(&mut self, name: &str, span: Span) {
        let mut issue =
            Issue::new(diagnostics::DIRECTIVE_UNKNOWN, self.location(span)).with_arg("name", name);
        if let Some(suggestion) = suggest(name, self.options) {
            issue = issue
                .with_arg("suggestion", suggestion)
                .with_variant("suggestion");
        }
        self.report(issue);
    }

    // -- inlines ------------------------------------------------------------

    fn inlines(&mut self, parent: Node<'_>) -> Vec<Inline> {
        let mut out: Vec<Inline> = Vec::new();
        for child in parent.children() {
            self.inline(child, &mut out);
        }
        out
    }

    fn inline(&mut self, node: Node<'_>, out: &mut Vec<Inline>) {
        // A line break's span is its line ending, so nothing is trimmed.
        let span = self.raw_span(node);
        let ast = node.data();
        let kind = match &ast.value {
            // An escape is text whose span includes its backslash.
            NodeValue::Text(_) | NodeValue::Escaped => {
                let text = match &ast.value {
                    NodeValue::Text(text) => text.to_string(),
                    _ => node
                        .first_child()
                        .and_then(|c| c.data().value.text().map(|t| t.to_string()))
                        .unwrap_or_default(),
                };
                // Text that touches the text before it is one node.
                if let Some(Inline {
                    span: previous,
                    kind: InlineKind::Text(previous_text),
                }) = out.last_mut()
                    && previous.end() == span.start()
                {
                    previous_text.push_str(&text);
                    *previous = Span::new(previous.start(), span.end());
                    return;
                }
                InlineKind::Text(text)
            }
            NodeValue::SoftBreak => InlineKind::SoftBreak,
            NodeValue::LineBreak => InlineKind::HardBreak,
            NodeValue::Code(code) => InlineKind::Code(code.literal.clone()),
            NodeValue::HtmlInline(html) => InlineKind::Html(html.clone()),
            NodeValue::Emph => InlineKind::Emphasis(self.inlines(node)),
            NodeValue::Strong => InlineKind::Strong(self.inlines(node)),
            NodeValue::Link(link) => InlineKind::Link(self.link(node, link, span)),
            NodeValue::Image(link) => InlineKind::Image(self.image(node, link, span)),
            _ => {
                // Inlines from extensions Ascribe doesn't enable: keep their
                // content.
                for child in node.children() {
                    self.inline(child, out);
                }
                return;
            }
        };
        out.push(Inline { span, kind });
    }

    fn link(&mut self, node: Node<'_>, link: &NodeLink, span: Span) -> Link {
        let (form, label) = self.link_form(span, false);
        Link {
            form,
            destination: link.url.clone(),
            title: (!link.title.is_empty()).then(|| link.title.clone()),
            label,
            destination_phrases: Vec::new(),
            children: self.inlines(node),
        }
    }

    fn image(&mut self, node: Node<'_>, link: &NodeLink, span: Span) -> Image {
        let (form, label) = self.link_form(span, true);
        let text = self.text(span);
        let alt = match matching_bracket(text, 1) {
            Some(close) => Span::new(span.start() + 2, span.start() + close),
            None => Span::empty(span.start() + 2),
        };
        Image {
            form,
            destination: link.url.clone(),
            title: (!link.title.is_empty()).then(|| link.title.clone()),
            label,
            alt,
            children: self.inlines(node),
            destination_phrases: Vec::new(),
            attributes: None,
        }
    }

    /// Which form a link or image was written in, from its source.
    fn link_form(&self, span: Span, image: bool) -> (LinkForm, Option<Span>) {
        let text = self.text(span);
        if !image && text.starts_with('<') {
            return (LinkForm::Autolink, None);
        }
        let open = usize::from(image);
        let Some(close) = matching_bracket(text, open) else {
            return (LinkForm::Inline, None);
        };
        let rest = &text[close + 1..];
        if rest.is_empty() {
            (LinkForm::Shortcut, None)
        } else if rest == "[]" {
            (LinkForm::Collapsed, None)
        } else if rest.starts_with('[') && rest.ends_with(']') {
            let label = Span::new(span.start() + close + 2, span.end() - 1);
            (LinkForm::Full, Some(label))
        } else {
            (LinkForm::Inline, None)
        }
    }
}

/// Cuts every inline at `limit`, and drops those that start at or after it.
fn clamp_inlines(inlines: &mut Vec<Inline>, limit: usize) {
    inlines.retain(|i| i.span.start() < limit || i.span.is_empty() && i.span.start() <= limit);
    for inline in inlines.iter_mut() {
        if inline.span.end() > limit {
            inline.span = Span::new(inline.span.start(), limit);
        }
        match &mut inline.kind {
            InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                clamp_inlines(children, limit);
            }
            InlineKind::Link(link) => clamp_inlines(&mut link.children, limit),
            InlineKind::Image(image) => clamp_inlines(&mut image.children, limit),
            _ => {}
        }
    }
}

/// Applies `fix` to the start and end of every span in `inlines`.
fn fix_offsets(inlines: &mut [Inline], fix: &dyn Fn(usize) -> usize) {
    let span = |s: Span| Span::new(fix(s.start()), fix(s.end()));
    for inline in inlines {
        inline.span = span(inline.span);
        match &mut inline.kind {
            InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                fix_offsets(children, fix);
            }
            InlineKind::Link(link) => {
                link.label = link.label.map(span);
                fix_offsets(&mut link.children, fix);
            }
            InlineKind::Image(image) => {
                image.label = image.label.map(span);
                image.alt = span(image.alt);
                fix_offsets(&mut image.children, fix);
            }
            _ => {}
        }
    }
}

/// `span`, grown to cover `others`. comrak's end position for a container
/// can fall short of its last child's when tabs or indented code end it.
fn hull(span: Span, others: impl Iterator<Item = Span>) -> Span {
    others.fold(span, |acc, other| {
        Span::new(acc.start().min(other.start()), acc.end().max(other.end()))
    })
}

/// The index of the `]` that matches the `[` at `open`, skipping escaped
/// brackets and code spans.
pub(crate) fn matching_bracket(text: &str, open: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    if bytes.get(open) != Some(&b'[') {
        return None;
    }
    let mut depth = 0usize;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 1,
            b'`' => {
                let run = bytes[i..].iter().take_while(|&&b| b == b'`').count();
                let closing = text[i + run..]
                    .match_indices(&"`".repeat(run))
                    .find(|(at, _)| {
                        let abs = i + run + at;
                        bytes.get(abs + run) != Some(&b'`') && (*at == 0 || bytes[abs - 1] != b'`')
                    });
                match closing {
                    Some((at, _)) => i += run + at + run - 1,
                    None => i += run - 1,
                }
            }
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_brackets() {
        assert_eq!(matching_bracket("![a](b)", 1), Some(3));
        assert_eq!(matching_bracket("[a [b] c](d)", 0), Some(8));
        assert_eq!(matching_bracket(r"[a \] b](c)", 0), Some(7));
        assert_eq!(matching_bracket("[a `]` b](c)", 0), Some(8));
        assert_eq!(matching_bracket("[a", 0), None);
        assert_eq!(matching_bracket("x", 0), None);
    }
}
