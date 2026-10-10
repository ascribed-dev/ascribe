//! The plain-markdown output (SPEC §9.4): fully resolved CommonMark with no
//! HTML, for language models, search indexing, and export.
//!
//! The emitter walks the resolved tree and writes what survived; it never
//! works out what a build mode keeps. Each construct follows SPEC §9.4's
//! table:
//!
//! | Source | Plain output |
//! |---|---|
//! | `@note` | A blockquote beginning `**Tip: Title**` (the type's display label) |
//! | `@steps` | The ordered list |
//! | A `@variant` group | One section per surviving arm, led by its bold label, each of its headings ending with the label in parentheses (a group reduced to one arm is that arm's content, already, in the tree) |
//! | `@details` | The title in bold, then the content |
//! | `@available` | `Available: Quill Cloud (GA); self-managed (preview, 3.4+)` |
//! | A project widget | Its `plain-fallback`, and its wrapped content unless `plain-content = "drop"` |
//! | Phrases, includes, glossary links | Resolved |
//! | Links | Absolute URLs (the site's origin, when the content model has one) |
//!
//! Pages start with their title as a level-1 heading. Raw HTML in the
//! source keeps its text and loses its tags, since the output has no HTML.
//!
//! # For agents
//!
//! With `[consumer] agents = true` the output is what a site publishes for
//! agents, laid out by URL so that it's copied as it is to the site's base
//! path: each page at its Markdown version's path
//! ([`ascribe_resolve::AstroRouter::markdown_path`]), opening with a
//! blockquote that points at the index; `llms.txt`, and a file per section
//! when it's split ([`ascribe_resolve::llms`]); and every asset under
//! `_ascribe/files/`, referred to by its absolute URL.

pub(crate) mod inline;

use std::cell::RefCell;

use ascribe_core::{AssetUse, RelPath};
use ascribe_model::{ContentModel, PlainContent, Segment};
use ascribe_resolve::{
    AstroRouter, ResolvedBlock, ResolvedBuild, ResolvedKind, ResolvedLink, ResolvedPage, llms,
};
use ascribe_syntax::{Alignment, BlockKind, Bound, CodeBlock, DirectiveLine, PrimaryValue, Table};

use crate::assets::{Placement, mirrored_path};
use crate::emitter::{EmitContext, Emitter, PageContext, mirrored_placement};
use crate::error::EmitError;
use crate::labels::{availability_display, availability_display_of_text, dimensional_label};
use crate::site::{FILES_DIR, route_collisions};
use crate::store::{Contents, EmittedFile, FileKind};
use inline::{Style, escape};

/// The plain-markdown emitter.
#[derive(Clone, Debug, Default)]
pub struct PlainEmitter {
    /// The router of the outputs for agents, when the content model asks for
    /// them (`[consumer] agents = true`).
    agents: Option<Agents>,
}

/// What the outputs for agents need: where each page is published, and the
/// site's origin for the absolute URLs.
#[derive(Clone, Debug)]
struct Agents {
    router: AstroRouter,
    origin: String,
}

impl Agents {
    /// A URL path under the base path as an absolute URL.
    fn absolute(&self, url: &str) -> String {
        format!("{}{url}", self.origin.trim_end_matches('/'))
    }
}

impl PlainEmitter {
    /// The emitter for a content model: laid out for agents when the model
    /// asks for that (`[consumer] agents = true`), as the module
    /// documentation says, and otherwise mirroring the sources.
    pub fn new(model: &ContentModel) -> PlainEmitter {
        let consumer = &model.consumer;
        PlainEmitter {
            agents: consumer
                .site
                .as_ref()
                .filter(|_| consumer.agents)
                .map(|origin| Agents {
                    router: AstroRouter::from_consumer(consumer),
                    origin: origin.clone(),
                }),
        }
    }
}

