//! The resolved tree: what a build produces for one page.
//!
//! A [`ResolvedPage`] is a page after includes are expanded and the build's
//! passes have run (SPEC §9.2). It has the shape of the source, so emitters
//! and page-level checks work from what
//! survived rather than re-deriving it from the build's mode:
//!
//! - every block keeps the **file and span** it was written in, and the
//!   includes it came through ([`ResolvedBlock::via`]);
//! - a group a selection reduced to several arms is still a
//!   [`ResolvedKind::Group`]; a group reduced to one arm is that arm's blocks;
//! - every `@available` that survives stays a directive block, with its
//!   [`Annotation`]; every block also carries its **effective availability**
//!   (SPEC §4.4, after inheritance), and a table the availability of each row
//!   that has its own ([`ResolvedBlock::rows`]);
//! - phrases are substituted in the inline content and code, and listed as
//!   [`Substitution`]s over the source text;
//! - every heading has its page id ([`HeadingIds`]);
//! - every link and image is resolved ([`ResolvedLink`]), and glossary terms
//!   are linked ([`GlossaryUse`]).

use std::sync::Arc;

use ascribe_core::availability::AvailabilitySpec;
use ascribe_core::{FileId, Location, RelPath, Span};
use ascribe_model::Segment;
use ascribe_syntax::{Block, Bound, DirectiveLine, EndLine};

use crate::expand::{IncludeSite, PageProblem};
use crate::index::RefKind;
use crate::project::PageAsset;
use crate::snippet::Snippet;

/// A page resolved for one build.
#[derive(Clone, Debug)]
pub struct ResolvedPage {
    /// The page's content path.
    pub path: RelPath,
    /// The page's file id.
    pub file: FileId,
    /// The build's name.
    pub build: String,
    /// The page's route, from the [`ascribe_core::Router`] used.
    pub route: String,
    /// The frontmatter, with phrases substituted in the fields the content
    /// model says take them (`phrases = true`). `None` when the file has no
    /// frontmatter or it isn't valid YAML.
    pub frontmatter: Option<serde_yaml_ng::Value>,
    /// The page's title as plain text: the frontmatter `title`, substituted,
    /// and without code spans' backticks when the field sets `inline`.
    pub title: Option<String>,
    /// The fields whose values are read with inline markup (`inline =
    /// "code"`), in declaration order, each in pieces. A field the page
    /// leaves out has its default's. The plain text of each is in
    /// [`ResolvedPage::frontmatter`].
    pub formatted: Vec<FormattedField>,
    /// The page-level availability (the frontmatter `available`, after
    /// feature keys), which every node inherits unless it has its own.
    pub availability: Option<Arc<Availability>>,
    /// The blocks that survived the build, includes expanded.
    pub blocks: Vec<ResolvedBlock>,
    /// The assets the surviving content references, in document order, each
    /// with where it's written.
    pub assets: Vec<PageAsset>,
    /// Problems found while resolving, for the page-level checks to report:
    /// what expansion
    /// found (`include-cycle`, `include-id-missing`), then per build
    /// `variant-no-arm-survives`, `available-exceeds-scope`,
    /// `link-id-removed`, and `link-page-dropped`. Each is located where its
    /// cause is written, with the includes it came through.
    pub problems: Vec<PageProblem>,
}

/// A frontmatter field read with inline markup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormattedField {
    /// The field's name.
    pub name: String,
    /// Its value: text, with phrases substituted, and code spans.
    pub segments: Vec<Segment>,
}

impl ResolvedPage {
    /// The title in pieces, when `title` sets `inline`.
    pub fn formatted_title(&self) -> Option<&[Segment]> {
        self.formatted
            .iter()
            .find(|f| f.name == "title")
            .map(|f| f.segments.as_slice())
    }

    /// Calls `f` on every block, in document order, a block before the
    /// blocks inside it.
    pub fn visit<'a>(&'a self, f: &mut impl FnMut(&'a ResolvedBlock)) {
        self.blocks.iter().for_each(|b| b.visit(f));
    }

    /// Every heading that survived, in document order, with its ids.
    pub fn headings(&self) -> Vec<(&ResolvedBlock, &HeadingIds)> {
        let mut out = Vec::new();
        self.visit(&mut |block| {
            if let Some(ids) = &block.heading {
                out.push((block, ids));
            }
        });
        out
    }

    /// The page id of the heading written at `span` in `file`, if it
    /// survived the build. A fragment included twice has its headings twice;
    /// this is the first.
    pub fn page_id_of(&self, file: FileId, span: Span) -> Option<&str> {
        let mut found = None;
        self.visit(&mut |block| {
            if found.is_none()
                && block.file == file
                && block.span == span
                && let Some(ids) = &block.heading
            {
                found = Some(ids.page_id.as_str());
            }
        });
        found
    }
}

