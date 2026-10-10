//! Acknowledgements (SPEC §4.9): where a check's problem is acknowledged, and
//! how a check says which content model entry a problem is about.
//!
//! Only checks whose next step is `review` can be acknowledged
//! ([`ACKNOWLEDGEABLE`](crate::diagnostics::ACKNOWLEDGEABLE)), and each
//! reports its problems at one kind of place, so each problem has one place
//! to be acknowledged: a page in its frontmatter, a block with `@intended`
//! above it, or a content model entry in `ascribe.toml`.

use crate::DiagnosticSlug;

/// Where a check reports its problems, and so where an author acknowledges
/// one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Place {
    /// A page or fragment as a whole: acknowledged in its `intended`
    /// frontmatter.
    Page,
    /// One block: acknowledged with `@intended` directly above it.
    Block,
    /// A content model entry or an image, which has no page: acknowledged
    /// in `ascribe.toml`'s `[[intended]]`.
    Entry,
}

impl Place {
    /// `"page"`, `"block"`, or `"entry"`, as the registry writes it.
    pub const fn as_str(self) -> &'static str {
        match self {
            Place::Page => "page",
            Place::Block => "block",
            Place::Entry => "entry",
        }
    }

    /// The place the registry names `name`.
    pub fn from_name(name: &str) -> Option<Place> {
        [Place::Page, Place::Block, Place::Entry]
            .into_iter()
            .find(|p| p.as_str() == name)
    }

    /// The place, for messages: "a page", "a block", or "a content model
    /// entry".
    pub const fn noun(self) -> &'static str {
        match self {
            Place::Page => "a page",
            Place::Block => "a block",
            Place::Entry => "a content model entry",
        }
    }

    /// Where an acknowledgement for this place is written, for messages:
    /// "in the page's `intended` frontmatter", say.
    pub const fn written(self) -> &'static str {
        match self {
            Place::Page => "in the page's `intended` frontmatter",
            Place::Block => "with `@intended` directly above the block",
            Place::Entry => "in `[[intended]]` in ascribe.toml",
        }
    }
}

/// The place a check's problems are reported at, when it can be acknowledged
/// at all: its entry in `review`, the list given (the registry's
/// [`ACKNOWLEDGEABLE`](crate::diagnostics::ACKNOWLEDGEABLE), or a test's).
pub fn place_of(review: &[(DiagnosticSlug, Place)], slug: DiagnosticSlug) -> Option<Place> {
    review.iter().find(|(s, _)| *s == slug).map(|(_, p)| *p)
}

/// Why an acknowledgement can't name a check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameProblem {
    /// No diagnostic has the name.
    Unknown,
    /// The check's next step isn't `review`, so its problems are never
    /// intended.
    NotReview,
    /// The check reports its problems at another kind of place, where they
    /// are acknowledged instead.
    Place(Place),
}

/// The check an acknowledgement written at `written` names, or why it can't
/// name it: `review` is the list of checks that can be acknowledged (the
/// registry's [`ACKNOWLEDGEABLE`](crate::diagnostics::ACKNOWLEDGEABLE), or a
/// test's).
///
/// # Errors
///
/// [`NameProblem`]: the name isn't a check, the check can't be acknowledged,
/// or it's acknowledged somewhere else.
pub fn check_named(
    name: &str,
    written: Place,
    review: &[(DiagnosticSlug, Place)],
) -> Result<DiagnosticSlug, NameProblem> {
    let slug = DiagnosticSlug::from_name(name).ok_or(NameProblem::Unknown)?;
    match place_of(review, slug) {
        None => Err(NameProblem::NotReview),
        Some(place) if place != written => Err(NameProblem::Place(place)),
        Some(_) => Ok(slug),
    }
}

/// The kinds of thing `[[intended]]` can name, each by its key there.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EntryKind {
    /// A phrase, by its key (`phrase = "old-name"`).
    Phrase,
    /// A feature, by its key (`feature = "sync"`).
    Feature,
    /// A glossary term, by its id (`term = "api-key"`).
    Term,
    /// An image file, by its path under the content root
    /// (`image = "images/diagram.png"`).
    Image,
}

impl EntryKind {
    /// Every kind, in the order the docs list them.
    pub const ALL: [EntryKind; 4] = [
        EntryKind::Phrase,
        EntryKind::Feature,
        EntryKind::Term,
        EntryKind::Image,
    ];

    /// Its key in `[[intended]]`.
    pub const fn key(self) -> &'static str {
        match self {
            EntryKind::Phrase => "phrase",
            EntryKind::Feature => "feature",
            EntryKind::Term => "term",
            EntryKind::Image => "image",
        }
    }

    /// The kind whose key is `key`.
    pub fn from_key(key: &str) -> Option<EntryKind> {
        EntryKind::ALL.into_iter().find(|k| k.key() == key)
    }
}

/// The issue argument a check whose place is [`Place::Entry`] names its
/// entry with, as [`entry_arg`] writes it. Acknowledgements in
/// `[[intended]]` are matched against it.
pub const ENTRY_ARG: &str = "entry";

/// The value of [`ENTRY_ARG`] for an entry: its kind's key, a space, and its
/// name (`phrase old-name`).
pub fn entry_arg(kind: EntryKind, name: &str) -> String {
    format!("{} {name}", kind.key())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics;

    #[test]
    fn places_and_kinds_round_trip() {
        for place in [Place::Page, Place::Block, Place::Entry] {
            assert_eq!(Place::from_name(place.as_str()), Some(place));
        }
        for kind in EntryKind::ALL {
            assert_eq!(EntryKind::from_key(kind.key()), Some(kind));
        }
        assert_eq!(entry_arg(EntryKind::Phrase, "old"), "phrase old");
    }

    #[test]
    fn a_name_is_a_review_check_acknowledged_where_it_reports() {
        let review = [(diagnostics::BINDING_BLANK_LINE, Place::Block)];
        let named = |name: &str, place| check_named(name, place, &review);
        assert_eq!(
            named("binding-blank-line", Place::Block),
            Ok(diagnostics::BINDING_BLANK_LINE)
        );
        assert_eq!(
            named("binding-blank-line", Place::Page),
            Err(NameProblem::Place(Place::Block))
        );
        assert_eq!(
            named("title-not-accepted", Place::Block),
            Err(NameProblem::NotReview)
        );
        assert_eq!(
            named("no-such-check", Place::Block),
            Err(NameProblem::Unknown)
        );
    }

    #[test]
    fn a_place_is_found_by_slug() {
        let review = [(diagnostics::BINDING_BLANK_LINE, Place::Block)];
        assert_eq!(
            place_of(&review, diagnostics::BINDING_BLANK_LINE),
            Some(Place::Block)
        );
        assert_eq!(place_of(&review, diagnostics::TITLE_NOT_ACCEPTED), None);
    }
}