impl Emitter for PlainEmitter {
    fn name(&self) -> &'static str {
        "plain"
    }

    fn page_path(&self, page: &RelPath) -> RelPath {
        match &self.agents {
            Some(agents) => {
                RelPath::parse(&agents.router.markdown_path(page)).unwrap_or_else(|_| page.clone())
            }
            None => page.clone(),
        }
    }

    fn prepare(&self, _cx: &EmitContext<'_>, build: &ResolvedBuild) -> Result<(), EmitError> {
        match &self.agents {
            // Laid out by URL, two pages with one route would be one file.
            Some(agents) => route_collisions(&agents.router, build, "the plain output for agents"),
            None => Ok(()),
        }
    }

    fn render_page(&self, cx: &PageContext<'_>, page: &ResolvedPage) -> Result<String, EmitError> {
        let text = render_page(cx, page);
        Ok(match &self.agents {
            Some(agents) => {
                let index = agents.absolute(&agents.router.url_of("llms.txt"));
                format!(
                    "> For the complete documentation index, see [llms.txt]({index}).\n\n{text}"
                )
            }
            None => text,
        })
    }

    fn place_asset(&self, page_output: &RelPath, asset: &RelPath, _usage: AssetUse) -> Placement {
        let Some(agents) = &self.agents else {
            return mirrored_placement(page_output, asset);
        };
        // Published at its URL, so a page refers to it from wherever an
        // agent reads it.
        let mirrored = mirrored_path(asset);
        let copy_to =
            RelPath::parse(&format!("{FILES_DIR}/{mirrored}")).unwrap_or_else(|_| mirrored.clone());
        let url = agents.router.url_of(copy_to.as_str());
        Placement {
            reference: agents.absolute(&url),
            copy_to,
            url: Some(url),
        }
    }

    fn generated(
        &self,
        cx: &EmitContext<'_>,
        build: &ResolvedBuild,
    ) -> Result<Vec<EmittedFile>, EmitError> {
        if self.agents.is_none() {
            return Ok(Vec::new());
        }
        llms::llms_files(cx.model, &build.pages)
            .into_iter()
            .map(|file| {
                let path = RelPath::parse(&file.path).map_err(|e| EmitError::Invalid {
                    message: format!("bad llms.txt path {}: {e}", file.path),
                })?;
                Ok(EmittedFile {
                    path,
                    kind: FileKind::Generated,
                    source: None,
                    url: None,
                    contents: Contents::Text(file.text),
                })
            })
            .collect()
    }

    fn warnings(&self, cx: &EmitContext<'_>) -> Vec<String> {
        // Without a site origin, links can only be root-relative. This is
        // about an output's configuration, not the source, so it's a build
        // warning rather than a registry diagnostic.
        if cx.site_origin().is_none() {
            vec![
                "[consumer] site isn't set in ascribe.toml, so links in the plain-markdown output are root-relative, not absolute URLs"
                    .to_owned(),
            ]
        } else {
            Vec::new()
        }
    }
}

/// Renders a page as plain markdown.
fn render_page(cx: &PageContext<'_>, page: &ResolvedPage) -> String {
    let r = Renderer {
        page: cx,
        model: cx.emit.model,
        arms: RefCell::new(Vec::new()),
    };
    let mut chunks = Vec::new();
    if let Some(segments) = page.formatted_title() {
        chunks.push(format!("# {}", formatted_heading(segments)));
    } else if let Some(title) = &page.title {
        // A folded YAML scalar can hold a line break; a heading can't.
        let title = title.split_whitespace().collect::<Vec<_>>().join(" ");
        chunks.push(format!("# {}", escape(&title, false)));
    }
    if let Some(availability) = &page.availability {
        chunks.push(r.availability_line(&availability.spec));
    }
    chunks.extend(r.blocks(&page.blocks));
    let mut out = chunks.join("\n\n");
    if !out.is_empty() {
        out.push('\n');
    }
    out
}

/// A title read with `inline = "code"` as a heading's text: its text
/// escaped, with line breaks and runs of spaces made one space, and its code
/// spans as code spans.
fn formatted_heading(segments: &[Segment]) -> String {
    let mut out = String::new();
    for segment in segments {
        match segment {
            Segment::Text(text) => {
                let mut collapsed = String::with_capacity(text.len());
                let mut space = false;
                for ch in text.chars() {
                    if ch.is_whitespace() {
                        space = true;
                        continue;
                    }
                    if std::mem::take(&mut space) {
                        collapsed.push(' ');
                    }
                    collapsed.push(ch);
                }
                if space {
                    collapsed.push(' ');
                }
                out.push_str(&escape(&collapsed, false));
            }
            Segment::Code(code) => out.push_str(&inline::code_span(code)),
        }
    }
    out.trim().to_owned()
}

