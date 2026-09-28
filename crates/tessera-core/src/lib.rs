//! Shared types for the Tessera toolchain.
//!
//! Every other Tessera crate depends on this one, and on nothing else for
//! these types. It holds types and traits only, with the small amount of
//! behavior they need to be exact (line and column conversion, applying
//! edits, normalizing paths). The parsers that live here are added by later
//! phases: the attribute parser (phase 05, `attributes`) and the
//! availability-spec parser (phase 08, `availability`).
//!
//! | Module | Contents |
//! |---|---|
//! | [`span`] | [`FileId`], [`Span`], and [`Location`]: where things are |
//! | [`line_index`] | [`LineIndex`]: byte offsets to lines and UTF-8, UTF-16, or scalar-value columns |
//! | [`text_edit`] | [`TextEdit`] and [`apply_edits`]: fixes, formatting, refactoring |
//! | [`issue`] | [`Issue`]: a problem, by registry slug, with its message arguments |
//! | [`diagnostics`] | One [`DiagnosticSlug`] constant per entry of `tests/conformance/diagnostics.toml` |
//! | [`schema`] | [`DirectiveSchema`], and the built-in schemas of SPEC §4 |
//! | [`attribute_block`] | [`AttributeBlock`]: parsed attributes, with spans (SPEC §3.3) |
//! | [`path`] | [`RelPath`], and how link and image destinations resolve |
//! | [`reserved`] | Attribute keys a content model can't declare (SPEC §7.2) |
//! | [`consumer`] | The [`Slugger`], [`Router`], and [`ConsumerProfile`] traits (SPEC §9.5) |
//!
//! These types are a contract between phases (see
//! `project-docs/phases/README.md`, "Contracts"). They change only through an
//! approved entry in `project-docs/questions.md`.

pub mod attribute_block;
pub mod consumer;
pub mod diagnostics;
pub mod issue;
pub mod line_index;
pub mod path;
pub mod reserved;
pub mod schema;
pub mod span;
pub mod text_edit;

pub use attribute_block::{Attribute, AttributeBlock, AttributeValue, Token};
pub use consumer::{AssetPlacement, AssetUse, ConsumerProfile, Router, SlugScope, Slugger};
pub use issue::{Arg, DiagnosticSlug, Fix, Issue, Related};
pub use line_index::{LineCol, LineIndex, WideEncoding, WideLineCol};
pub use path::{Destination, LocalDestination, PathError, RelPath, classify_destination};
pub use schema::{
    AttributeSchema, AttributeType, Attributes, Binding, Builtin, DefaultValue, DirectiveSchema,
    END_KEYWORD, Forms, Origin, Primary, SetMember, TitleRule, builtin_schemas,
};
pub use span::{FileId, Location, Span};
pub use text_edit::{EditError, TextEdit, apply_edits};
