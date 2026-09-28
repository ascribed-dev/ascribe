//! Include expansion (SPEC §4.2, §9.2 step 1).
//!
//! [`Project::expand`] replaces each `@include` in a file with its target,
//! recursively, and gives the result as an [`ExpandedPage`]: the file's blocks
//! with the included blocks in place of the directive. The expansion is
//! independent of any build: availability, variants, and phrases are applied
//! afterwards (phase 12).
//!
//! Every block in the result keeps the file it was written in and its span
//! there ([`ExpandedBlock::file`], [`ExpandedBlock::span`]), and the chain of
//! includes it came through ([`ExpandedBlock::via`]). Relative references in
//! a block resolve from that file, not from the page (SPEC §4.2).

use std::sync::Arc;

use tessera_core::{FileId, Issue, Location, RelPath, Span, diagnostics};
use tessera_syntax::{Block, BlockKind, DirectiveLine, EndLine};

use crate::index::{FileIndex, Heading, Include, walk};
use crate::project::Project;

/// An `@include` directive an expanded block came through.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IncludeSite {
    /// The file the directive is written in.
    pub file: FileId,
    /// The directive line.
    pub span: Span,
}

/// A file with its includes expanded.
#[derive(Clone, Debug)]
pub struct ExpandedPage {
    /// The file's content path.
    pub path: RelPath,
    /// The file's id.
    pub file: FileId,
    /// The blocks, with each include replaced by its content.
    pub blocks: Vec<ExpandedBlock>,
    /// Problems found while expanding, in the order they were met.
    pub problems: Vec<PageProblem>,
}

/// A problem found while expanding a page.
///
/// The issue is located where its cause is written, which may be inside a
/// fragment. A page-level report goes at the outermost include site, `via[0]`
/// (SPEC §8.1), with the issue's own location as related information; an
/// include cycle is reported where it closes, in the file containing that
/// include (Q20).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageProblem {
    /// The problem: `include-id-missing` or `include-cycle`.
    pub issue: Issue,
    /// The includes between the page and the file the issue is in, outermost
    /// first. Empty when the issue is in the page's own file.
    pub via: Arc<[IncludeSite]>,
}

/// A block of an expanded page: a block of one source file.
#[derive(Clone, Debug, PartialEq)]
pub struct ExpandedBlock {
    /// The file the block is written in.
    pub file: FileId,
    /// The block's span in that file.
    pub span: Span,
    /// The includes the block came through, outermost first: the first is in
    /// the page's own file. Empty for the page's own blocks.
    pub via: Arc<[IncludeSite]>,
    /// What the block is.
    pub kind: ExpandedKind,
}

/// The kinds of expanded block. A block that holds other blocks holds their
/// expansion; every other block is kept as it was parsed.
#[derive(Clone, Debug, PartialEq)]
pub enum ExpandedKind {
    /// A block with no nested blocks: a heading, paragraph, code block, HTML
    /// block, table, thematic break, or line-form directive. An `@include`
    /// that couldn't be expanded (its problem is recorded) stays as its
    /// directive.
    Leaf(Block),
    /// A block quote.
    BlockQuote {
        /// The blocks in it.
        children: Vec<ExpandedBlock>,
    },
    /// A list.
    List {
        /// Whether it's ordered.
        ordered: bool,
        /// The first number of an ordered list.
        start: Option<u64>,
        /// Whether it's tight.
        tight: bool,
        /// The items.
        items: Vec<ExpandedItem>,
    },
    /// A container directive.
    Container {
        /// The opener, with its title.
        opener: DirectiveLine,
        /// The end line, when it was closed.
        end: Option<EndLine>,
        /// The blocks in it.
        children: Vec<ExpandedBlock>,
    },
    /// A group of arms.
    Group {
        /// The groupable directive's name.
        name: String,
        /// The arms.
        arms: Vec<ExpandedArm>,
        /// The group's end line, when it was closed.
        end: Option<EndLine>,
    },
}