/// What kind of list the last emitted block was, so that an adjacent list of
/// the same kind gets another marker and doesn't merge with it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Prev {
    None,
    Bullet,
    Ordered,
}

pub(crate) struct Renderer<'a> {
    pub(crate) page: &'a PageContext<'a>,
    model: &'a ContentModel,
    /// The labels of the arms the block being written is in, outermost
    /// first, which its headings carry.
    arms: RefCell<Vec<String>>,
}

impl Renderer<'_> {
    fn inlines(
        &self,
        block: &ResolvedBlock,
        list: &[ascribe_syntax::Inline],
        style: Style,
    ) -> String {
        inline::render(self, &block.links, list, style)
    }

    fn availability_line(&self, spec: &ascribe_core::availability::AvailabilitySpec) -> String {
        format!(
            "Available: {}",
            escape(&availability_display(self.model, spec), false)
        )
    }

    /// A list of sibling blocks as chunks: one string per block (a directive
    /// that wraps the next block is written with it).
    fn blocks(&self, blocks: &[ResolvedBlock]) -> Vec<String> {
        let mut out = Vec::new();
        let mut pending: Vec<&ResolvedBlock> = Vec::new();
        let mut prev = Prev::None;
        for block in blocks {
            if is_following(block) {
                pending.push(block);
                continue;
            }
            let mut chunks = self.block(block, prev);
            let mut wrapped_only_by_transparent = true;
            while let Some(directive) = pending.pop() {
                wrapped_only_by_transparent &=
                    matches!(leaf_name(directive), Some("steps" | "available"));
                chunks = self.apply(directive, chunks);
            }
            prev = match &block.kind {
                ResolvedKind::List { ordered, .. } if wrapped_only_by_transparent => {
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
            chunks = self.apply(directive, chunks);
        }
        out.extend(chunks);
        out
    }

    fn block(&self, block: &ResolvedBlock, prev: Prev) -> Vec<String> {
        match &block.kind {
            ResolvedKind::Leaf(b) => self.leaf(block, &b.kind),
            ResolvedKind::BlockQuote { children } => {
                vec![quote(&self.blocks(children).join("\n\n"))]
            }
            ResolvedKind::List {
                ordered,
                start,
                tight,
                items,
            } => {
                let bodies: Vec<Vec<String>> =
                    items.iter().map(|i| self.blocks(&i.children)).collect();
                vec![list(*ordered, *start, *tight, &bodies, prev)]
            }
            ResolvedKind::Container {
                opener, children, ..
            } => self.container(block, opener, children),
            ResolvedKind::Group { name, arms, .. } => {
                let mut out = Vec::new();
                let widget = self.model.widget(name);
                if let Some(widget) = widget {
                    out.extend(self.fallback(widget));
                    if widget.plain_content == PlainContent::Drop {
                        return out;
                    }
                }
                for arm in arms {
                    let label = self.arm_label(block, &arm.opener);
                    if let Some(label) = &label {
                        out.push(format!("**{label}**"));
                        self.arms.borrow_mut().push(label.clone());
                    }
                    out.extend(self.blocks(&arm.children));
                    if label.is_some() {
                        self.arms.borrow_mut().pop();
                    }
                }
                out
            }
        }
    }

    fn leaf(&self, block: &ResolvedBlock, kind: &BlockKind) -> Vec<String> {
        match kind {
            BlockKind::Heading(h) => {
                let text = self.inlines(
                    block,
                    &h.inlines,
                    Style {
                        flat: false,
                        one_line: true,
                    },
                );
                // A heading in an arm says which: "Install the CLI (pnpm)".
                let arms = self.arms.borrow();
                let text = if arms.is_empty() || text.is_empty() {
                    escape_closing_hash(text)
                } else {
                    format!("{} ({})", escape_closing_hash(text), arms.join(", "))
                };
                let marks = "#".repeat(usize::from(h.level.clamp(1, 6)));
                vec![if text.is_empty() {
                    marks
                } else {
                    format!("{marks} {text}")
                }]
            }
            BlockKind::Paragraph(p) => {
                vec![self.inlines(block, &p.inlines, Style::default())]
            }
            BlockKind::CodeBlock(c) => vec![fenced(c)],
            BlockKind::HtmlBlock(h) => {
                // An HTML block with no text (a comment, a script) leaves
                // nothing behind.
                let text = html_as_text(&h.literal);
                if text.is_empty() {
                    Vec::new()
                } else {
                    vec![text]
                }
            }
            BlockKind::ThematicBreak => vec!["---".to_owned()],
            BlockKind::Table(t) => vec![self.table(block, t)],
            BlockKind::Directive(line) => self.directive(block, line),
            BlockKind::BlockQuote(_)
            | BlockKind::List(_)
            | BlockKind::End(_)
            | BlockKind::Container(_)
            | BlockKind::Group(_)
            | BlockKind::Title(_) => Vec::new(),
        }
    }

    /// A directive line that stands alone (it wraps no following block).
    fn directive(&self, block: &ResolvedBlock, line: &DirectiveLine) -> Vec<String> {
        match line.name.as_str() {
            "id" | "include" | "snippet" | "steps" | "details" => Vec::new(),
            "available" => self.annotation(block).into_iter().collect(),
            "note" => match &line.primary {
                Some(PrimaryValue::Text(text)) => {
                    let content = self.inlines(block, &text.inlines, Style::default());
                    vec![self.note(block, line, &[content])]
                }
                _ => Vec::new(),
            },
            name => match self.model.widget(name) {
                Some(widget) => self.fallback(widget),
                None => Vec::new(),
            },
        }
    }

    /// A following-block directive applied to the block it binds.
    fn apply(&self, directive: &ResolvedBlock, bound: Vec<String>) -> Vec<String> {
        let ResolvedKind::Leaf(b) = &directive.kind else {
            return bound;
        };
        let BlockKind::Directive(line) = &b.kind else {
            return bound;
        };
        match line.name.as_str() {
            "note" => vec![self.note(directive, line, &bound)],
            "details" => {
                let mut out = vec![self.bold_title(directive, line)];
                out.extend(bound);
                out
            }
            "available" => {
                let mut out: Vec<String> = self.annotation(directive).into_iter().collect();
                out.extend(bound);
                out
            }
            name => match self.model.widget(name) {
                Some(widget) => {
                    let mut out = self.fallback(widget);
                    if widget.plain_content == PlainContent::Keep {
                        out.extend(bound);
                    }
                    out
                }
                None => bound,
            },
        }
    }

    fn container(
        &self,
        block: &ResolvedBlock,
        opener: &DirectiveLine,
        children: &[ResolvedBlock],
    ) -> Vec<String> {
        let inner = self.blocks(children);
        match opener.name.as_str() {
            "note" => vec![self.note(block, opener, &inner)],
            "details" => {
                let mut out = vec![self.bold_title(block, opener)];
                out.extend(inner);
                out
            }
            name => match self.model.widget(name) {
                Some(widget) => {
                    let mut out = self.fallback(widget);
                    if widget.plain_content == PlainContent::Keep {
                        out.extend(inner);
                    }
                    out
                }
                None => inner,
            },
        }
    }

    /// The `Available:` line of a surviving `@available` directive.
    fn annotation(&self, block: &ResolvedBlock) -> Option<String> {
        let annotation = block.annotation.as_ref()?;
        let shown = availability_display_of_text(self.model, &annotation.text)
            .unwrap_or_else(|| annotation.text.clone());
        Some(format!("Available: {}", escape(&shown, false)))
    }

    /// A note: a blockquote beginning with its type's label, and its title.
    fn note(&self, block: &ResolvedBlock, line: &DirectiveLine, content: &[String]) -> String {
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
        let head = match self.title_text(block, line) {
            Some(title) => format!("**{label}: {title}**"),
            None => format!("**{label}**"),
        };
        let mut chunks = vec![head];
        chunks.extend(content.iter().cloned());
        quote(&chunks.join("\n\n"))
    }

    fn bold_title(&self, block: &ResolvedBlock, line: &DirectiveLine) -> String {
        match self.title_text(block, line) {
            Some(title) => format!("**{title}**"),
            None => String::new(),
        }
    }

    /// A directive's title, written to go inside bold.
    fn title_text(&self, block: &ResolvedBlock, line: &DirectiveLine) -> Option<String> {
        let title = line.title.as_ref()?;
        let text = self.inlines(
            block,
            &title.inlines,
            Style {
                flat: true,
                one_line: true,
            },
        );
        (!text.trim().is_empty()).then_some(text)
    }

    /// The label that leads an arm's section: its title, or the display labels
    /// of the values it names.
    fn arm_label(&self, block: &ResolvedBlock, opener: &DirectiveLine) -> Option<String> {
        self.title_text(block, opener)
            .or_else(|| dimensional_label(self.model, opener).map(|l| escape(&l, false)))
    }

    /// A widget's plain fallback, with phrases substituted. The widget's own
    /// title, primary, and attributes aren't shown.
    fn fallback(&self, widget: &ascribe_model::Widget) -> Vec<String> {
        widget
            .plain_fallback
            .as_deref()
            .map(|text| substitute_phrases(self.model, text))
            .filter(|text| !text.trim().is_empty())
            .into_iter()
            .collect()
    }

    fn table(&self, block: &ResolvedBlock, table: &Table) -> String {
        let cell = |links: &[ResolvedLink], inlines: &[ascribe_syntax::Inline]| {
            inline::render(
                self,
                links,
                inlines,
                Style {
                    flat: false,
                    one_line: true,
                },
            )
            .replace('|', "\\|")
        };
        let width = table.rows.first().map_or(0, |r| r.cells.len());
        let mut lines = Vec::new();
        for (n, row) in table.rows.iter().enumerate() {
            let mut cells: Vec<String> = row
                .cells
                .iter()
                .take(width)
                .map(|c| cell(&block.links, &c.inlines))
                .collect();
            cells.resize(width, String::new());
            // A row's availability ends its first cell.
            if let Some(r) = block.rows.iter().find(|r| r.span == row.span)
                && let Some(first) = cells.first_mut()
            {
                let shown = availability_display(self.model, &r.availability.spec);
                let note = format!("(Available: {})", escape(&shown, false).replace('|', "\\|"));
                if first.is_empty() {
                    *first = note;
                } else {
                    first.push(' ');
                    first.push_str(&note);
                }
            }
            lines.push(format!("| {} |", cells.join(" | ")));
            if n == 0 {
                // Each column keeps its alignment.
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

fn leaf_name(block: &ResolvedBlock) -> Option<&str> {
    match &block.kind {
        ResolvedKind::Leaf(b) => match &b.kind {
            BlockKind::Directive(line) => Some(line.name.as_str()),
            _ => None,
        },
        _ => None,
    }
}

/// A heading's text with a trailing `#` escaped, so it isn't read as a
/// closing sequence. A `#` that is already escaped is left alone.
pub(crate) fn escape_closing_hash(text: String) -> String {
    let Some(rest) = text.strip_suffix('#') else {
        return text;
    };
    let backslashes = rest.chars().rev().take_while(|c| *c == '\\').count();
    if backslashes % 2 == 1 {
        text
    } else {
        format!("{rest}\\#")
    }
}

/// `> ` before each line; an empty line is just `>`.
pub(crate) fn quote(text: &str) -> String {
    text.lines()
        .map(|l| {
            if l.is_empty() {
                ">".to_owned()
            } else {
                format!("> {l}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A list. Each item's blocks are joined with a blank line, except that a
/// tight list keeps a nested list or a code block directly under its text.
pub(crate) fn list(
    ordered: bool,
    start: Option<u64>,
    tight: bool,
    bodies: &[Vec<String>],
    prev: Prev,
) -> String {
    let mut items = Vec::new();
    for (n, chunks) in bodies.iter().enumerate() {
        let marker = if ordered {
            let delimiter = if prev == Prev::Ordered { ')' } else { '.' };
            format!("{}{delimiter}", start.unwrap_or(1).saturating_add(n as u64))
        } else if prev == Prev::Bullet {
            "*".to_owned()
        } else {
            "-".to_owned()
        };
        let mut body = String::new();
        for (i, chunk) in chunks.iter().enumerate() {
            if i > 0 {
                body.push_str(if tight && starts_nested(chunk) {
                    "\n"
                } else {
                    "\n\n"
                });
            }
            body.push_str(chunk);
        }
        let pad = " ".repeat(marker.chars().count() + 1);
        let mut out = marker;
        for (i, line) in body.split('\n').enumerate() {
            if i == 0 {
                if !line.is_empty() {
                    out.push(' ');
                    out.push_str(line);
                }
            } else if line.is_empty() {
                out.push('\n');
            } else {
                out.push('\n');
                out.push_str(&pad);
                out.push_str(line);
            }
        }
        items.push(out);
    }
    items.join(if tight { "\n" } else { "\n\n" })
}

/// Whether a chunk starts a list or a code fence, which can sit directly under
/// a paragraph without a blank line. A site output chunk's source anchor, on
/// the line before, can too (an HTML comment can interrupt a paragraph), so
/// the line after it decides.
pub(crate) fn starts_nested(chunk: &str) -> bool {
    let mut lines = chunk.lines();
    let mut first = lines.next().unwrap_or("");
    if crate::site::starts_anchor(first) {
        first = lines.next().unwrap_or("");
    }
    let digits = first.chars().take_while(char::is_ascii_digit).count();
    first.starts_with("```")
        || first.starts_with("~~~")
        || first.starts_with("- ")
        || first.starts_with("* ")
        || (digits > 0 && matches!(first[digits..].chars().next(), Some('.' | ')')))
}

/// A code block as a fenced one: the fence is longer than any run of backticks
/// in the code. A `phrases=true` info word has done its work and is dropped.
pub(crate) fn fenced(code: &CodeBlock) -> String {
    let info = code
        .info
        .split_whitespace()
        .filter(|word| *word != "phrases=true")
        .collect::<Vec<_>>()
        .join(" ");
    let mut longest = 0;
    let mut run = 0;
    for ch in code.literal.chars() {
        if ch == '`' {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    let fence_char = if info.contains('`') { '~' } else { '`' };
    let mut length = 3;
    if fence_char == '`' {
        length = length.max(longest + 1);
    } else {
        // Tildes: any run of them at the start of a line.
        let longest_tildes = code
            .literal
            .lines()
            .map(|l| l.trim_start().chars().take_while(|c| *c == '~').count())
            .max()
            .unwrap_or(0);
        length = length.max(longest_tildes + 1);
    }
    let fence: String = std::iter::repeat_n(fence_char, length).collect();
    let literal = code.literal.strip_suffix('\n').unwrap_or(&code.literal);
    if literal.is_empty() {
        format!("{fence}{info}\n{fence}")
    } else {
        format!("{fence}{info}\n{literal}\n{fence}")
    }
}

/// Raw HTML in the plain output, which has no HTML: its text
/// is kept and its tags are dropped, so `<kbd>Ctrl</kbd>` reads `Ctrl`.
/// Comments, and the contents of `<script>` and `<style>`, aren't text and are
/// dropped too. Each line is escaped, so a parser reads it as prose.
fn html_as_text(html: &str) -> String {
    html_text(html)
        .lines()
        .map(|l| escape(l.trim(), true))
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The text of raw HTML: tags, comments, and `<script>` and `<style>`
/// contents removed.
pub(crate) fn html_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(open) = rest.find('<') {
        out.push_str(&rest[..open]);
        let tag = &rest[open..];
        if let Some(after) = tag.strip_prefix("<!--") {
            rest = after.find("-->").map_or("", |end| &after[end + 3..]);
            continue;
        }
        // A tag starts with `<` and a letter, `/`, `!`, or `?`; any other
        // `<` is text, as is one with no `>` after it.
        let starts_tag = tag[1..]
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || matches!(c, '/' | '!' | '?'));
        let close = if starts_tag { tag.find('>') } else { None };
        let Some(close) = close else {
            out.push('<');
            rest = &tag[1..];
            continue;
        };
        let name: String = tag[1..close]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        rest = &tag[close + 1..];
        if name == "script" || name == "style" {
            let end = format!("</{name}");
            let lower = rest.to_ascii_lowercase();
            rest = match lower.find(&end) {
                Some(at) => rest[at..].find('>').map_or("", |gt| &rest[at + gt + 1..]),
                None => "",
            };
        }
    }
    out.push_str(rest);
    out
}

/// Replaces each `{key}` that names a declared phrase with its value.
fn substitute_phrases(model: &ContentModel, text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find('}').map(|close| (close, &after[..close])) {
            Some((close, key)) if model.has_phrase(key) => {
                out.push_str(model.phrase(key).unwrap_or(""));
                rest = &after[close + 1..];
            }
            _ => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}