/// A block of a resolved page: a block of one source file.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedBlock {
    /// The file the block is written in.
    pub file: FileId,
    /// The block's span in that file.
    pub span: Span,
    /// The includes the block came through, outermost first: the first is in
    /// the page's own file. Empty for the page's own blocks.
    pub via: Arc<[IncludeSite]>,
    /// The block's effective availability (SPEC §4.4): its own spec, or the
    /// one it inherits from its section, its enclosing blocks, or the page.
    /// `None` when nothing restricts it.
    pub availability: Option<Arc<Availability>>,
    /// For a heading, its ids (SPEC §5.5).
    pub heading: Option<HeadingIds>,
    /// For a surviving `@available` directive, what it declares.
    pub annotation: Option<Annotation>,
    /// For a code block a `@snippet` became, the snippet (SPEC §4.8).
    pub snippet: Option<Arc<Snippet>>,
    /// For a table, its body rows that have an availability of their own
    /// (SPEC §4.4), in source order. A filter build has removed the rows that
    /// aren't available from the table itself.
    pub rows: Vec<ResolvedRow>,
    /// The phrases substituted in this block's own inline content, in source
    /// order, as replacements over the source text. Their spans are in
    /// [`ResolvedBlock::file`].
    pub substitutions: Vec<Substitution>,
    /// The links and images in this block's own inline content, in source
    /// order.
    pub links: Vec<ResolvedLink>,
    /// The glossary terms linked in this block's own prose.
    pub glossary: Vec<GlossaryUse>,
    /// What the block is.
    pub kind: ResolvedKind,
}

/// The kinds of resolved block. A block that holds other blocks holds their
/// resolution; every other block is kept as it was parsed, with phrases
/// substituted and links resolved in its inline content.
#[derive(Clone, Debug, PartialEq)]
pub enum ResolvedKind {
    /// A block with no nested blocks: a heading, paragraph, code block, HTML
    /// block, table, thematic break, or line-form directive. An `@include`
    /// that couldn't be expanded stays as its directive.
    Leaf(Block),
    /// A block quote.
    BlockQuote {
        /// The blocks in it.
        children: Vec<ResolvedBlock>,
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
        items: Vec<ResolvedItem>,
    },
    /// A container directive.
    Container {
        /// The opener, with its title.
        opener: DirectiveLine,
        /// The end line, when it was closed.
        end: Option<EndLine>,
        /// The blocks in it.
        children: Vec<ResolvedBlock>,
    },
    /// A group that survived the build's selection with all its arms, or
    /// with several of them (a group reduced to one arm is that arm's blocks).
    Group {
        /// The groupable directive's name.
        name: String,
        /// The surviving arms.
        arms: Vec<ResolvedArm>,
        /// The group's end line, when it was closed.
        end: Option<EndLine>,
    },
}

/// A list item of a resolved list.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedItem {
    /// The item, from its marker through its last line.
    pub span: Span,
    /// The marker.
    pub marker: Span,
    /// The blocks in it.
    pub children: Vec<ResolvedBlock>,
}

/// A table row with its own availability (SPEC §4.4).
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedRow {
    /// The row, in the table's file.
    pub span: Span,
    /// The row's effective availability: its own spec, in the table's.
    pub availability: Arc<Availability>,
}

/// An arm of a resolved group.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedArm {
    /// The arm's span in its file.
    pub span: Span,
    /// The opener, with its title.
    pub opener: DirectiveLine,
    /// The blocks in it.
    pub children: Vec<ResolvedBlock>,
}

impl ResolvedBlock {
    /// Calls `f` on this block and every block inside it, a block before the
    /// blocks inside it.
    pub fn visit<'a>(&'a self, f: &mut impl FnMut(&'a ResolvedBlock)) {
        f(self);
        self.for_each_child(&mut |c| c.visit(f));
    }

    /// Calls `f` on each block directly inside this one.
    pub fn for_each_child<'a>(&'a self, f: &mut impl FnMut(&'a ResolvedBlock)) {
        match &self.kind {
            ResolvedKind::Leaf(_) => {}
            ResolvedKind::BlockQuote { children } | ResolvedKind::Container { children, .. } => {
                children.iter().for_each(f);
            }
            ResolvedKind::List { items, .. } => {
                items.iter().flat_map(|i| &i.children).for_each(f);
            }
            ResolvedKind::Group { arms, .. } => {
                arms.iter().flat_map(|a| &a.children).for_each(f);
            }
        }
    }

    /// The nested block lists of this block, mutably.
    pub(crate) fn child_lists_mut(&mut self) -> Vec<&mut Vec<ResolvedBlock>> {
        match &mut self.kind {
            ResolvedKind::Leaf(_) => Vec::new(),
            ResolvedKind::BlockQuote { children } | ResolvedKind::Container { children, .. } => {
                vec![children]
            }
            ResolvedKind::List { items, .. } => items.iter_mut().map(|i| &mut i.children).collect(),
            ResolvedKind::Group { arms, .. } => arms.iter_mut().map(|a| &mut a.children).collect(),
        }
    }
}

/// A heading's ids (SPEC §5.5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeadingIds {
    /// 1 to 6.
    pub level: u8,
    /// The heading's text as plain text, phrases substituted: what the slug
    /// is computed from.
    pub text: String,
    /// The id links and includes use to name the heading in its own file.
    pub source_id: String,
    /// The id of the heading's anchor on this page in this build: its
    /// `@id`, or its slug numbered across the whole expanded page.
    pub page_id: String,
    /// Whether the id is the heading's `@id`. A heading without one can get a
    /// different page id than source id when includes and build modes change
    /// what precedes it.
    pub explicit: bool,
    /// Where the `@id` directive is, when the heading has one.
    pub explicit_at: Option<Location>,
}