/// A list item of an expanded list.
#[derive(Clone, Debug, PartialEq)]
pub struct ExpandedItem {
    /// The item, from its marker through its last line.
    pub span: Span,
    /// The marker.
    pub marker: Span,
    /// The blocks in it.
    pub children: Vec<ExpandedBlock>,
}

/// An arm of an expanded group.
#[derive(Clone, Debug, PartialEq)]
pub struct ExpandedArm {
    /// The arm's span in its file.
    pub span: Span,
    /// The opener, with its title.
    pub opener: DirectiveLine,
    /// The blocks in it.
    pub children: Vec<ExpandedBlock>,
}

impl ExpandedBlock {
    /// The source ranges of this block's own inline content: the whole block
    /// for a leaf, and for a container or a group only the titles of its
    /// opener or arms, since the blocks inside are blocks of their own.
    pub fn own_ranges(&self) -> Vec<Span> {
        match &self.kind {
            ExpandedKind::Leaf(block) => walk::own_ranges(block),
            ExpandedKind::BlockQuote { .. } | ExpandedKind::List { .. } => Vec::new(),
            ExpandedKind::Container { opener, .. } => opener.title.iter().map(|t| t.span).collect(),
            ExpandedKind::Group { arms, .. } => arms
                .iter()
                .filter_map(|a| a.opener.title.as_ref().map(|t| t.span))
                .collect(),
        }
    }

    fn visit<'a>(&'a self, f: &mut impl FnMut(&'a ExpandedBlock)) {
        f(self);
        match &self.kind {
            ExpandedKind::Leaf(_) => {}
            ExpandedKind::BlockQuote { children } | ExpandedKind::Container { children, .. } => {
                children.iter().for_each(|c| c.visit(f));
            }
            ExpandedKind::List { items, .. } => {
                items
                    .iter()
                    .flat_map(|i| &i.children)
                    .for_each(|c| c.visit(f));
            }
            ExpandedKind::Group { arms, .. } => {
                arms.iter()
                    .flat_map(|a| &a.children)
                    .for_each(|c| c.visit(f));
            }
        }
    }
}

impl ExpandedPage {
    /// Calls `f` on every block, in document order, a block before the blocks
    /// inside it.
    pub fn visit<'a>(&'a self, f: &mut impl FnMut(&'a ExpandedBlock)) {
        self.blocks.iter().for_each(|b| b.visit(f));
    }

    /// Every heading on the expanded page, in document order, with the path
    /// of the file it's written in. A heading inside an included section that
    /// was cut off from its own heading (`heading=false`) is still listed.
    pub fn headings<'p>(&self, project: &'p Project) -> Vec<(RelPath, &'p Heading)> {
        let mut out = Vec::new();
        self.visit(&mut |block| {
            let ExpandedKind::Leaf(leaf) = &block.kind else {
                return;
            };
            if !matches!(leaf.kind, BlockKind::Heading(_)) {
                return;
            }
            let Some(index) = project.file_by_id(block.file) else {
                return;
            };
            if let Some(heading) = index.headings.iter().find(|h| h.span == block.span) {
                out.push((index.path.clone(), heading));
            }
        });
        out
    }
}

/// A file, and which of its sections is being expanded: the whole file, or the
/// section of the heading that starts at this offset.
type Key = (RelPath, Option<usize>);

/// Expands one page. It holds the stack of what's being expanded, so an
/// include that would expand something already on it is a cycle.
pub(crate) struct Expander<'p> {
    project: &'p Project,
    stack: Vec<Key>,
    problems: Vec<PageProblem>,
}

