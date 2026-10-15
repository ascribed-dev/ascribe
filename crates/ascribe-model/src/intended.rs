//! `[[intended]]`: the acknowledgements of problems about a content model
//! entry or an image, which have no page to hold them (SPEC §4.9).

use ascribe_core::intended::{NameProblem, PLACEHOLDER_REASON, check_named, is_placeholder};
use ascribe_core::{EntryKind, Place, diagnostics};
use toml::de::DeValue;

use crate::loader::Loader;
use crate::model::{Feature, Glossary, Intended, Phrase};
use crate::names::{list, suggest};
use crate::toml_util::{V, entries, sp};

impl Loader<'_> {
    /// `[[intended]]`, against the entries the content model declares.
    pub(crate) fn intended(
        &mut self,
        v: Option<&V<'_>>,
        phrases: &[Phrase],
        features: &[Feature],
        glossary: &Glossary,
    ) -> Vec<Intended> {
        let mut out = Vec::new();
        let Some(v) = v else { return out };
        let DeValue::Array(items) = v.get_ref() else {
            self.wrong_type("intended", v, "an array of tables, written [[intended]]");
            return out;
        };
        let keys = ["check", "reason", "phrase", "feature", "term", "image"];
        for item in items.iter() {
            let Some(t) = self.as_table("intended[]", item) else {
                continue;
            };
            let span = sp(item);
            self.check_keys("intended", t, &keys);
            let check = self.require("intended", t, span, "check").and_then(|c| {
                let name = self.string("intended.check", c, false)?;
                self.check_name(&name, sp(c))
            });
            let reason = self.require("intended", t, span, "reason").and_then(|r| {
                let reason = self.string("intended.reason", r, true)?;
                if is_placeholder(&reason) {
                    self.push(
                        self.issue(diagnostics::MODEL_INTENDED_ENTRY, sp(r))
                            .with_variant("placeholder")
                            .with_arg("reason", PLACEHOLDER_REASON),
                    );
                }
                Some(reason)
            });
            let named: Vec<(EntryKind, &V<'_>)> = entries(t)
                .filter_map(|(key, _, value)| Some((EntryKind::from_key(key)?, value)))
                .collect();
            let entry = match named.as_slice() {
                [] => {
                    self.push(self.issue(diagnostics::MODEL_INTENDED_ENTRY, span));
                    None
                }
                [(kind, value)] => {
                    let path = format!("intended.{}", kind.key());
                    self.string(&path, value, true).and_then(|name| {
                        let declared = match kind {
                            EntryKind::Phrase => phrases.iter().any(|p| p.key == name),
                            EntryKind::Feature => features.iter().any(|f| f.key == name),
                            EntryKind::Term => glossary.terms.iter().any(|t| t.id == name),
                            // An image is a file, and a check that reports
                            // one names it by its path; one that names no
                            // image is reported as unused.
                            EntryKind::Image => true,
                        };
                        if declared {
                            Some((*kind, name))
                        } else {
                            self.push(
                                self.issue(diagnostics::MODEL_INTENDED_ENTRY, sp(value))
                                    .with_variant("unknown")
                                    .with_arg("kind", kind_noun(*kind))
                                    .with_arg("name", name),
                            );
                            None
                        }
                    })
                }
                several => {
                    let keys: Vec<&str> = several.iter().map(|(k, _)| k.key()).collect();
                    self.push(
                        self.issue(diagnostics::MODEL_INTENDED_ENTRY, span)
                            .with_variant("several")
                            .with_arg("keys", list(&keys)),
                    );
                    None
                }
            };
            if let (Some((check, check_span)), Some(reason), Some((kind, name))) =
                (check, reason, entry)
            {
                out.push(Intended {
                    check,
                    kind,
                    name,
                    reason,
                    span,
                    check_span,
                });
            }
        }
        out
    }

    /// The check an acknowledgement here names: a review check whose problems
    /// are about an entry.
    fn check_name(
        &mut self,
        name: &str,
        span: ascribe_core::Span,
    ) -> Option<(ascribe_core::DiagnosticSlug, ascribe_core::Span)> {
        let problem = match check_named(name, Place::Entry, self.review) {
            Ok(slug) => return Some((slug, span)),
            Err(problem) => problem,
        };
        let issue = self
            .issue(diagnostics::MODEL_INTENDED_CHECK, span)
            .with_arg("check", name);
        let issue = match problem {
            NameProblem::Unknown => {
                match suggest(name, diagnostics::ALL.iter().map(|s| s.as_str())) {
                    Some(s) => issue.with_variant("suggestion").with_arg("suggestion", s),
                    None => issue,
                }
            }
            NameProblem::NotReview => issue.with_variant("not-review"),
            NameProblem::Place(place) => issue
                .with_variant("place")
                .with_arg("place", place.noun())
                .with_arg("written", place.written()),
        };
        self.push(issue);
        None
    }
}

