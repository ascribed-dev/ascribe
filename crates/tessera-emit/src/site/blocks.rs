//! The site output's blocks: markdown, and the custom elements the element
//! contract (`packages/elements/CONTRACT.md`) defines for what needs
//! presentation or interaction.
//!
//! The renderer walks the resolved tree and writes what survived; it never
//! works out what a build mode keeps (SPEC §9.2). A block becomes a chunk of
//! text, and chunks are separated by blank lines, so every element is
//! separated from the blocks around it (contract §0) and the markdown an
//! element wraps is parsed as markdown (SPEC §9.4).
//!
//! With anchors on, each chunk also says where its block came from
//! ([`super::anchor`]).

use tessera_core::availability::{Detail, Entry, parse_availability};
use tessera_core::names;
use tessera_core::{Attributes, DefaultValue};
use tessera_model::ContentModel;
use tessera_resolve::{ResolvedArm, ResolvedBlock, ResolvedKind};
use tessera_syntax::{
    Alignment, BlockKind, Bound, DirectiveLine, Inline, Link, PrimaryValue, Table,
};

use super::anchor::{Anchor, html_tag};
use super::element::{Attrs, close, empty, marker, open, wrap};
use super::inline::{Mode, render, value_text};
use crate::emitter::PageContext;
use crate::labels::{availability_target_text, plain_text};
use crate::plain::{Prev, escape_closing_hash, fenced, list, quote, starts_nested};

pub(crate) struct Renderer<'a> {
    pub(crate) page: &'a PageContext<'a>,
    pub(crate) model: &'a ContentModel,
    /// Whether to write source anchors.
    pub(crate) anchors: bool,
}

/// One block of site markdown, and its anchor comment.
struct Chunk {
    text: String,
    /// The anchor comment, written on the line before the text.
    anchor: Option<String>,
    /// What the block is, as far as a list's looseness goes.
    kind: ChunkKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ChunkKind {
    /// A paragraph, which has no element of its own in a tight list.
    Paragraph,
    /// A block whose blank lines are inside it (a code block, a raw HTML
    /// block, a list), so they don't make a list loose.
    Closed,
    /// Anything else. A blank line in it is a blank line between blocks: an
    /// element wrapping markdown.
    Other,
}

impl Chunk {
    fn new(text: String, kind: ChunkKind) -> Chunk {
        Chunk {
            text,
            anchor: None,
            kind,
        }
    }

    /// The chunk with its anchor comment, if it has one.
    fn full(self) -> String {
        match self.anchor {
            Some(anchor) => format!("{anchor}\n{}", self.text),
            None => self.text,
        }
    }
}

fn texts(chunks: Vec<Chunk>) -> Vec<String> {
    chunks.into_iter().map(Chunk::full).collect()
}

/// Whether a list the emitter writes renders as a loose list: its items are
/// separated by blank lines, an item's blocks are, or an item holds an element
/// that wraps markdown, which has blank lines inside it.
fn renders_loose(tight: bool, bodies: &[Vec<Chunk>]) -> bool {
    (!tight && bodies.len() > 1)
        || bodies.iter().any(|chunks| {
            chunks
                .iter()
                .skip(1)
                .any(|c| !tight || !starts_nested(&c.text))
                || chunks
                    .iter()
                    .any(|c| c.kind == ChunkKind::Other && c.text.contains("\n\n"))
        })
}

impl Renderer<'_> {
    fn inlines(&self, block: &ResolvedBlock, list: &[Inline], mode: Mode) -> String {
        render(self, block, list, mode)
    }

    /// The glossary term a link is a use of, as its id and the definition
    /// the link carries as its title: the term whose use in this block has
    /// this URL and text.
    pub(crate) fn glossary_term(
        &self,
        block: &ResolvedBlock,
        link: &Link,
    ) -> Option<(String, String)> {
        let text = plain_text(&link.children);
        let used = block
            .glossary
            .iter()
            .find(|g| g.url == link.destination && g.text == text)?;
        self.model
            .glossary
            .terms
            .iter()
            .find(|t| t.id == used.term)
            .map(|t| (t.id.clone(), t.definition.clone()))
    }

