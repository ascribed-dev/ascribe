//! The project's glossary (SPEC §5.4): its terms, read from the headings
//! marked `@term` on the pages `[glossary] pages` lists, or in the fragments
//! those pages include.
//!
//! A term's id is its heading's source id, its text the heading's text, and
//! its settings its `@term`'s, else the glossary's. A heading that two
//! glossary pages show, through a fragment, is one term, listed with both
//! pages in the order `pages` gives them. Two different headings with the
//! same text, or a text that is another term's alias, or with the same id,
//! are a duplicate: the first is the term, and the second is left out and
//! reported.
//!
//! A term's definition is the paragraph that opens its section (SPEC §5.4):
//! the first block of the section's own content, after the directives that
//! bind its heading and those that only annotate the block below them
//! (`@available`, `@intended`, `@memo`). A paragraph a fragment brings in
//! counts, and a section that opens with groups, such as `@variant`, has
//! the paragraph that opens each arm, for the builds that keep it. A section
//! that opens with anything else has no definition.
//!
//! `@term` on a heading that no glossary page shows isn't a term, and is
//! listed apart ([`Glossary::off_page`]).

use std::collections::{HashMap, HashSet};

use ascribe_core::{FileId, Location, RelPath, Span};
use ascribe_model::GlossaryMatch;
use ascribe_syntax::{Block, BlockKind, Bound};

use crate::expand::{ExpandedBlock, ExpandedKind};
use crate::index::FileKind;
use crate::project::Project;

/// A glossary term: a heading marked `@term` on a glossary page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Term {
    /// The term's id: the heading's source id.
    pub id: String,
    /// The term as prose writes it: the heading's text.
    pub term: String,
    /// Other forms that count as the term.
    pub aliases: Vec<String>,
    /// Whether matching is case-sensitive: the `@term`'s setting, else the
    /// glossary's.
    pub case_sensitive: bool,
    /// Which occurrences are linked: the `@term`'s setting, else the
    /// glossary's.
    pub match_mode: GlossaryMatch,
    /// The file the heading is written in: a glossary page, or a fragment one
    /// includes.
    pub written_in: RelPath,
    /// That file's id.
    pub file: FileId,
    /// The heading block, in that file.
    pub heading: Span,
    /// The `@term` directive line, in that file.
    pub mark: Span,
    /// Where the paragraphs that may define it are written, in its first
    /// page's expansion: the first block of its section's own content, or,
    /// when the section opens with groups, the paragraph that opens each of
    /// their arms. Empty
    /// when the section opens with anything else.
    pub definition: Vec<Location>,
    /// The glossary pages that show the term, in the order `pages` lists
    /// them. A term is linked to the first a build publishes.
    pub pages: Vec<RelPath>,
}

impl Term {
    /// The term's text, then its aliases.
    pub fn texts(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.term.as_str()).chain(self.aliases.iter().map(String::as_str))
    }

    /// Whether the term is the heading written at `heading` in `file`.
    pub fn is_heading(&self, file: FileId, heading: Span) -> bool {
        self.file == file && self.heading == heading
    }
}

/// A heading marked `@term` whose text, one of whose aliases, or whose id
/// is already an earlier term's (`glossary-term-duplicate`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DuplicateTerm {
    /// The text the two share, as the second writes it, or the id they
    /// share.
    pub text: String,
    /// Whether the two share their id, rather than a text.
    pub id: bool,
    /// The file the second is written in.
    pub file: FileId,
    /// Where the second writes the text: its heading, or its `@term` line
    /// for an alias.
    pub span: Span,
    /// The earlier term's id.
    pub first: String,
    /// The first glossary page that shows the earlier term.
    pub first_page: RelPath,
}

/// `@term` on a heading no glossary page shows (`term-off-glossary-page`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OffPage {
    /// The file it's written in.
    pub file: FileId,
    /// That file's content path.
    pub path: RelPath,
    /// The `@term` directive line.
    pub mark: Span,
    /// Whether the file is a page, which `[glossary] pages` could list; a
    /// fragment is shown only by the pages that include it.
    pub page: bool,
}

