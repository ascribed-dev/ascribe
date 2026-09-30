//! Phrase candidates in link reference definitions (SPEC §5.1).
//!
//! The destination of `[ref]: {api}streaming "title"` is a link destination,
//! so phrases apply in it, as in an inline link's. Backslash escapes apply
//! there too: `\{key}` is text, and is listed in
//! [`ParsedDocument::escaped_phrases`](crate::ParsedDocument::escaped_phrases)
//! with the escapes elsewhere. The definitions themselves come from the fork
//! (`comrak_tessera::parse_document_with_definitions`); this reads their
//! destinations' source, as [`super::phrase`] does for an inline link's.

use super::Pass;
use super::phrase::{Found, scan};
use crate::tree::*;

impl Pass<'_> {
    /// Fills in [`LinkDefinition::destination_phrases`].
    pub(super) fn definitions(&mut self, definitions: &mut [LinkDefinition]) {
        for definition in definitions {
            let span = definition.destination;
            let Some(text) = self.source.get(span.start()..span.end()) else {
                continue;
            };
            for found in scan(text, span.start(), true) {
                match found {
                    Found::Escaped(phrase) => self.escaped.push(phrase),
                    Found::Candidate(phrase) => definition.destination_phrases.push(phrase),
                }
            }
        }
    }
}