    /// A list of sibling blocks as chunks: one string per block (a directive
    /// that wraps the next block is written with it).
    pub(crate) fn blocks(&self, blocks: &[ResolvedBlock]) -> Vec<String> {
        texts(self.chunks(blocks))
    }

    fn chunks(&self, blocks: &[ResolvedBlock]) -> Vec<Chunk> {
        let mut out = Vec::new();
        let mut pending: Vec<&ResolvedBlock> = Vec::new();
        let mut prev = Prev::None;
        for block in blocks {
            if is_following(block) {
                pending.push(block);
                continue;
            }
            let mut chunks = self.block(block, prev);
            let wrapped = !pending.is_empty();
            while let Some(directive) = pending.pop() {
                chunks = self.apply(directive, Some(block), chunks);
            }
            // A list holding an element is loose whatever
            // the source said, since the element needs a blank line.
            // A list written straight after another of the same kind would
            // merge with it, so the second gets another marker. A directive
            // that wrote an element between them keeps them apart.
            prev = match &block.kind {
                ResolvedKind::List { ordered, .. } if !wrapped => {
                    if *ordered {
                        Prev::Ordered
                    } else {
                        Prev::Bullet
                    }
                }
                _ => Prev::None,
            };
            out.extend(chunks);
        }
        // Directives with no block after them (an error the checks report).
        let mut chunks = Vec::new();
        while let Some(directive) = pending.pop() {
            chunks = self.apply(directive, None, chunks);
        }
        out.extend(chunks);
        out
    }

    /// The block's anchor, when anchors are on.
    fn anchor(&self, block: &ResolvedBlock) -> Option<Anchor> {
        if !self.anchors {
            return None;
        }
        Anchor::of(self.page.emit, block)
    }

    /// A chunk for a block the markdown pipeline renders as a `tag` element,
    /// with its anchor comment.
    fn anchored(&self, block: &ResolvedBlock, tag: &str, text: String, kind: ChunkKind) -> Chunk {
        Chunk {
            anchor: self.anchor(block).map(|a| a.comment(tag, &[])),
            text,
            kind,
        }
    }

    /// An element the emitter writes, with the anchor's attributes on its tag.
    fn element(&self, anchor: Option<Anchor>, text: String) -> Chunk {
        let text = match anchor {
            Some(anchor) => anchor.on_element(text),
            None => text,
        };
        Chunk::new(text, ChunkKind::Other)
    }

    fn block(&self, block: &ResolvedBlock, prev: Prev) -> Vec<Chunk> {
        match &block.kind {
            ResolvedKind::Leaf(b) => self.leaf(block, &b.kind),
            ResolvedKind::BlockQuote { children } => {
                let text = quote(&self.blocks(children).join("\n\n"));
                vec![self.anchored(block, "blockquote", text, ChunkKind::Other)]
            }
            ResolvedKind::List {
                ordered,
                start,
                tight,
                items,
            } => {
                let bodies: Vec<Vec<Chunk>> =
                    items.iter().map(|i| self.chunks(&i.children)).collect();
                // A paragraph in a tight list has no `<p>` to carry its anchor:
                // the item's says where it is.
                let loose = renders_loose(*tight, &bodies);
                let bodies: Vec<Vec<String>> = bodies
                    .into_iter()
                    .map(|chunks| {
                        chunks
                            .into_iter()
                            .map(|c| {
                                if !loose && c.kind == ChunkKind::Paragraph {
                                    c.text
                                } else {
                                    c.full()
                                }
                            })
                            .collect()
                    })
                    .collect();
                let text = list(*ordered, *start, *tight, &bodies, prev);
                let tag = if *ordered { "ol" } else { "ul" };
                let anchor = self.anchor(block).map(|anchor| {
                    let cx = self.page.emit;
                    let items: Option<Vec<(u32, u32)>> = items
                        .iter()
                        .map(|i| super::anchor::lines(cx, block.file, i.span))
                        .collect();
                    anchor.comment(tag, &items.unwrap_or_default())
                });
                vec![Chunk {
                    text,
                    anchor,
                    kind: ChunkKind::Closed,
                }]
            }
            ResolvedKind::Container {
                opener, children, ..
            } => self.container(block, opener, children),
            ResolvedKind::Group { name, arms, .. } => {
                let text = self.group(block, name, arms);
                vec![self.element(self.anchor(block), text)]
            }
        }
    }

