//! The project's glossary (SPEC §5.4): its terms, read from the headings
//! marked `@term` on the pages `[glossary] pages` lists, or in the fragments
//! those pages include.
//!
//! A term's id is its heading's source id, its text the heading's text, and
//! its settings its `@term`'s, else the glossary's. A heading that two
//! glossary pages show, through a fragment, is one term, listed with both
//! pages in the order `pages` gives them. Two different headings with the
//! same text, or a text that is another term's alias, are a duplicate: the
//! first is the term, and the second is left out and reported.

use std::collections::HashMap;

use ascribe_core::{FileId, RelPath, Span};
use ascribe_model::GlossaryMatch;

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

/// A heading marked `@term` whose text, or one of whose aliases, is already
/// an earlier term's (`glossary-term-duplicate`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DuplicateTerm {
    /// The text the two share, as the second writes it.
    pub text: String,
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

/// The project's glossary terms, and the headings left out as duplicates.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Glossary {
    /// The terms, in the order of `pages`, and on each page in document
    /// order.
    pub terms: Vec<Term>,
    /// The headings left out because an earlier term has their text.
    pub duplicates: Vec<DuplicateTerm>,
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
        let settings = &self.model().glossary;
        let mut terms: Vec<Term> = Vec::new();
        let mut duplicates = Vec::new();
        // Each text seen so far, lowercased, with the term and the text as
        // written.
        let mut seen: HashMap<String, Vec<(usize, String)>> = HashMap::new();
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
                    pages: vec![page.clone()],
                };
                if let Some(duplicate) = duplicate_of(&term, &terms, &seen) {
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
        Glossary { terms, duplicates }
    }
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
            file: term.file,
            span: if n == 0 { term.heading } else { term.mark },
            first: first.id.clone(),
            first_page: first.pages[0].clone(),
        })
    })
}