/// A phrase replaced in the source text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Substitution {
    /// The candidate, braces included (or, in a link or image destination,
    /// the same), in the block's file.
    pub span: Span,
    /// The key.
    pub key: String,
    /// The value it was replaced by.
    pub value: String,
}

/// A link or image, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedLink {
    /// The link or image node in its file.
    pub span: Span,
    /// Link or image.
    pub kind: RefKind,
    /// The destination as written, with phrases substituted.
    pub destination: String,
    /// What it points at.
    pub target: LinkTarget,
}

/// What a resolved link points at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkTarget {
    /// A URL with a scheme, passed through unchanged.
    External,
    /// A page the build publishes.
    Page {
        /// The page's content path.
        page: RelPath,
        /// The page id of the heading the link names, when it names one.
        id: Option<String>,
        /// The link's URL: the route, and `#` and the page id.
        url: String,
        /// Whether the link had no text and took the target's title.
        text_filled: bool,
    },
    /// A local file the build copies. The reference is rewritten by the
    /// emitter.
    Asset {
        /// The asset's source path.
        path: RelPath,
        /// The part after `#`, kept after the rewritten reference.
        fragment: Option<String>,
    },
    /// The link can't be resolved: its target is missing, is a fragment,
    /// names an id the page doesn't have, or is removed or dropped by the
    /// build. The reason is a problem, in the file's problems or the page's.
    Unresolved,
}

/// A glossary term linked in prose (SPEC §5.4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlossaryUse {
    /// The term's id in the glossary.
    pub term: String,
    /// The text that matched, as written.
    pub text: String,
    /// The URL the text now links to.
    pub url: String,
}

/// What a surviving `@available` directive declares (SPEC §4.4).
#[derive(Clone, Debug, PartialEq)]
pub struct Annotation {
    /// The spec as shown: what was written, or, for a feature key, the spec
    /// the key stands for.
    pub text: String,
    /// The feature key, when the primary was one.
    pub feature: Option<String>,
    /// What the directive binds.
    pub binding: Option<Bound>,
}

/// Which scope an availability spec was written for (SPEC §4.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Scope {
    /// The page's frontmatter `available`.
    Page,
    /// An `@available` at the top of a section: the heading and everything
    /// in its section.
    Section,
    /// An `@available` that binds the block it touches.
    Block,
    /// The `available` attribute of a table row.
    Row,
}

impl Scope {
    /// The word messages use for it.
    pub fn as_str(self) -> &'static str {
        match self {
            Scope::Page => "page",
            Scope::Section => "section",
            Scope::Block => "block",
            Scope::Row => "row",
        }
    }
}

/// An availability spec in force, with the scope it was written for and the
/// one it sits in (SPEC §4.4).
#[derive(Clone, Debug, PartialEq)]
pub struct Availability {
    /// The spec, with a feature key replaced by the spec it stands for.
    pub spec: AvailabilitySpec,
    /// The spec as shown (see [`Annotation::text`]).
    pub text: String,
    /// The feature key, when the primary was one.
    pub feature: Option<String>,
    /// The scope it was written for.
    pub scope: Scope,
    /// Where it's written: the `@available` directive line, the frontmatter,
    /// or a table row's attribute block.
    pub written_at: Location,
    /// The scope it sits in, whose availability it inherits. A node is
    /// available only if every spec in this chain says so.
    pub enclosing: Option<Arc<Availability>>,
}

/// Why a build doesn't publish a page (SPEC §9.3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DropReason {
    /// The page's `variant` frontmatter names a dimension the build selects,
    /// and none of the selected values.
    Variant,
    /// The page's `available` frontmatter makes it unavailable for the
    /// build's target and version.
    Unavailable,
}

/// A page a build doesn't publish.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DroppedPage {
    /// The page's content path.
    pub path: RelPath,
    /// Why.
    pub reason: DropReason,
}

/// Everything a build produces.
#[derive(Clone, Debug)]
pub struct ResolvedBuild {
    /// The build's name.
    pub build: String,
    /// The pages the build publishes, in path order.
    pub pages: Vec<ResolvedPage>,
    /// The pages it drops.
    pub dropped: Vec<DroppedPage>,
}

impl ResolvedBuild {
    /// The assets the build copies: every asset a surviving page references,
    /// once each, in path order.
    pub fn assets(&self) -> Vec<RelPath> {
        let mut paths: Vec<RelPath> = self
            .pages
            .iter()
            .flat_map(|p| p.assets.iter().map(|a| a.path.clone()))
            .collect();
        paths.sort();
        paths.dedup();
        paths
    }

    /// The page with this path, if the build publishes it.
    pub fn page(&self, path: &RelPath) -> Option<&ResolvedPage> {
        self.pages.iter().find(|p| &p.path == path)
    }
}