    fn leaf(&self, block: &ResolvedBlock, kind: &BlockKind) -> Vec<Chunk> {
        match kind {
            BlockKind::Heading(h) => {
                let tag = format!("h{}", h.level.clamp(1, 6));
                let text = self.heading(block, h);
                vec![self.anchored(block, &tag, text, ChunkKind::Other)]
            }
            BlockKind::Paragraph(p) => {
                let text = self.inlines(block, &p.inlines, Mode::default());
                vec![self.anchored(block, "p", text, ChunkKind::Paragraph)]
            }
            BlockKind::CodeBlock(c) => {
                vec![self.anchored(block, "pre", fenced(c), ChunkKind::Closed)]
            }
            // Raw HTML passes through (SPEC §9.5, "HTML passthrough"). Its
            // anchor names the element it starts with; one that starts with
            // anything else has none.
            BlockKind::HtmlBlock(h) => {
                let text = h.literal.trim_end_matches(['\n', '\r']);
                if text.trim().is_empty() {
                    Vec::new()
                } else {
                    match html_tag(text) {
                        Some(tag) => {
                            vec![self.anchored(block, &tag, text.to_owned(), ChunkKind::Closed)]
                        }
                        None => vec![Chunk::new(text.to_owned(), ChunkKind::Closed)],
                    }
                }
            }
            BlockKind::ThematicBreak => {
                vec![self.anchored(block, "hr", "---".to_owned(), ChunkKind::Other)]
            }
            BlockKind::Table(t) => {
                let text = self.table(block, t);
                vec![self.anchored(block, "table", text, ChunkKind::Other)]
            }
            BlockKind::Directive(line) => self.directive(block, line),
            BlockKind::BlockQuote(_)
            | BlockKind::List(_)
            | BlockKind::End(_)
            | BlockKind::Container(_)
            | BlockKind::Group(_)
            | BlockKind::Title(_) => Vec::new(),
        }
    }

    /// An ATX heading ending in a marker with its page id, one space after the
    /// text. A heading whose page id is empty gets
    /// no marker.
    fn heading(&self, block: &ResolvedBlock, heading: &tessera_syntax::Heading) -> String {
        let text = self.inlines(
            block,
            &heading.inlines,
            Mode {
                one_line: true,
                ..Mode::default()
            },
        );
        let id = block
            .heading
            .as_ref()
            .map(|h| h.page_id.as_str())
            .filter(|id| !id.is_empty());
        let marks = "#".repeat(usize::from(heading.level.clamp(1, 6)));
        match (text.is_empty(), id) {
            (true, None) => marks,
            (true, Some(id)) => format!("{marks} {}", marker(&[("id".to_owned(), id.to_owned())])),
            (false, Some(id)) => format!(
                "{marks} {text} {}",
                marker(&[("id".to_owned(), id.to_owned())])
            ),
            (false, None) => format!("{marks} {}", escape_closing_hash(text)),
        }
    }

    fn table(&self, block: &ResolvedBlock, table: &Table) -> String {
        let mode = Mode {
            one_line: true,
            ..Mode::default()
        };
        let width = table.rows.first().map_or(0, |r| r.cells.len());
        let mut lines = Vec::new();
        for (n, row) in table.rows.iter().enumerate() {
            let mut cells: Vec<String> = row
                .cells
                .iter()
                .take(width)
                .map(|c| self.inlines(block, &c.inlines, mode).replace('|', "\\|"))
                .collect();
            cells.resize(width, String::new());
            // A row's availability ends its first cell (contract §4).
            if let Some(r) = block.rows.iter().find(|r| r.span == row.span)
                && let Some(first) = cells.first_mut()
            {
                let badge = availability_element(self.model, "row", &r.availability.spec.entries)
                    .replace('\n', "")
                    .replace('|', "\\|");
                if first.is_empty() {
                    *first = badge;
                } else {
                    first.push(' ');
                    first.push_str(&badge);
                }
            }
            lines.push(format!("| {} |", cells.join(" | ")));
            if n == 0 {
                let delimiters: Vec<&str> = (0..width)
                    .map(|i| match table.alignments.get(i) {
                        Some(Alignment::Left) => ":---",
                        Some(Alignment::Center) => ":---:",
                        Some(Alignment::Right) => "---:",
                        Some(Alignment::None) | None => "---",
                    })
                    .collect();
                lines.push(format!("| {} |", delimiters.join(" | ")));
            }
        }
        lines.join("\n")
    }

