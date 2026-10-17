//! Glossary terms (SPEC §5.4, §9.2 step 7): linking the occurrences of the
//! project's glossary terms in prose, as the glossary's settings say, and
//! each term's definition for the build.
//!
//! - A term is a heading marked `@term` on a glossary page
//!   ([`crate::Glossary`]). It's linked to its heading on the first of its
//!   glossary pages the build publishes; a term whose pages the build doesn't
//!   publish, or whose heading it removes, isn't linked.
//! - Its **definition** is the first block of its section's own content,
//!   when that's a paragraph, as plain text: phrases substituted for the
//!   build, emphasis and code reduced to their text, a link to its text, an
//!   image to its alt text, inline HTML dropped, and whitespace runs one
//!   space. A section that opens with anything else has an empty definition.
//! - Occurrences match **whole words**, and where terms overlap the
//!   **longest** match wins (`API key` over `API`).
//! - Matching applies to **prose** only: paragraphs, table cells, and a
//!   directive's text primary, never headings, titles, link text, alt text,
//!   code, or raw HTML.
//! - `match = "first"` links the first occurrence of each term on the page
//!   (the resolved page, after includes and build modes); `"every"` links all.
//!   A term with `match = "marked"` is never linked here: authors link the
//!   occurrences that mean it.
//! - A term isn't linked on a glossary page that shows it.
//! - A link an author writes to a term's heading is a use of the term, as a
//!   linked occurrence is, except on a glossary page that shows it. Under
//!   `match = "first"` it's the page's linked occurrence, so no other is.
//!
//! A linked occurrence becomes an ordinary link, and each use is listed in
//! [`ResolvedBlock::glossary`] with the term's definition.

use std::collections::{BTreeMap, BTreeSet};

use ascribe_core::RelPath;
use ascribe_model::{ContentModel, GlossaryMatch};
use ascribe_syntax::{BlockKind, Bound, Inline, InlineKind, Link, LinkForm};

use super::BuildResolver;
use super::inlines::{inline_text, prose_lists_mut};
use super::tree::{GlossaryUse, LinkTarget, ResolvedBlock, ResolvedKind, ResolvedPage};
use crate::glossary::Term;

/// A term as one build links it.
#[derive(Clone, Debug)]
pub(crate) struct BuildTerm {
    /// The term.
    pub(crate) term: Term,
    /// The URL of its heading on the first of its pages the build publishes.
    pub(crate) url: String,
    /// Its definition in this build.
    pub(crate) definition: String,
}

/// The terms a build links, with their URLs and definitions.
pub(crate) fn build_terms(resolver: &BuildResolver<'_>) -> Vec<BuildTerm> {
    let mut out = Vec::new();
    for term in resolver.project().glossary().terms {
        let Some(page) = term.pages.iter().find(|p| resolver.is_published(p)) else {
            continue;
        };
        let Some(page_id) = resolver.page_id_of(page, term.file, term.heading) else {
            continue;
        };
        let url = resolver.router().link(page, Some(&page_id));
        let definition = resolver
            .stage(page)
            .and_then(|staged| definition(&staged.blocks, &term))
            .unwrap_or_default();
        out.push(BuildTerm {
            term,
            url,
            definition,
        });
    }
    out
}

/// Links glossary terms in a page's prose, and marks the links an author
/// wrote to a term's heading as its uses.
pub(crate) fn link_terms(resolver: &BuildResolver<'_>, page: &mut ResolvedPage) {
    let all = resolver.terms();
    if all.is_empty() {
        return;
    }
    let here: Vec<&BuildTerm> = all
        .iter()
        .filter(|t| !t.term.pages.contains(&page.path))
        .collect();
    let mut by_id: BTreeMap<&str, &BuildTerm> = BTreeMap::new();
    for t in &here {
        by_id.entry(t.term.id.as_str()).or_insert(t);
    }
    let terms = Terms::new(
        here.iter()
            .map(|t| &t.term)
            .filter(|t| t.match_mode != GlossaryMatch::Marked),
    );
    let mut linker = Linker {
        resolver,
        terms,
        by_id,
        here: &here,
        linked: BTreeSet::new(),
    };
    linker.blocks(&mut page.blocks);
}

/// The pages `[glossary] pages` lists: a change to what one shows can change
/// every page that uses a term.
pub fn glossary_targets(model: &ContentModel) -> BTreeSet<RelPath> {
    model
        .glossary
        .pages
        .iter()
        .filter_map(|p| RelPath::parse(&p.path).ok())
        .collect()
}