impl<'p> Expander<'p> {
    pub(crate) fn new(project: &'p Project) -> Expander<'p> {
        Expander {
            project,
            stack: Vec::new(),
            problems: Vec::new(),
        }
    }

    pub(crate) fn expand(mut self, path: &RelPath) -> Option<ExpandedPage> {
        let index = self.project.file(path)?;
        self.stack.push((path.clone(), None));
        let via: Arc<[IncludeSite]> = Arc::from(Vec::new());
        let blocks = self.blocks(index, &index.document.blocks, &via);
        Some(ExpandedPage {
            path: path.clone(),
            file: index.file,
            blocks,
            problems: self.problems,
        })
    }

    /// Expands one list of blocks, written in `file`.
    fn blocks(
        &mut self,
        file: &FileIndex,
        blocks: &[Block],
        via: &Arc<[IncludeSite]>,
    ) -> Vec<ExpandedBlock> {
        let mut out = Vec::with_capacity(blocks.len());
        for block in blocks {
            match &block.kind {
                BlockKind::Directive(line) if line.name == "include" => {
                    self.include(file, block, line, via, &mut out);
                }
                _ => out.push(self.block(file, block, via)),
            }
        }
        out
    }

    fn block(
        &mut self,
        file: &FileIndex,
        block: &Block,
        via: &Arc<[IncludeSite]>,
    ) -> ExpandedBlock {
        let kind = match &block.kind {
            BlockKind::BlockQuote(q) => ExpandedKind::BlockQuote {
                children: self.blocks(file, &q.children, via),
            },
            BlockKind::List(l) => ExpandedKind::List {
                ordered: l.ordered,
                start: l.start,
                tight: l.tight,
                items: l
                    .items
                    .iter()
                    .map(|item| ExpandedItem {
                        span: item.span,
                        marker: item.marker,
                        children: self.blocks(file, &item.children, via),
                    })
                    .collect(),
            },
            BlockKind::Container(c) => ExpandedKind::Container {
                opener: c.opener.clone(),
                end: c.end.clone(),
                children: self.blocks(file, &c.children, via),
            },
            BlockKind::Group(g) => ExpandedKind::Group {
                name: g.name.clone(),
                end: g.end.clone(),
                arms: g
                    .arms
                    .iter()
                    .map(|arm| ExpandedArm {
                        span: arm.span,
                        opener: arm.opener.clone(),
                        children: self.blocks(file, &arm.children, via),
                    })
                    .collect(),
            },
            _ => ExpandedKind::Leaf(block.clone()),
        };
        ExpandedBlock {
            file: file.file,
            span: block.span,
            via: via.clone(),
            kind,
        }
    }

    /// Replaces an `@include` with its target, or, when it can't be expanded,
    /// keeps the directive.
    fn include(
        &mut self,
        file: &FileIndex,
        block: &Block,
        line: &DirectiveLine,
        via: &Arc<[IncludeSite]>,
        out: &mut Vec<ExpandedBlock>,
    ) {
        let keep = |out: &mut Vec<ExpandedBlock>| {
            out.push(ExpandedBlock {
                file: file.file,
                span: block.span,
                via: via.clone(),
                kind: ExpandedKind::Leaf(block.clone()),
            });
        };
        let Some(include) = file.include_at(line.span) else {
            return keep(out);
        };
        let Some(target) = include.target.as_ref().and_then(|t| self.project.file(t)) else {
            // A missing file is a file-level problem (`Project::problems`).
            return keep(out);
        };

        // Which part of the target.
        let (blocks, key): (&[Block], Key) = match &include.section {
            None => (&target.document.blocks, (target.path.clone(), None)),
            Some(id) => {
                let found = target.heading_by_id(id).and_then(|heading| {
                    find_section(&target.document.blocks, heading.span.start())
                        .map(|blocks| (blocks, heading.span.start()))
                });
                match found {
                    Some((blocks, start)) => (blocks, (target.path.clone(), Some(start))),
                    None => {
                        self.report_id_missing(file, include, id, via);
                        return keep(out);
                    }
                }
            }
        };

        // SPEC-QUESTION(Q66): a cycle is expanding the same file, or the same
        // section of it, again while it's still being expanded.
        if let Some(at) = self.stack.iter().position(|k| *k == key) {
            self.report_cycle(file, include, at, via);
            return keep(out);
        }

        let mut chain: Vec<IncludeSite> = via.to_vec();
        chain.push(IncludeSite {
            file: file.file,
            span: include.span,
        });
        let chain: Arc<[IncludeSite]> = Arc::from(chain);

        // SPEC-QUESTION(Q65): `heading=false` drops the included section's own
        // heading; without an id there's no section heading, so it drops
        // nothing.
        let blocks = if include.section.is_some() && !include.heading {
            blocks.get(1..).unwrap_or_default()
        } else {
            blocks
        };
        self.stack.push(key);
        let expanded = self.blocks(target, blocks, &chain);
        self.stack.pop();
        out.extend(expanded);
    }

    fn report_id_missing(
        &mut self,
        file: &FileIndex,
        include: &Include,
        id: &str,
        via: &Arc<[IncludeSite]>,
    ) {
        let issue = Issue::new(
            diagnostics::INCLUDE_ID_MISSING,
            Location::new(file.file, include.primary.unwrap_or(include.span)),
        )
        .with_arg("path", include.written.clone())
        .with_arg("id", id);
        self.push(issue, via);
    }

    /// An include that would expand something already being expanded closes a
    /// cycle; it's reported where it closes (Q20).
    fn report_cycle(
        &mut self,
        file: &FileIndex,
        include: &Include,
        cycle_start: usize,
        via: &Arc<[IncludeSite]>,
    ) {
        let name = |key: &Key| match key.1 {
            None => format!("`{}`", key.0),
            Some(_) => format!("a section of `{}`", key.0),
        };
        let mut names: Vec<String> = self.stack[cycle_start..].iter().map(&name).collect();
        if let Some(first) = self.stack.get(cycle_start) {
            names.push(name(first));
        }
        let issue = Issue::new(
            diagnostics::INCLUDE_CYCLE,
            Location::new(file.file, include.primary.unwrap_or(include.span)),
        )
        .with_arg("path", include.written.clone())
        .with_arg("cycle", names.join(" → "));
        self.push(issue, via);
    }

    fn push(&mut self, issue: Issue, via: &Arc<[IncludeSite]>) {
        self.problems.push(PageProblem {
            issue,
            via: via.clone(),
        });
    }
}

/// The blocks of the section whose heading starts at `heading_start`: the
/// heading and the blocks after it in the same list of blocks, up to the next
/// heading of the same or a higher level. The heading may be nested in a block
/// quote, list item, container, or arm; its section ends with that list.
fn find_section(blocks: &[Block], heading_start: usize) -> Option<&[Block]> {
    for (i, block) in blocks.iter().enumerate() {
        match &block.kind {
            BlockKind::Heading(h) if block.span.start() == heading_start => {
                let end = blocks[i + 1..]
                    .iter()
                    .position(|b| matches!(&b.kind, BlockKind::Heading(n) if n.level <= h.level))
                    .map_or(blocks.len(), |p| i + 1 + p);
                return Some(&blocks[i..end]);
            }
            BlockKind::BlockQuote(q) => {
                if let Some(found) = find_section(&q.children, heading_start) {
                    return Some(found);
                }
            }
            BlockKind::List(l) => {
                for item in &l.items {
                    if let Some(found) = find_section(&item.children, heading_start) {
                        return Some(found);
                    }
                }
            }
            BlockKind::Container(c) => {
                if let Some(found) = find_section(&c.children, heading_start) {
                    return Some(found);
                }
            }
            BlockKind::Group(g) => {
                for arm in &g.arms {
                    if let Some(found) = find_section(&arm.children, heading_start) {
                        return Some(found);
                    }
                }
            }
            _ => {}
        }
    }
    None
}