/// The project's glossary terms, and the headings left out as duplicates.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Glossary {
    /// The terms, in the order of `pages`, and on each page in document
    /// order.
    pub terms: Vec<Term>,
    /// The headings left out because an earlier term has their text or
    /// their id.
    pub duplicates: Vec<DuplicateTerm>,
    /// Each `@term` on a heading no glossary page shows, in path order and
    /// source order.
    pub off_page: Vec<OffPage>,
}

impl Glossary {
    /// The term with this id.
    pub fn term(&self, id: &str) -> Option<&Term> {
        self.terms.iter().find(|t| t.id == id)
    }

    /// The term written at `heading` in `file`.
    pub fn term_at(&self, file: FileId, heading: Span) -> Option<&Term> {
        self.terms.iter().find(|t| t.is_heading(file, heading))
    }

    /// Whether `page` is one of the glossary's pages that shows a term.
    pub fn shows_terms(&self, page: &RelPath) -> bool {
        self.terms.iter().any(|t| t.pages.contains(page))
    }
}

impl Project {
    /// The project's glossary terms (SPEC §5.4), read from the pages
    /// `[glossary] pages` lists. Empty when the project has no glossary.
    pub fn glossary(&self) -> Glossary {
        self.read_glossary(true)
    }

    /// The glossary's terms alone, without looking through every file for
    /// the `@term`s no glossary page shows ([`Glossary::off_page`]): what a
    /// build links, which resolving each page asks for, and what an edit
    /// compares (#894).
    pub fn glossary_terms(&self) -> Vec<Term> {
        self.read_glossary(false).terms
    }

    /// The glossary, with the `@term`s off its pages when `with_off_page`
    /// is set.
    fn read_glossary(&self, with_off_page: bool) -> Glossary {
        let settings = &self.model().glossary;
        let mut terms: Vec<Term> = Vec::new();
        let mut duplicates = Vec::new();
        // Each text seen so far, lowercased, with the term and the text as
        // written.
        let mut seen: HashMap<String, Vec<(usize, String)>> = HashMap::new();
        // Every heading marked `@term` that a glossary page shows.
        let mut shown: HashSet<(FileId, Span)> = HashSet::new();
        for listed in &settings.pages {
            let Ok(page) = RelPath::parse(&listed.path) else {
                continue;
            };
            let Some(expanded) = self.expansion(&page) else {
                continue;
            };
            for (written_in, heading) in expanded.headings(self) {
                let Some(mark) = &heading.term else {
                    continue;
                };
                let Some(file) = self.file(&written_in).map(|f| f.file) else {
                    continue;
                };
                shown.insert((file, heading.span));
                if let Some(same) = terms.iter_mut().find(|t| t.is_heading(file, heading.span)) {
                    if !same.pages.contains(&page) {
                        same.pages.push(page.clone());
                    }
                    continue;
                }
                let term = Term {
                    id: heading.source_id.clone(),
                    term: heading.text.trim().to_owned(),
                    aliases: mark.aliases.clone(),
                    case_sensitive: mark.case_sensitive.unwrap_or(settings.case_sensitive),
                    match_mode: mark.match_mode.unwrap_or(settings.match_mode),
                    written_in,
                    file,
                    heading: heading.span,
                    mark: mark.span,
                    definition: opening(&expanded.blocks, file, heading.span).unwrap_or_default(),
                    pages: vec![page.clone()],
                };
                if let Some(duplicate) =
                    duplicate_of(&term, &terms, &seen).or_else(|| same_id(&term, &terms))
                {
                    duplicates.push(duplicate);
                    continue;
                }
                let at = terms.len();
                for text in term.texts() {
                    seen.entry(text.to_lowercase())
                        .or_default()
                        .push((at, text.to_owned()));
                }
                terms.push(term);
            }
        }
        let mut off_page = Vec::new();
        if with_off_page {
            for index in self.files() {
                for heading in &index.headings {
                    let Some(mark) = &heading.term else {
                        continue;
                    };
                    if !shown.contains(&(index.file, heading.span)) {
                        off_page.push(OffPage {
                            file: index.file,
                            path: index.path.clone(),
                            mark: mark.span,
                            page: index.kind == FileKind::Page,
                        });
                    }
                }
            }
        }
        Glossary {
            terms,
            duplicates,
            off_page,
        }
    }
}