    /// A directive line that stands alone (it wraps no following block).
    fn directive(&self, block: &ResolvedBlock, line: &DirectiveLine) -> Vec<Chunk> {
        let anchor = self.anchor(block);
        match line.name.as_str() {
            "id" | "include" | "snippet" | "steps" | "details" => Vec::new(),
            "available" => self
                .availability(block)
                .map(|text| self.element(anchor, text))
                .into_iter()
                .collect(),
            "note" => match &line.primary {
                Some(PrimaryValue::Text(text)) => {
                    let content = self.inlines(block, &text.inlines, Mode::default());
                    let content = self.anchored(block, "p", content, ChunkKind::Paragraph);
                    vec![self.element(anchor, self.note(line, &[content.full()]))]
                }
                _ => Vec::new(),
            },
            name if self.model.widget(name).is_some() => {
                let inner = match &line.primary {
                    Some(PrimaryValue::Text(text)) => {
                        let content = self.inlines(block, &text.inlines, Mode::default());
                        vec![
                            self.anchored(block, "p", content, ChunkKind::Paragraph)
                                .full(),
                        ]
                    }
                    _ => Vec::new(),
                };
                if inner.is_empty() {
                    // An empty element on a line of its own is a paragraph
                    // to CommonMark, which gets the anchor too.
                    let text = self.element(anchor, self.widget(line, &inner)).text;
                    vec![self.anchored(block, "p", text, ChunkKind::Paragraph)]
                } else {
                    vec![self.element(anchor, self.widget(line, &inner))]
                }
            }
            _ => Vec::new(),
        }
    }

    /// A following-block directive applied to the block it binds. An element
    /// that wraps the block is anchored from the directive through the block.
    fn apply(
        &self,
        directive: &ResolvedBlock,
        block: Option<&ResolvedBlock>,
        bound: Vec<Chunk>,
    ) -> Vec<Chunk> {
        let ResolvedKind::Leaf(b) = &directive.kind else {
            return bound;
        };
        let BlockKind::Directive(line) = &b.kind else {
            return bound;
        };
        let spanning = || {
            if self.anchors {
                Anchor::spanning(self.page.emit, directive, block)
            } else {
                None
            }
        };
        match line.name.as_str() {
            "note" => vec![self.element(spanning(), self.note(line, &texts(bound)))],
            "steps" => vec![self.element(
                spanning(),
                wrap(names::ELEMENT_STEPS, &Attrs::new(), &texts(bound)),
            )],
            "details" => {
                vec![self.element(spanning(), self.details(directive, line, &texts(bound)))]
            }
            "available" => {
                let anchor = self.anchor(directive);
                let mut out: Vec<Chunk> = self
                    .availability(directive)
                    .map(|text| self.element(anchor, text))
                    .into_iter()
                    .collect();
                out.extend(bound);
                out
            }
            name if self.model.widget(name).is_some() => {
                vec![self.element(spanning(), self.widget(line, &texts(bound)))]
            }
            _ => bound,
        }
    }

    fn container(
        &self,
        block: &ResolvedBlock,
        opener: &DirectiveLine,
        children: &[ResolvedBlock],
    ) -> Vec<Chunk> {
        let anchor = self.anchor(block);
        match opener.name.as_str() {
            "note" => vec![self.element(anchor, self.note(opener, &self.blocks(children)))],
            "details" => {
                vec![self.element(anchor, self.details(block, opener, &self.blocks(children)))]
            }
            name if self.model.widget(name).is_some() => {
                vec![self.element(anchor, self.widget(opener, &self.blocks(children)))]
            }
            _ => self.chunks(children),
        }
    }

