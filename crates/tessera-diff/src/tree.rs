//! A resolved page as the comparison sees it: a tree of blocks, each with
//! its source anchor and a fingerprint of its content without positions.
//!
//! Positions (byte spans, lines) never reach a fingerprint, so a block that
//! only moved down the file, or was rewrapped, fingerprints the same. Text is
//! compared with its whitespace collapsed, as a reader sees it. What a
//! reader sees besides the text is in the fingerprint too: a link's resolved
//! URL, an availability badge's labels, a glossary link, an arm's label.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use serde::Serialize;
use tessera_core::{FileId, LineIndex, RelPath, Span};
use tessera_emit::labels::{attribute_values, availability_display, dimensional_label};
use tessera_model::ContentModel;
use tessera_resolve::{
    Availability, IncludeSite, LinkTarget, Project, RefKind, ResolvedBlock, ResolvedKind,
    ResolvedPage, Scope,
};
use tessera_syntax::{
    Alignment, BlockKind, Bound, DirectiveLine, Inline, InlineKind, PrimaryValue,
};

/// Where a block's text is written: the README's anchor grammar, the same
/// string the rendered page carries in `data-ascribe-source` and
/// `data-ascribe-via`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Anchor {
    /// `<path>:<first>-<last>`: the file's content path, percent-encoded by
    /// segment, and the block's first and last lines, from 1.
    pub source: String,
    /// The includes the block came through, outermost first, each
    /// `<path>:<line>`. Empty for a block written in the page itself.
    pub via: Vec<String>,
}

/// A block of a page, or a list item or a group's arm, for comparing.
#[derive(Clone, Debug)]
pub(crate) struct Node {
    /// Where it's written.
    pub anchor: Anchor,
    /// What kind of node it is. Only nodes of one kind are paired as a
    /// change of one into the other.
    pub kind: String,
    /// Its own content, without positions and without its children's.
    pub own: String,
    /// Its own text as a reader reads it, whitespace collapsed.
    pub text: String,
    /// Whether its text is prose, which gets a word-level diff.
    pub prose: bool,
    /// The nodes inside it.
    pub children: Vec<Node>,
    /// A fingerprint of `own` and every child's fingerprint.
    pub hash: u64,
    /// The files its own content comes from: the file it's written in, the
    /// files its includes are in, and the pages its links take text or
    /// headings from.
    pub files: Vec<RelPath>,
    /// Whether the content model shapes its own content beyond the text
    /// written: a phrase, a glossary link, an availability label, or an
    /// arm's label.
    pub uses_model: bool,
    /// For a code block a `@snippet` became, its address and a fingerprint
    /// of its code.
    pub snippet: Option<(String, u64)>,
}

impl Node {
    /// Its text and every child's, for judging how similar two nodes are.
    pub fn all_text(&self) -> String {
        let mut out = self.text.clone();
        for child in &self.children {
            let text = child.all_text();
            if !text.is_empty() {
                if !out.is_empty() {
                    out.push(' ');
                }
                out.push_str(&text);
            }
        }
        out
    }

    /// Calls `f` on this node and every node inside it.
    pub fn visit<'a>(&'a self, f: &mut impl FnMut(&'a Node)) {
        f(self);
        for child in &self.children {
            child.visit(f);
        }
    }
}

/// A page as the comparison sees it.
#[derive(Clone, Debug)]
pub(crate) struct PageTree {
    /// The page's route.
    pub route: String,
    /// Its title, phrases substituted.
    pub title: Option<String>,
    /// Its frontmatter, serialized, without `title` and `available`.
    pub frontmatter: Option<String>,
    /// Its page-level availability, as shown.
    pub availability: Option<String>,
    /// Its blocks.
    pub nodes: Vec<Node>,
    /// A fingerprint of the whole page: its title, frontmatter, page-level
    /// availability, and every block's fingerprint.
    pub hash: u64,
}

