//! Which reported issues make the formatter leave a construct alone.

use tessera_core::Span;
use tessera_syntax::ParsedDocument;

/// Diagnostics that are warnings about layout or prose, not about a
/// construct's meaning. They don't stop the formatter: `binding-blank-line`
/// is what its blank-line rule fixes, and the rest say nothing about the
/// bytes it edits. Every other issue the parser and the structure pass report
/// is an error. A test checks this list against the diagnostics registry.
pub const NON_BLOCKING: &[&str] = &[
    "binding-blank-line",
    "container-nesting-deep",
    "directive-indented-code",
    "directive-unknown",
    "list-ended-by-directive",
    "steps-numbering-continued",
    "title-dot-space",
    "title-not-accepted",
];

/// The locations of every issue in `doc` that blocks formatting.
pub(crate) fn blocking_spans(doc: &ParsedDocument) -> Vec<Span> {
    doc.issues
        .iter()
        .filter(|i| !NON_BLOCKING.contains(&i.slug.as_str()))
        .map(|i| i.location.span)
        .collect()
}