    // ------------------------------------------------------------------
    // Elements

    /// The plain text of a directive's title line (contract §0), or `None`.
    fn title_text(&self, line: &DirectiveLine) -> Option<String> {
        let title = line.title.as_ref()?;
        let text = plain_text(&title.inlines);
        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
        (!text.is_empty()).then_some(text)
    }

    /// `<ascribe-note>` (contract §1).
    fn note(&self, line: &DirectiveLine, content: &[String]) -> String {
        let kind = line
            .attributes
            .as_ref()
            .and_then(|a| a.get("type"))
            .and_then(|a| a.value.as_ref())
            .and_then(|v| v.as_text())
            .unwrap_or("note");
        let label = self
            .model
            .note_type(kind)
            .map_or_else(|| kind.to_owned(), |t| t.label.clone());
        let attrs = Attrs::new()
            .with("type", kind)
            .with("label", label)
            .with_opt("heading", self.title_text(line));
        wrap(names::ELEMENT_NOTE, &attrs, content)
    }

    /// `<details>` with a `<summary>` (contract §5): the title's inline
    /// content is rendered as HTML, since an HTML block holds no markdown.
    // An image in the title is its alt text.
    fn details(&self, block: &ResolvedBlock, line: &DirectiveLine, content: &[String]) -> String {
        let mut out = String::from("<details>\n");
        if let Some(title) = &line.title {
            let markdown = self.inlines(
                block,
                &title.inlines,
                Mode {
                    one_line: true,
                    images_as_alt: true,
                },
            );
            out.push_str(&format!(
                "<summary>{}</summary>\n",
                crate::render::inline_html(&markdown)
            ));
        }
        out.push('\n');
        for chunk in content {
            out.push_str(chunk);
            out.push_str("\n\n");
        }
        out.push_str("</details>");
        out
    }

    /// `<ascribe-availability>` for a surviving `@available` (contract §4),
    /// written where the directive was.
    fn availability(&self, block: &ResolvedBlock) -> Option<String> {
        let annotation = block.annotation.as_ref()?;
        let spec = parse_availability(&annotation.text, 0).ok()?;
        let scope = match annotation.binding {
            Some(Bound::Heading) => "section",
            _ => "block",
        };
        Some(availability_element(self.model, scope, &spec.entries))
    }

    /// A project widget (contract §6): an element named after the widget,
    /// wrapping `content`, or empty when there is none.
    fn widget(&self, line: &DirectiveLine, content: &[String]) -> String {
        let attrs = self.widget_attrs(line);
        if content.is_empty() {
            empty(&line.name, &attrs)
        } else {
            wrap(&line.name, &attrs, content)
        }
    }

    /// A widget's attributes, in the contract's order: `heading`, `primary`,
    /// then each declared attribute with the value given or its default.
    fn widget_attrs(&self, line: &DirectiveLine) -> Attrs {
        let mut attrs = Attrs::new().with_opt("heading", self.title_text(line));
        if let Some(PrimaryValue::Identifier(p)) = &line.primary {
            attrs = attrs.with_opt("primary", Some(p.text.clone()));
        }
        let declared = self
            .model
            .widget_schema(&line.name)
            .map(|s| match &s.attributes {
                Attributes::Declared(list) => list.as_slice(),
                Attributes::Dimensions => &[],
            })
            .unwrap_or(&[]);
        for schema in declared {
            let value = line
                .attributes
                .as_ref()
                .and_then(|b| b.get(&schema.key))
                .and_then(|a| a.value.as_ref())
                .map(value_text)
                .or_else(|| schema.default.as_ref().map(default_text));
            if let Some(value) = value {
                attrs = attrs.with(&schema.key, value);
            }
        }
        attrs
    }

    /// An arm's anchor: its span, in the group's file.
    fn arm_anchor(&self, group: &ResolvedBlock, arm: &ResolvedArm) -> Option<Anchor> {
        if !self.anchors {
            return None;
        }
        Anchor::of_span(self.page.emit, group.file, arm.span, &group.via)
    }