/// What a kind of entry is called in a message.
fn kind_noun(kind: EntryKind) -> &'static str {
    match kind {
        EntryKind::Phrase => "phrase",
        EntryKind::Feature => "feature",
        EntryKind::Term => "glossary term",
        EntryKind::Image => "image",
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use ascribe_core::{DiagnosticSlug, FileId};

    use crate::loader::load_reviewing;

    // No check can be acknowledged yet, so these make two of them review
    // checks: one about an entry, one about a page.
    const REVIEW: &[(DiagnosticSlug, Place)] = &[
        (diagnostics::BINDING_BLANK_LINE, Place::Entry),
        (diagnostics::CONTAINER_NESTING_DEEP, Place::Page),
    ];

    const DECLARED: &str = "spec = \"0.1\"\n\
        [phrases]\nold-name = \"Quill\"\n\
        [features.sync]\nname = \"Sync\"\navailable = \"cloud\"\n\
        [dimensions.deployment]\nvalues = [\"cloud\"]\n\
        [glossary.terms.api-key]\nterm = \"API key\"\ndefinition = \"A key.\"\n";

    fn load(intended: &str) -> Result<Vec<Intended>, Vec<(DiagnosticSlug, String)>> {
        let text = format!("{DECLARED}{intended}");
        load_reviewing(&text, FileId::new(0), REVIEW)
            .map(|m| m.intended)
            .map_err(|issues| {
                issues
                    .into_iter()
                    .map(|i| {
                        let found = text.get(i.location.span.range()).unwrap_or("");
                        (i.slug, found.to_owned())
                    })
                    .collect()
            })
    }

    #[test]
    fn an_acknowledgement_names_a_check_an_entry_and_a_reason() {
        let found = load(
            "[[intended]]\ncheck = \"binding-blank-line\"\nphrase = \"old-name\"\n\
             reason = \"Kept for the 2.x pages.\"\n\
             [[intended]]\ncheck = \"binding-blank-line\"\nimage = \"images/old.png\"\n\
             reason = \"Linked from the changelog.\"\n",
        )
        .unwrap();
        let got: Vec<_> = found
            .iter()
            .map(|i| (i.check, i.kind, i.name.as_str(), i.reason.as_str()))
            .collect();
        assert_eq!(
            got,
            [
                (
                    diagnostics::BINDING_BLANK_LINE,
                    EntryKind::Phrase,
                    "old-name",
                    "Kept for the 2.x pages."
                ),
                (
                    diagnostics::BINDING_BLANK_LINE,
                    EntryKind::Image,
                    "images/old.png",
                    "Linked from the changelog."
                ),
            ]
        );
    }

    #[test]
    fn the_check_must_be_a_review_check_about_an_entry() {
        let entry = "phrase = \"old-name\"\nreason = \"r\"\n";
        let found = load(&format!(
            "[[intended]]\ncheck = \"binding-blank-lines\"\n{entry}\
             [[intended]]\ncheck = \"title-not-accepted\"\n{entry}\
             [[intended]]\ncheck = \"container-nesting-deep\"\n{entry}"
        ))
        .unwrap_err();
        assert_eq!(
            found,
            [
                (
                    diagnostics::MODEL_INTENDED_CHECK,
                    "\"binding-blank-lines\"".to_owned()
                ),
                (
                    diagnostics::MODEL_INTENDED_CHECK,
                    "\"title-not-accepted\"".to_owned()
                ),
                (
                    diagnostics::MODEL_INTENDED_CHECK,
                    "\"container-nesting-deep\"".to_owned()
                ),
            ]
        );
    }

    #[test]
    fn it_names_one_declared_entry_and_gives_a_reason() {
        let check = "check = \"binding-blank-line\"\n";
        let found = load(&format!(
            "[[intended]]\n{check}reason = \"r\"\n\
             [[intended]]\n{check}phrase = \"old-name\"\nterm = \"api-key\"\nreason = \"r\"\n\
             [[intended]]\n{check}feature = \"syncs\"\nreason = \"r\"\n\
             [[intended]]\n{check}term = \"api-key\"\nreason = \" \"\n\
             [[intended]]\n{check}feature = \"sync\"\n"
        ))
        .unwrap_err();
        let slugs: Vec<DiagnosticSlug> = found.iter().map(|(s, _)| *s).collect();
        assert_eq!(
            slugs,
            [
                diagnostics::MODEL_INTENDED_ENTRY,
                diagnostics::MODEL_INTENDED_ENTRY,
                diagnostics::MODEL_INTENDED_ENTRY,
                diagnostics::MODEL_EMPTY_TEXT,
                diagnostics::MODEL_MISSING_KEY,
            ]
        );
        assert_eq!(found[2].1, "\"syncs\"");
    }
}
