//! Ascribe parsing: Ascribe lines, the structure pass, and the syntax tree.
//!
//! [`parse`] reads one source file into a [`ParsedDocument`], Ascribe's own
//! syntax tree (see [`tree`]). It builds on `comrak-tessera`, the CommonMark
//! parser with Ascribe's block-level changes, and converts comrak's tree to
//! Ascribe's, so no downstream crate sees comrak's types.
//!
//! What phase 05 produces:
//!
//! - every CommonMark block and inline Ascribe needs, with exact byte spans;
//! - **directive lines** and **end lines**, with the head parsed into name,
//!   attribute block (and each pair), colon, and primary, each with a span;
//! - **issues** for what the head gets wrong, by registry slug: malformed
//!   attribute blocks, a primary the directive doesn't take or lacks, and a
//!   directive-shaped line with an unknown name (which stays text).
//!
//! What phase 06's structure pass (`src/structure/`) adds, in the same call:
//!
//! - **containers** ([`Container`]) holding the blocks between an opener and
//!   its end line, and **groups** ([`Group`]) of [`Arm`]s for runs of openers
//!   of a groupable directive such as `@variant`;
//! - **titles**, attached to the directive below them
//!   ([`DirectiveLine::title`]);
//! - **bindings** ([`DirectiveLine::binding`], [`bound_heading`],
//!   [`bound_block`]);
//! - **issues** for every structural row of SPEC §8.2: forms, unclosed
//!   containers and stray end lines, nesting, binding, titles, `@variant`
//!   groups, `@steps` and `@details`, and the list warnings.
//!
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
//! Phase 23 adds **link reference definitions** ([`ParsedDocument::definitions`]),
//! a side list rather than blocks: each has exact spans for its label,
//! destination, and title, and the phrase candidates in its destination
//! ([`LinkDefinition::destination_phrases`], SPEC §5.1). The parser consumes
//! them, so the fork reports them (`comrak_tessera::parse_document_with_definitions`).
//!
//! Phase 06 owns `src/structure/` and phase 07 owns `src/inline/`. The inline
//! pass runs after the structure pass.
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
mod structure;
mod unknown;

pub mod tree;

pub use options::ParseOptions;
pub use structure::{bound_block, bound_heading};
pub use tree::*;

/// Parses a Ascribe source file.
///
/// Never panics, whatever the input. Anything malformed is reported in
/// [`ParsedDocument::issues`] and the tree keeps as much of the source as it
/// can.
pub fn parse(source: &str, options: &ParseOptions) -> ParsedDocument {
    convert::convert(source, options)
}