    /// An arm's element, anchored.
    fn arm_element(&self, group: &ResolvedBlock, arm: &ResolvedArm, text: String) -> String {
        match self.arm_anchor(group, arm) {
            Some(anchor) => anchor.on_element(text),
            None => text,
        }
    }

    /// A surviving group (SPEC §9.4): `<ascribe-tabs>` for `@variant`, and a
    /// `<ascribe-group>` of one element per arm for a widget.
    fn group(&self, block: &ResolvedBlock, name: &str, arms: &[ResolvedArm]) -> String {
        if name == "variant" {
            return self.tabs(block, arms);
        }
        let elements: Vec<String> = arms
            .iter()
            .map(|arm| {
                let text = self.widget(&arm.opener, &self.blocks(&arm.children));
                self.arm_element(block, arm, text)
            })
            .collect();
        wrap(
            names::ELEMENT_GROUP,
            &Attrs::new().with("widget", name),
            &elements,
        )
    }

    /// `<ascribe-tabs>` and its `<ascribe-tab>`s (contract §3).
    fn tabs(&self, block: &ResolvedBlock, arms: &[ResolvedArm]) -> String {
        let values: Vec<Vec<(String, Vec<String>)>> = arms
            .iter()
            .map(|arm| arm_values(self.model, &arm.opener))
            .collect();
        let sync = sync_dimension(self.model, &values);
        let tabs: Vec<String> = arms
            .iter()
            .zip(&values)
            .map(|(arm, arm_values)| {
                let value = sync.as_deref().and_then(|dim| {
                    arm_values
                        .iter()
                        .find(|(k, _)| k == dim)
                        .map(|(_, v)| v.join(" "))
                });
                let label = if arm_values.is_empty() {
                    arm.opener.title.as_ref().map(|t| {
                        plain_text(&t.inlines)
                            .split_whitespace()
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                } else {
                    Some(dimensional_label(self.model, arm_values))
                };
                let attrs = Attrs::new()
                    .with_opt("value", value)
                    .with_opt("label", label);
                let text = wrap(names::ELEMENT_TAB, &attrs, &self.blocks(&arm.children));
                self.arm_element(block, arm, text)
            })
            .collect();
        let attrs = Attrs::new().with_opt("sync", sync);
        wrap(names::ELEMENT_TABS, &attrs, &tabs)
    }
}

/// `<ascribe-availability>` with one `<ascribe-availability-target>` per
/// entry, as one HTML block (contract §4): no blank line inside.
pub(crate) fn availability_element(model: &ContentModel, scope: &str, entries: &[Entry]) -> String {
    let targets: Vec<String> = entries
        .iter()
        .map(|entry| {
            let view = availability_target(model, entry);
            let attrs = Attrs::new()
                .with("target", view.target)
                .with("dimension", view.dimension)
                .with("states", view.states.join(" "))
                .with_opt(
                    "versions",
                    (!view.versions.is_empty()).then(|| view.versions.join(" ")),
                );
            format!(
                "{}{}{}",
                open(names::ELEMENT_AVAILABILITY_TARGET, &attrs),
                super::element::escape(&view.text),
                close(names::ELEMENT_AVAILABILITY_TARGET)
            )
        })
        .collect();
    format!(
        "{}\n{}\n{}",
        open(
            names::ELEMENT_AVAILABILITY,
            &Attrs::new().with("scope", scope)
        ),
        targets.join("; "),
        close(names::ELEMENT_AVAILABILITY)
    )
}

/// One target of an availability spec, as the element and the page-level
/// frontmatter give it.
pub(crate) struct TargetView {
    pub(crate) target: String,
    pub(crate) dimension: String,
    pub(crate) states: Vec<String>,
    pub(crate) versions: Vec<String>,
    pub(crate) text: String,
}

/// Works out a spec entry's `dimension`, `states`, and `versions` (contract
/// §4), and the text a reader sees.
pub(crate) fn availability_target(model: &ContentModel, entry: &Entry) -> TargetView {
    let target = entry.target.text.clone();
    let dimension = model
        .dimension_of_value(&target)
        .map(|d| d.name.clone())
        .unwrap_or_else(|| target.clone());
    let (states, versions) = match &entry.detail {
        Detail::None => (vec!["ga".to_owned()], Vec::new()),
        Detail::Version(v) => (vec!["ga".to_owned()], vec![v.text.clone()]),
        Detail::State { state, version } => (
            vec![state.text.clone()],
            version.iter().map(|v| v.text.clone()).collect(),
        ),
        Detail::History(steps) => (
            steps.iter().map(|s| s.state.text.clone()).collect(),
            steps.iter().map(|s| s.version.text.clone()).collect(),
        ),
    };
    TargetView {
        text: availability_target_text(model, entry),
        target,
        dimension,
        states,
        versions,
    }
}

/// An attribute default as its text: booleans are `true` or `false`, a value
/// set's members are joined by spaces (contract §6).
pub(crate) fn default_text(value: &DefaultValue) -> String {
    match value {
        DefaultValue::Text(t) => t.clone(),
        DefaultValue::Boolean(b) => b.to_string(),
        DefaultValue::Set(members) => members.join(" "),
    }
}

/// An arm's attributes as `(dimension, values)`, in the content model's
/// declaration order of dimensions (canonical order), then any other key in
/// written order. Empty for a labeled arm.
pub(crate) fn arm_values(
    model: &ContentModel,
    opener: &DirectiveLine,
) -> Vec<(String, Vec<String>)> {
    let Some(block) = &opener.attributes else {
        return Vec::new();
    };
    let mut pairs: Vec<(String, Vec<String>)> = block
        .attributes
        .iter()
        .filter_map(|a| {
            a.value.as_ref().map(|v| {
                (
                    a.key.clone(),
                    v.members().into_iter().map(str::to_owned).collect(),
                )
            })
        })
        .collect();
    let position = |key: &str| {
        model
            .dimensions
            .iter()
            .position(|d| d.name == key)
            .unwrap_or(usize::MAX)
    };
    pairs.sort_by_key(|(k, _)| position(k));
    pairs
}

/// A dimensional arm's label: each attribute's values as their labels joined
/// with ` / `, attributes joined with `, ` (contract §3).
pub(crate) fn dimensional_label(model: &ContentModel, values: &[(String, Vec<String>)]) -> String {
    values
        .iter()
        .map(|(_, members)| {
            members
                .iter()
                .map(|v| model.value_label(v).unwrap_or(v).to_owned())
                .collect::<Vec<_>>()
                .join(" / ")
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The dimension a group syncs on (contract §3): among the dimensions every
/// arm names, the first in the content model's order on which the arms'
/// values differ, else the first they share. `None` for a labeled group.
pub(crate) fn sync_dimension(
    model: &ContentModel,
    arms: &[Vec<(String, Vec<String>)>],
) -> Option<String> {
    if arms.is_empty() || arms.iter().any(Vec::is_empty) {
        return None;
    }
    let values = |arm: &[(String, Vec<String>)], dim: &str| -> Option<Vec<String>> {
        arm.iter().find(|(k, _)| k == dim).map(|(_, v)| {
            let mut v = v.clone();
            v.sort();
            v
        })
    };
    let shared: Vec<&str> = model
        .dimensions
        .iter()
        .map(|d| d.name.as_str())
        .filter(|dim| arms.iter().all(|arm| values(arm, dim).is_some()))
        .collect();
    let differs = |dim: &&str| {
        let first = values(&arms[0], dim);
        arms.iter().any(|arm| values(arm, dim) != first)
    };
    shared
        .iter()
        .find(|dim| differs(dim))
        .or_else(|| shared.first())
        .map(|dim| (*dim).to_owned())
}

/// Whether a block is a line-form directive that binds the next block.
fn is_following(block: &ResolvedBlock) -> bool {
    match &block.kind {
        ResolvedKind::Leaf(b) => matches!(
            &b.kind,
            BlockKind::Directive(line) if line.binding == Some(Bound::FollowingBlock)
        ),
        _ => false,
    }
}