/// Where the paragraphs that open the section of the heading written at
/// `heading` in `file` are written, among `blocks` and the blocks inside
/// them: empty when the section opens with anything else, and `None` when
/// the heading isn't there.
fn opening(blocks: &[ExpandedBlock], file: FileId, heading: Span) -> Option<Vec<Location>> {
    let Some(at) = blocks
        .iter()
        .position(|b| b.file == file && b.span == heading && is_heading(b))
    else {
        return blocks
            .iter()
            .find_map(|b| children(b).find_map(|list| opening(list, file, heading)));
    };
    let mut out = Vec::new();
    paragraphs(&blocks[at + 1..], &mut out);
    Some(out)
}

/// The paragraph that `blocks` opens with, or, when they open with groups
/// (one `@variant` group, or several in a row, each of which a build may
/// remove), the paragraph each arm opens with, in `out`.
fn paragraphs(blocks: &[ExpandedBlock], out: &mut Vec<Location>) {
    let mut groups = false;
    for block in blocks.iter().filter(|b| !annotates(b)) {
        match &block.kind {
            ExpandedKind::Leaf(Block {
                kind: BlockKind::Paragraph(_),
                ..
            }) if !groups => {
                out.push(Location::new(block.file, block.span));
                return;
            }
            ExpandedKind::Group { arms, .. } => {
                groups = true;
                for arm in arms {
                    paragraphs(&arm.children, out);
                }
            }
            _ => return,
        }
    }
}

/// The lists of blocks directly inside a block.
fn children(block: &ExpandedBlock) -> Box<dyn Iterator<Item = &[ExpandedBlock]> + '_> {
    match &block.kind {
        ExpandedKind::Leaf(_) => Box::new(std::iter::empty()),
        ExpandedKind::BlockQuote { children } | ExpandedKind::Container { children, .. } => {
            Box::new(std::iter::once(children.as_slice()))
        }
        ExpandedKind::List { items, .. } => Box::new(items.iter().map(|i| i.children.as_slice())),
        ExpandedKind::Group { arms, .. } => Box::new(arms.iter().map(|a| a.children.as_slice())),
    }
}

fn is_heading(block: &ExpandedBlock) -> bool {
    matches!(&block.kind, ExpandedKind::Leaf(b) if matches!(b.kind, BlockKind::Heading(_)))
}

/// Whether a block comes before a section's own content without being any
/// of it: a directive that binds the heading (`@id`, `@term`), or one that
/// annotates the block below it without changing what that block is
/// (`@available`, `@intended`, `@memo`, `@summary`).
fn annotates(block: &ExpandedBlock) -> bool {
    let line = match &block.kind {
        ExpandedKind::Leaf(Block {
            kind: BlockKind::Directive(line),
            ..
        }) => line,
        ExpandedKind::Container { opener, .. } => opener,
        _ => return false,
    };
    match line.binding {
        Some(Bound::Heading) => true,
        Some(Bound::FollowingBlock) => {
            matches!(
                line.name.as_str(),
                "available" | "intended" | "memo" | "summary"
            )
        }
        _ => false,
    }
}

/// The earlier term with `term`'s id, as a duplicate: a link to the id, and
/// the build's map from ids to terms, can name one term only.
fn same_id(term: &Term, terms: &[Term]) -> Option<DuplicateTerm> {
    let first = terms.iter().find(|t| t.id == term.id)?;
    Some(DuplicateTerm {
        text: term.id.clone(),
        id: true,
        file: term.file,
        span: term.heading,
        first: first.id.clone(),
        first_page: first.pages[0].clone(),
    })
}

/// The first of `term`'s texts that an earlier term already has: the same
/// text, or text that differs only in case where either term ignores case.
fn duplicate_of(
    term: &Term,
    terms: &[Term],
    seen: &HashMap<String, Vec<(usize, String)>>,
) -> Option<DuplicateTerm> {
    term.texts().enumerate().find_map(|(n, text)| {
        let (first, _) = seen
            .get(&text.to_lowercase())?
            .iter()
            .find(|(at, written)| {
                written == text || !terms[*at].case_sensitive || !term.case_sensitive
            })?;
        let first = &terms[*first];
        Some(DuplicateTerm {
            text: text.to_owned(),
            id: false,
            file: term.file,
            span: if n == 0 { term.heading } else { term.mark },
            first: first.id.clone(),
            first_page: first.pages[0].clone(),
        })
    })
}