/// A term's definition in a list of blocks: the paragraph that opens its
/// heading's section, as plain text. `None` when the heading isn't there.
pub(crate) fn definition(blocks: &[ResolvedBlock], term: &Term) -> Option<String> {
    let at = blocks
        .iter()
        .position(|b| b.heading.is_some() && term.is_heading(b.file, b.span));
    let Some(at) = at else {
        return blocks
            .iter()
            .flat_map(|b| child_lists(b))
            .find_map(|list| definition(list, term));
    };
    // The heading's own directives (`@id`, `@term`, a section's
    // `@available`) come before its content.
    let first = blocks[at + 1..].iter().find(|b| !binds_heading(b));
    Some(match first.map(|b| &b.kind) {
        Some(ResolvedKind::Leaf(block)) => match &block.kind {
            BlockKind::Paragraph(p) => collapse(&inline_text(&p.inlines)),
            _ => String::new(),
        },
        _ => String::new(),
    })
}

/// The lists of blocks directly inside a block.
fn child_lists(block: &ResolvedBlock) -> Vec<&Vec<ResolvedBlock>> {
    match &block.kind {
        ResolvedKind::Leaf(_) => Vec::new(),
        ResolvedKind::BlockQuote { children } | ResolvedKind::Container { children, .. } => {
            vec![children]
        }
        ResolvedKind::List { items, .. } => items.iter().map(|i| &i.children).collect(),
        ResolvedKind::Group { arms, .. } => arms.iter().map(|a| &a.children).collect(),
    }
}

/// Whether a block is a directive line that binds the heading above it.
fn binds_heading(block: &ResolvedBlock) -> bool {
    matches!(
        &block.kind,
        ResolvedKind::Leaf(b) if matches!(&b.kind, BlockKind::Directive(line) if line.binding == Some(Bound::Heading))
    )
}

/// Text with every run of whitespace one space, and none at either end.
fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The glossary's terms and aliases, ready to be found in text.
pub(crate) struct Terms {
    /// Longest first, so that where terms overlap the longest wins.
    candidates: Vec<Candidate>,
}

/// A term, or one of its aliases.
pub(crate) struct Candidate {
    text: String,
    /// The term's id.
    pub(crate) term: String,
    case_sensitive: bool,
    first_only: bool,
}

impl Terms {
    /// The terms' texts and aliases, matched as each term's settings say.
    pub(crate) fn new<'a>(terms: impl IntoIterator<Item = &'a Term>) -> Terms {
        let mut candidates: Vec<Candidate> = Vec::new();
        for term in terms {
            for text in term.texts() {
                candidates.push(Candidate {
                    text: text.to_owned(),
                    term: term.id.clone(),
                    case_sensitive: term.case_sensitive,
                    first_only: term.match_mode == GlossaryMatch::First,
                });
            }
        }
        candidates.sort_by_key(|c| std::cmp::Reverse(c.text.chars().count()));
        Terms { candidates }
    }

    /// The first occurrence at or after byte offset `at`, as its start, its
    /// end, and the candidate it matches.
    pub(crate) fn next_match(
        &self,
        text: &str,
        mut at: usize,
    ) -> Option<(usize, usize, &Candidate)> {
        while at < text.len() {
            if let Some((len, candidate)) = self.match_at(text, at) {
                return Some((at, at + len, candidate));
            }
            at += text[at..].chars().next().map_or(1, char::len_utf8);
        }
        None
    }

    /// The longest candidate that matches as a whole word at byte offset
    /// `at`, and its length in `text`.
    fn match_at(&self, text: &str, at: usize) -> Option<(usize, &Candidate)> {
        let before = text[..at].chars().next_back();
        if before.is_some_and(is_word) {
            return None;
        }
        self.candidates.iter().find_map(|c| {
            let len = prefix_len(&text[at..], &c.text, c.case_sensitive)?;
            let after = text[at + len..].chars().next();
            (!after.is_some_and(is_word)).then_some((len, c))
        })
    }
}

struct Linker<'a, 'p> {
    resolver: &'a BuildResolver<'p>,
    terms: Terms,
    /// The terms this page links, by id.
    by_id: BTreeMap<&'a str, &'a BuildTerm>,
    /// The terms this page may use: those it doesn't show.
    here: &'a [&'a BuildTerm],
    linked: BTreeSet<String>,
}

impl Linker<'_, '_> {
    fn blocks(&mut self, blocks: &mut [ResolvedBlock]) {
        for block in blocks {
            let mut used = self.written(block);
            for list in prose_lists_mut(block) {
                self.list(list, &mut used);
            }
            block.glossary = used;
            for children in block.child_lists_mut() {
                self.blocks(children);
            }
        }
    }