/// Builds [`PageTree`]s for one side of a comparison.
pub(crate) struct TreeBuilder<'p> {
    project: &'p Project,
    model: &'p ContentModel,
    lines: HashMap<FileId, LineIndex>,
}

impl<'p> TreeBuilder<'p> {
    pub fn new(project: &'p Project) -> TreeBuilder<'p> {
        TreeBuilder {
            project,
            model: project.model(),
            lines: HashMap::new(),
        }
    }

    pub fn page(&mut self, page: &ResolvedPage) -> PageTree {
        let nodes = self.blocks(&page.blocks);
        // YAML values don't implement `Hash`; their serialization is stable.
        // `title` and `available` are left out: they're compared as shown,
        // as the title and the availability.
        let frontmatter = page.frontmatter.as_ref().and_then(|f| {
            let mut f = f.clone();
            if let Some(map) = f.as_mapping_mut() {
                map.remove("title");
                map.remove("available");
            }
            serde_yaml_ng::to_string(&f).ok()
        });
        let availability = page.availability.as_deref().map(|a| self.availability(a));
        let mut hasher = DefaultHasher::new();
        page.title.hash(&mut hasher);
        frontmatter.hash(&mut hasher);
        availability.hash(&mut hasher);
        for node in &nodes {
            node.hash.hash(&mut hasher);
        }
        PageTree {
            route: page.route.clone(),
            title: page.title.clone(),
            frontmatter,
            availability,
            nodes,
            hash: hasher.finish(),
        }
    }

    /// Sibling blocks as nodes. A line-form directive that the page renders
    /// as an element around the block after it (`@note`, `@steps`,
    /// `@details`, a widget) is one node holding that block, anchored from
    /// the directive through the block, as the element is.
    fn blocks(&mut self, blocks: &[ResolvedBlock]) -> Vec<Node> {
        let mut out = Vec::new();
        let mut pending: Vec<&ResolvedBlock> = Vec::new();
        for block in blocks {
            if following(block).is_some() {
                pending.push(block);
                continue;
            }
            let mut nodes = vec![self.block(block)];
            while let Some(directive) = pending.pop() {
                let mut node = self.block(directive);
                if !self.wraps(directive) {
                    nodes.insert(0, node);
                    continue;
                }
                if block.file == directive.file && block.via == directive.via {
                    node.anchor = self.anchor(
                        directive.file,
                        directive.span.cover(block.span),
                        &directive.via,
                    );
                }
                node.children = std::mem::take(&mut nodes);
                node.hash = fingerprint(&node.own, &node.children);
                nodes = vec![node];
            }
            out.extend(nodes);
        }
        // Directives with no block after them (an error the checks report).
        out.extend(pending.into_iter().map(|d| self.block(d)));
        out
    }

    /// Whether a line-form directive renders as an element around its block.
    fn wraps(&self, directive: &ResolvedBlock) -> bool {
        following(directive).is_some_and(|line| {
            matches!(line.name.as_str(), "note" | "steps" | "details")
                || self.model.widget(&line.name).is_some()
        })
    }

    fn block(&mut self, block: &ResolvedBlock) -> Node {
        let anchor = self.anchor(block.file, block.span, &block.via);
        let mut files = self.files_of(block.file, &block.via);
        for link in &block.links {
            // A linked page shapes this block only through what the link
            // takes from it: its title as the link's text, or a heading's id.
            if let LinkTarget::Page {
                page,
                id,
                text_filled,
                ..
            } = &link.target
                && (*text_filled || id.is_some())
            {
                files.push(page.clone());
            }
        }
        // A snippet's substituted phrases aren't listed (they're in the code
        // file), so one that opts in to phrases is taken to use the model.
        let snippet_phrases = block
            .snippet
            .as_ref()
            .is_some_and(|s| s.info.split_whitespace().any(|w| w == "phrases=true"));
        let mut uses_model =
            !block.substitutions.is_empty() || !block.glossary.is_empty() || snippet_phrases;
        let mut own = String::new();
        let mut text = String::new();
        let mut prose = false;
        let mut children = Vec::new();
        let kind;
        match &block.kind {
            ResolvedKind::Leaf(leaf) => match &leaf.kind {
                BlockKind::Heading(h) => {
                    kind = format!("heading{}", h.level);
                    own.push_str(&kind);
                    if let Some(ids) = block.heading.as_ref().filter(|i| i.explicit) {
                        own.push_str(&format!("#{}", ids.page_id));
                    }
                    self.inlines(block, &h.inlines, &mut own);
                    text = plain(&h.inlines);
                    prose = true;
                }
                BlockKind::Paragraph(p) => {
                    kind = "paragraph".into();
                    own.push_str(&kind);
                    self.inlines(block, &p.inlines, &mut own);
                    text = plain(&p.inlines);
                    prose = true;
                }
                BlockKind::CodeBlock(c) => {
                    kind = "code".into();
                    own.push_str(&format!("code\u{1}{}\u{1}{}", c.info, lf(&c.literal)));
                    text = collapse(&c.literal);
                }
                BlockKind::HtmlBlock(h) => {
                    kind = "html".into();
                    own.push_str(&format!("html\u{1}{}", lf(h.literal.trim_end())));
                    text = collapse(&h.literal);
                }
                BlockKind::ThematicBreak => {
                    kind = "break".into();
                    own.push_str(&kind);
                }
                BlockKind::Table(t) => {
                    kind = "table".into();
                    own.push_str(&kind);
                    for a in &t.alignments {
                        own.push(match a {
                            Alignment::None => 'n',
                            Alignment::Left => 'l',
                            Alignment::Center => 'c',
                            Alignment::Right => 'r',
                        });
                    }
                    let mut cells = Vec::new();
                    for row in &t.rows {
                        own.push_str(if row.header { "\u{1}H" } else { "\u{1}R" });
                        if let Some(r) = block.rows.iter().find(|r| r.span == row.span) {
                            own.push_str(&format!(
                                "\u{1}available\u{1}{}",
                                self.availability(&r.availability)
                            ));
                            uses_model = true;
                        }
                        for cell in &row.cells {
                            own.push('\u{2}');
                            self.inlines(block, &cell.inlines, &mut own);
                            cells.push(plain(&cell.inlines));
                        }
                    }
                    text = collapse(&cells.join(" "));
                }
                BlockKind::Directive(line) => {
                    kind = format!("directive:{}", line.name);
                    own.push_str(&kind);
                    self.directive_head(block, line, &mut own);
                    text = directive_text(line);
                    prose = !text.is_empty();
                    if let Some(annotation) = &block.annotation {
                        own.push_str(&format!("\u{1}@{}", annotation.text));
                        if let Ok(spec) =
                            tessera_core::availability::parse_availability(&annotation.text, 0)
                        {
                            own.push_str(&format!(
                                "\u{1}{}",
                                availability_display(self.model, &spec)
                            ));
                        }
                        uses_model = true;
                    }
                }
                // Not leaves in a resolved tree; kept as they are.
                other => {
                    kind = "other".into();
                    own.push_str(&format!("other\u{1}{other:?}"));
                }
            },
            ResolvedKind::BlockQuote { children: blocks } => {
                kind = "quote".into();
                own.push_str(&kind);
                children = self.blocks(blocks);
            }
            ResolvedKind::List {
                ordered,
                start,
                tight,
                items,
            } => {
                kind = if *ordered { "list:ordered" } else { "list" }.into();
                own.push_str(&format!("{kind}\u{1}{start:?}\u{1}{tight}"));
                children = items
                    .iter()
                    .map(|item| {
                        let blocks = self.blocks(&item.children);
                        self.wrapper(block, item.span, "item".into(), String::new(), blocks)
                    })
                    .collect();
            }
            ResolvedKind::Container {
                opener,
                children: blocks,
                ..
            } => {
                kind = format!("container:{}", opener.name);
                own.push_str(&kind);
                self.directive_head(block, opener, &mut own);
                text = opener
                    .title
                    .as_ref()
                    .map(|t| plain(&t.inlines))
                    .unwrap_or_default();
                children = self.blocks(blocks);
            }
            ResolvedKind::Group { name, arms, .. } => {
                kind = format!("group:{name}");
                own.push_str(&kind);
                uses_model = true;
                children = arms
                    .iter()
                    .map(|arm| {
                        let mut head = String::new();
                        self.directive_head(block, &arm.opener, &mut head);
                        if let Some(label) = dimensional_label(self.model, &arm.opener) {
                            head.push_str(&format!("\u{1}{label}"));
                        }
                        let blocks = self.blocks(&arm.children);
                        self.wrapper(block, arm.span, "arm".into(), head, blocks)
                    })
                    .collect();
            }
        }
        if let Some(a) = block
            .availability
            .as_deref()
            .filter(|a| a.scope != Scope::Page)
        {
            own.push_str(&format!("\u{1}available\u{1}{}", self.availability(a)));
            uses_model = true;
        }
        for g in &block.glossary {
            own.push_str(&format!("\u{1}term\u{1}{}\u{1}{}", g.text, g.url));
        }
        let hash = fingerprint(&own, &children);
        files.sort();
        files.dedup();
        Node {
            anchor,
            kind,
            own,
            text,
            prose,
            children,
            hash,
            files,
            uses_model,
            snippet: block.snippet.as_ref().map(|s| {
                let mut hasher = DefaultHasher::new();
                lf(&s.code).hash(&mut hasher);
                (s.address.clone(), hasher.finish())
            }),
        }
    }

    /// A list item or an arm: a node written in its list's or group's file,
    /// at its own span.
    fn wrapper(
        &mut self,
        parent: &ResolvedBlock,
        span: Span,
        kind: String,
        head: String,
        children: Vec<Node>,
    ) -> Node {
        let own = format!("{kind}{head}");
        let hash = fingerprint(&own, &children);
        Node {
            anchor: self.anchor(parent.file, span, &parent.via),
            kind,
            own,
            text: String::new(),
            prose: false,
            children,
            hash,
            files: self.files_of(parent.file, &parent.via),
            uses_model: false,
            snippet: None,
        }
    }

    fn files_of(&self, file: FileId, via: &[IncludeSite]) -> Vec<RelPath> {
        std::iter::once(file)
            .chain(via.iter().map(|s| s.file))
            .filter_map(|f| self.project.path_of(f).cloned())
            .collect()
    }

    fn line_of(&mut self, file: FileId, offset: usize) -> u32 {
        let index = match self.lines.get(&file) {
            Some(index) => index,
            None => {
                let source = self
                    .project
                    .file_by_id(file)
                    .map(|f| &*f.source)
                    .unwrap_or_default();
                self.lines
                    .entry(file)
                    .or_insert_with(|| LineIndex::new(source))
            }
        };
        index.line_col(offset).map_or(1, |p| p.line + 1)
    }

    fn anchor(&mut self, file: FileId, span: Span, via: &[IncludeSite]) -> Anchor {
        let first = self.line_of(file, span.start());
        let last = self.line_of(file, span.end()).max(first);
        let path = self.path(file);
        Anchor {
            source: format!("{path}:{first}-{last}"),
            via: via
                .iter()
                .map(|site| {
                    let line = self.line_of(site.file, site.span.start());
                    format!("{}:{line}", self.path(site.file))
                })
                .collect(),
        }
    }

    fn path(&self, file: FileId) -> String {
        self.project
            .path_of(file)
            .map(encode_path)
            .unwrap_or_default()
    }

    fn availability(&self, a: &Availability) -> String {
        let mut out = availability_display(self.model, &a.spec);
        if let Some(enclosing) = a.enclosing.as_deref().filter(|e| e.scope != Scope::Page) {
            out.push_str(" / ");
            out.push_str(&self.availability(enclosing));
        }
        out
    }

    /// A directive line's name, attributes, title, and primary.
    fn directive_head(&self, block: &ResolvedBlock, line: &DirectiveLine, out: &mut String) {
        for (key, values) in attribute_values(line.attributes.as_ref()) {
            out.push_str(&format!("\u{1}{key}={}", values.join(",")));
        }
        if let Some(title) = &line.title {
            out.push_str("\u{1}title");
            self.inlines(block, &title.inlines, out);
        }
        match &line.primary {
            Some(PrimaryValue::Text(t)) => {
                out.push_str("\u{1}text");
                self.inlines(block, &t.inlines, out);
            }
            Some(PrimaryValue::Identifier(i)) => out.push_str(&format!("\u{1}id {}", i.text)),
            Some(PrimaryValue::Line(l)) => out.push_str(&format!("\u{1}line {}", l.text)),
            Some(PrimaryValue::Unexpected(_)) | None => {}
        }
    }

    /// Inline content without positions, whitespace collapsed, with links
    /// as their resolved URLs.
    fn inlines(&self, block: &ResolvedBlock, list: &[Inline], out: &mut String) {
        let mut raw = String::new();
        self.push_inlines(block, list, &mut raw);
        out.push('\u{1}');
        out.push_str(&collapse(&raw));
    }

    fn push_inlines(&self, block: &ResolvedBlock, list: &[Inline], out: &mut String) {
        for node in list {
            match &node.kind {
                InlineKind::Text(v) => out.push_str(v),
                InlineKind::Code(v) => {
                    out.push_str("\u{2}c");
                    out.push_str(v);
                    out.push('\u{3}');
                }
                InlineKind::SoftBreak => out.push(' '),
                InlineKind::HardBreak => out.push_str("\u{2}br\u{3}"),
                InlineKind::Html(v) => {
                    out.push_str("\u{2}h");
                    out.push_str(v);
                    out.push('\u{3}');
                }
                InlineKind::Emphasis(c) => {
                    out.push_str("\u{2}em");
                    self.push_inlines(block, c, out);
                    out.push('\u{3}');
                }
                InlineKind::Strong(c) => {
                    out.push_str("\u{2}strong");
                    self.push_inlines(block, c, out);
                    out.push('\u{3}');
                }
                InlineKind::Link(link) => {
                    out.push_str("\u{2}a");
                    out.push_str(&self.url(block, node.span, RefKind::Link, &link.destination));
                    if let Some(title) = &link.title {
                        out.push_str(&format!("\u{4}{title}"));
                    }
                    out.push('\u{4}');
                    self.push_inlines(block, &link.children, out);
                    out.push('\u{3}');
                }
                InlineKind::Image(image) => {
                    out.push_str("\u{2}img");
                    out.push_str(&self.url(block, node.span, RefKind::Image, &image.destination));
                    if let Some(title) = &image.title {
                        out.push_str(&format!("\u{4}{title}"));
                    }
                    for (key, values) in
                        attribute_values(image.attributes.as_ref().map(|a| &a.block))
                    {
                        out.push_str(&format!("\u{4}{key}={}", values.join(",")));
                    }
                    out.push('\u{4}');
                    out.push_str(&plain(&image.children));
                    out.push('\u{3}');
                }
                // Substituted by the build; an undeclared one is text.
                InlineKind::Phrase(p) => out.push_str(&format!("{{{}}}", p.key)),
            }
        }
    }

    /// Where a link or image goes, as the build resolved it.
    fn url(&self, block: &ResolvedBlock, span: Span, kind: RefKind, written: &str) -> String {
        let resolved = block
            .links
            .iter()
            .find(|l| l.span == span && l.kind == kind);
        match resolved.map(|l| &l.target) {
            Some(LinkTarget::Page { url, .. }) => url.clone(),
            Some(LinkTarget::Asset { path, fragment }) => match fragment {
                Some(f) => format!("{path}#{f}"),
                None => path.to_string(),
            },
            _ => written.to_owned(),
        }
    }
}

/// The directive line of a line-form directive that binds the next block.
fn following(block: &ResolvedBlock) -> Option<&DirectiveLine> {
    match &block.kind {
        ResolvedKind::Leaf(leaf) => match &leaf.kind {
            BlockKind::Directive(line) if line.binding == Some(Bound::FollowingBlock) => Some(line),
            _ => None,
        },
        _ => None,
    }
}

fn fingerprint(own: &str, children: &[Node]) -> u64 {
    let mut hasher = DefaultHasher::new();
    own.hash(&mut hasher);
    children.len().hash(&mut hasher);
    for child in children {
        child.hash.hash(&mut hasher);
    }
    hasher.finish()
}

/// The text of a directive line as a reader reads it: its title and its
/// primary.
fn directive_text(line: &DirectiveLine) -> String {
    let mut parts = Vec::new();
    if let Some(title) = &line.title {
        parts.push(plain(&title.inlines));
    }
    match &line.primary {
        Some(PrimaryValue::Text(t)) => parts.push(plain(&t.inlines)),
        Some(PrimaryValue::Identifier(i)) => parts.push(i.text.clone()),
        Some(PrimaryValue::Line(l)) => parts.push(l.text.clone()),
        Some(PrimaryValue::Unexpected(_)) | None => {}
    }
    collapse(&parts.join(" "))
}

/// Inline content as plain text, whitespace collapsed: what the word-level
/// diff's ranges count in.
pub(crate) fn plain(list: &[Inline]) -> String {
    fn push(list: &[Inline], out: &mut String) {
        for node in list {
            match &node.kind {
                InlineKind::Text(v) | InlineKind::Code(v) | InlineKind::Html(v) => {
                    out.push_str(v);
                }
                InlineKind::SoftBreak | InlineKind::HardBreak => out.push(' '),
                InlineKind::Emphasis(c) | InlineKind::Strong(c) => push(c, out),
                InlineKind::Link(link) => push(&link.children, out),
                InlineKind::Image(image) => push(&image.children, out),
                InlineKind::Phrase(p) => out.push_str(&format!("{{{}}}", p.key)),
            }
        }
    }
    let mut out = String::new();
    push(list, &mut out);
    collapse(&out)
}

/// Text with every `\r\n` as `\n`: a working copy checked out with CRLF line
/// endings (Git for Windows' default) reads the same as the LF blob it came
/// from.
pub(crate) fn lf(text: &str) -> std::borrow::Cow<'_, str> {
    if text.contains("\r\n") {
        std::borrow::Cow::Owned(text.replace("\r\n", "\n"))
    } else {
        std::borrow::Cow::Borrowed(text)
    }
}

/// Runs of whitespace as one space, and none at either end.
pub(crate) fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A content path as the anchor grammar writes it: `/` between segments,
/// and each segment percent-encoded except ASCII letters, digits, `-`, `.`,
/// `_`, and `~`.
pub fn encode_path(path: &RelPath) -> String {
    let mut out = String::with_capacity(path.as_str().len());
    for (i, segment) in path.segments().enumerate() {
        if i > 0 {
            out.push('/');
        }
        for byte in segment.bytes() {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
                out.push(char::from(byte));
            } else {
                out.push_str(&format!("%{byte:02X}"));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_are_encoded_by_segment() {
        let path = RelPath::parse("guides/my page/é:x.md").unwrap();
        assert_eq!(encode_path(&path), "guides/my%20page/%C3%A9%3Ax.md");
        let path = RelPath::parse("../shared/a-b_c~d.md").unwrap();
        assert_eq!(encode_path(&path), "../shared/a-b_c~d.md");
    }

    #[test]
    fn whitespace_collapses() {
        assert_eq!(collapse("  a \n b\t\tc "), "a b c");
    }
}
