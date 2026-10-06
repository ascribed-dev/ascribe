//! Glossary terms (SPEC §5.4, §9.2 step 7): linking the occurrences of the
//! content model's glossary terms in prose, as the glossary's settings say.
//!
//! - Occurrences match **whole words**, and where terms overlap the
//!   **longest** match wins (`API key` over `API`).
//! - Matching applies to **prose** only: paragraphs, table cells, and a
//!   directive's text primary, never headings, titles, link text, alt text,
//!   code, or raw HTML.
//! - `match = "first"` links the first occurrence of each term on the page
//!   (the resolved page, after includes and build modes); `"every"` links all.
//! - A term with no `link` isn't linked. Neither is one whose page the build
//!   doesn't publish, whose id the build removes, or that is the page itself.
//!
//! A linked occurrence becomes an ordinary link, and is listed in
//! [`ResolvedBlock::glossary`].

// What the content model leaves open (emphasis, a term's
// own page, a term whose page the build doesn't publish, and where the text is
// matched) is settled as the list above says.

use std::collections::{BTreeMap, BTreeSet};

use tessera_core::RelPath;
use tessera_model::{GlossaryMatch, GlossaryTerm};
use tessera_syntax::{Inline, InlineKind, Link, LinkForm};

use super::BuildResolver;
use super::inlines::prose_lists_mut;
use super::tree::{GlossaryUse, ResolvedBlock, ResolvedPage};

/// Links glossary terms in a page's prose.
pub(crate) fn link_terms(resolver: &BuildResolver<'_>, page: &mut ResolvedPage) {
    let glossary = &resolver.project().model().glossary;
    if glossary.terms.is_empty() {
        return;
    }
    let mut candidates: Vec<Candidate> = Vec::new();
    let mut urls: BTreeMap<&str, String> = BTreeMap::new();
    for term in &glossary.terms {
        if let Some(url) = term_url(resolver, page, term) {
            urls.insert(term.id.as_str(), url);
        } else {
            continue;
        }
        for text in std::iter::once(&term.term).chain(&term.aliases) {
            candidates.push(Candidate {
                text: text.clone(),
                term: term.id.clone(),
                case_sensitive: term.case_sensitive,
            });
        }
    }
    // Longest first, so that where terms overlap the longest wins.
    candidates.sort_by_key(|c| std::cmp::Reverse(c.text.chars().count()));
    let mut linker = Linker {
        candidates,
        urls,
        first_only: glossary.match_mode == GlossaryMatch::First,
        linked: BTreeSet::new(),
    };
    linker.blocks(&mut page.blocks);
}

/// The URL a term links to on this page, if it links anywhere.
fn term_url(
    resolver: &BuildResolver<'_>,
    page: &ResolvedPage,
    term: &GlossaryTerm,
) -> Option<String> {
    let (path, id) = term.link.as_ref()?;
    let target = RelPath::parse(path.trim_start_matches('/')).ok()?;
    if target == page.path || !resolver.is_published(&target) {
        return None;
    }
    let router = resolver.router();
    match id {
        None => Some(router.link(&target, None)),
        Some(id) => {
            let project = resolver.project();
            let (written_in, heading) = project.page_heading(&target, id)?;
            let file = project.file(&written_in)?;
            let page_id = resolver.page_id_of(&target, file.file, heading.span)?;
            Some(router.link(&target, Some(&page_id)))
        }
    }
}

struct Candidate {
    text: String,
    term: String,
    case_sensitive: bool,
}

struct Linker<'a> {
    candidates: Vec<Candidate>,
    urls: BTreeMap<&'a str, String>,
    first_only: bool,
    linked: BTreeSet<String>,
}

impl Linker<'_> {
    fn blocks(&mut self, blocks: &mut [ResolvedBlock]) {
        for block in blocks {
            let mut used = Vec::new();
            for list in prose_lists_mut(block) {
                self.list(list, &mut used);
            }
            block.glossary = used;
            for children in block.child_lists_mut() {
                self.blocks(children);
            }
        }
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
        span: tessera_core::Span,
        out: &mut Vec<Inline>,
        used: &mut Vec<GlossaryUse>,
    ) {
        let mut plain_from = 0;
        let mut at = 0;
        while at < text.len() {
            let Some((len, candidate)) = self.match_at(text, at) else {
                at += text[at..].chars().next().map_or(1, char::len_utf8);
                continue;
            };
            let end = at + len;
            let term = candidate.term.clone();
            let already = self.linked.contains(&term);
            if self.first_only && already {
                at = end;
                continue;
            }
            let Some(url) = self.urls.get(term.as_str()).cloned() else {
                at = end;
                continue;
            };
            if plain_from < at {
                out.push(Inline {
                    span,
                    kind: InlineKind::Text(text[plain_from..at].to_owned()),
                });
            }
            let matched = text[at..end].to_owned();
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
            });
            self.linked.insert(term);
            plain_from = end;
            at = end;
        }
        if plain_from < text.len() {
            out.push(Inline {
                span,
                kind: InlineKind::Text(text[plain_from..].to_owned()),
            });
        }
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