    /// The links in a block that an author wrote to a term's heading: each a
    /// use of the term (SPEC §5.4).
    fn written(&mut self, block: &ResolvedBlock) -> Vec<GlossaryUse> {
        let mut used = Vec::new();
        for link in &block.links {
            let LinkTarget::Page {
                page,
                id: Some(id),
                url,
                ..
            } = &link.target
            else {
                continue;
            };
            let Some(t) = self.here.iter().find(|t| {
                t.term.pages.contains(page)
                    && self
                        .resolver
                        .page_id_of(page, t.term.file, t.term.heading)
                        .as_ref()
                        == Some(id)
            }) else {
                continue;
            };
            let Some(text) = link_text(block, link.span) else {
                continue;
            };
            used.push(GlossaryUse {
                term: t.term.id.clone(),
                text,
                url: url.clone(),
                definition: t.definition.clone(),
            });
            self.linked.insert(t.term.id.clone());
        }
        used
    }

    /// Links occurrences in the text nodes of one list of inlines, and in
    /// emphasis inside it. Link text and images are left alone.
    fn list(&mut self, list: &mut Vec<Inline>, used: &mut Vec<GlossaryUse>) {
        let mut out: Vec<Inline> = Vec::with_capacity(list.len());
        for inline in list.drain(..) {
            match inline.kind {
                InlineKind::Text(text) => {
                    self.text(&text, inline.span, &mut out, used);
                }
                InlineKind::Emphasis(mut children) => {
                    self.list(&mut children, used);
                    out.push(Inline {
                        span: inline.span,
                        kind: InlineKind::Emphasis(children),
                    });
                }
                InlineKind::Strong(mut children) => {
                    self.list(&mut children, used);
                    out.push(Inline {
                        span: inline.span,
                        kind: InlineKind::Strong(children),
                    });
                }
                kind => out.push(Inline {
                    span: inline.span,
                    kind,
                }),
            }
        }
        *list = out;
    }

    /// Splits a text node around its linked occurrences. Every piece has the
    /// span of the node: a decoded text can't be mapped back more exactly.
    fn text(
        &mut self,
        text: &str,
        span: ascribe_core::Span,
        out: &mut Vec<Inline>,
        used: &mut Vec<GlossaryUse>,
    ) {
        let mut plain_from = 0;
        let mut at = 0;
        while let Some((start, end, candidate)) = self.terms.next_match(text, at) {
            at = end;
            let term = candidate.term.clone();
            if candidate.first_only && self.linked.contains(&term) {
                continue;
            }
            let Some(linked) = self.by_id.get(term.as_str()) else {
                continue;
            };
            let url = linked.url.clone();
            if plain_from < start {
                out.push(Inline {
                    span,
                    kind: InlineKind::Text(text[plain_from..start].to_owned()),
                });
            }
            let matched = text[start..end].to_owned();
            out.push(Inline {
                span,
                kind: InlineKind::Link(Link {
                    form: LinkForm::Inline,
                    destination: url.clone(),
                    title: None,
                    label: None,
                    destination_phrases: Vec::new(),
                    children: vec![Inline {
                        span,
                        kind: InlineKind::Text(matched.clone()),
                    }],
                }),
            });
            used.push(GlossaryUse {
                term: term.clone(),
                text: matched,
                url,
                definition: linked.definition.clone(),
            });
            self.linked.insert(term);
            plain_from = end;
        }
        if plain_from < text.len() {
            out.push(Inline {
                span,
                kind: InlineKind::Text(text[plain_from..].to_owned()),
            });
        }
    }
}

/// The text of the link written at `span` in a block's own inline content,
/// as the outputs show it.
fn link_text(block: &ResolvedBlock, span: ascribe_core::Span) -> Option<String> {
    let mut found = None;
    for list in super::inlines::own_lists(block) {
        crate::index::walk::walk_inlines(list, &mut |inline| {
            if found.is_none()
                && inline.span == span
                && let InlineKind::Link(link) = &inline.kind
            {
                found = Some(inline_text(&link.children));
            }
        });
    }
    found
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The byte length of the prefix of `text` that equals `term`, comparing
/// case-insensitively unless `case_sensitive`.
fn prefix_len(text: &str, term: &str, case_sensitive: bool) -> Option<usize> {
    let mut chars = text.char_indices();
    for expected in term.chars() {
        let (_, actual) = chars.next()?;
        let same = if case_sensitive {
            actual == expected
        } else {
            actual.to_lowercase().eq(expected.to_lowercase())
        };
        if !same {
            return None;
        }
    }
    Some(chars.next().map_or(text.len(), |(i, _)| i))
}
