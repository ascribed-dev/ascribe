//! Tessera parsing: Tessera lines, the structure pass, and the syntax tree.
//!
//! [`parse`] reads one source file into a [`ParsedDocument`], Tessera's own
//! syntax tree (see [`tree`]). It builds on `comrak-tessera`, the CommonMark
//! parser with Tessera's block-level changes, and converts comrak's tree to
//! Tessera's, so no downstream crate sees comrak's types.
//!
//! What phase 05 produces:
//!
//! - every CommonMark block and inline Tessera needs, with exact byte spans;
//! - **directive lines** and **end lines**, with the head parsed into name,
//!   attribute block (and each pair), colon, and primary, each with a span;
//! - **issues** for what the head gets wrong, by registry slug: malformed
//!   attribute blocks, a primary the directive doesn't take or lacks, and a
//!   directive-shaped line with an unknown name (which stays text).
//!
//! Phase 07 adds the inline extensions (in `src/inline/`):
//!
//! - **phrase candidates** (`{key}`, SPEC §5.1) as [`InlineKind::Phrase`] in
//!   text, and in link and image destinations ([`Link::destination_phrases`]),
//!   and in fences that opt in with `phrases=true` ([`CodeBlock::phrases`]);
//!   an escaped `\{key}` stays text and is listed in
//!   [`ParsedDocument::escaped_phrases`];
//! - **image attribute blocks** (SPEC §5.3) as [`Image::attributes`], for
//!   every form of image, parsed by `tessera_core::parse_attribute_block`,
//!   with their issues in [`ParsedDocument::issues`].
//!
//! What a later phase adds to the tree (its node kinds are already defined):
//! containers, groups, titles, and bindings (phase 06). Phase 06 owns
//! `src/structure/` and phase 07 owns `src/inline/`.
//!
//! ```
//! use tessera_syntax::{parse, ParseOptions, BlockKind, PrimaryValue};
//!
//! let doc = parse("@note {type=caution}: Back up first.\n", &ParseOptions::default());
//! let BlockKind::Directive(line) = &doc.blocks[0].kind else { unreachable!() };
//! assert_eq!(line.name, "note");
//! assert!(matches!(line.primary, Some(PrimaryValue::Text(_))));
//! assert!(doc.issues.is_empty());
//! ```

mod convert;
mod head;
mod inline;
mod options;
mod unknown;

pub mod tree;

pub use options::ParseOptions;
pub use tree::*;

/// Parses a Tessera source file.
///
/// Never panics, whatever the input. Anything malformed is reported in
/// [`ParsedDocument::issues`] and the tree keeps as much of the source as it
/// can.
pub fn parse(source: &str, options: &ParseOptions) -> ParsedDocument {
    convert::convert(source, options)
}
