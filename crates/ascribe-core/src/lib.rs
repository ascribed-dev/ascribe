//! Shared types for the Ascribe toolchain.
//!
//! Every other Ascribe crate depends on this one, and on nothing else for
//! these types. It holds types and traits only, with the small amount of
//! behavior they need to be exact (line and column conversion, applying
//! edits, normalizing paths), and three small parsers every crate shares: the
//! attribute parser ([`attributes`]), the info-string parser
//! ([`info_string`]), and the availability-spec parser (`availability`).
//!
//! | Module | Contents |
//! |---|---|
//! | [`span`] | [`FileId`], [`Span`], and [`Location`]: where things are |
//! | [`line_index`] | [`LineIndex`]: byte offsets to lines and UTF-8, UTF-16, or scalar-value columns |
//! | [`text_edit`] | [`TextEdit`] and [`apply_edits`]: fixes, formatting, refactoring |
//! | [`issue`] | [`Issue`]: a problem, by registry slug, with its message arguments |
//! | [`error`] | [`Coded`]: errors a caller can tell apart by a stable code |
//! | [`diagnostics`] | One [`DiagnosticSlug`] constant per entry of `tests/conformance/diagnostics.toml` |
//! | [`git`] | The options and the cleared environment every `git` Ascribe starts runs with |
//! | [`intended`] | [`Place`]: where a check's problem is acknowledged, and how a check names a content model entry |
//! | [`schema`] | [`DirectiveSchema`], and the built-in schemas of SPEC §4 |
//! | [`attributes`] | [`parse_attribute_block`]: the attribute-block parser |
//! | [`attribute_block`] | [`AttributeBlock`]: parsed attributes, with spans (SPEC §3.3) |
//! | [`info_string`] | [`InfoString`](info_string::InfoString): a code block's language and its `key=value` pairs (SPEC §3.3.1) |
//! | [`path`] | [`RelPath`], and how link and image destinations resolve |
//! | [`boundary`] | [`SourceBoundary`]: which files a symbolic link may lead to, for a crate that walks the disk itself; [`Visited`]: a walk goes into each folder once |
//! | [`names`] | The names Ascribe puts on a page: elements, attributes, classes, and ids |
//! | [`reserved`] | Attribute keys a content model can't declare (SPEC §7.2) |
//! | [`consumer`] | The [`Slugger`], [`Router`], and [`ConsumerProfile`] traits (SPEC §9.5) |
//!
//! Every other crate builds on these types, so a change here is a change to
//! all of them.

pub mod attribute_block;
pub mod attributes;
pub mod availability;
pub mod boundary;
pub mod consumer;
pub mod date;
pub mod diagnostics;
pub mod error;
pub mod git;
pub mod info_string;
pub mod intended;
pub mod issue;
pub mod line_index;
pub mod names;
pub mod path;
pub mod reserved;
pub mod schema;
pub mod span;
pub mod text_edit;

/// The docs site's address, with no trailing slash. It's `[consumer] site` in
/// `docs/ascribe.toml`, and a test in `ascribe-cli` checks that they agree.
/// Every address of the docs that Ascribe builds is a permalink,
/// [`docs_permalink!`], so nothing adds a page's path to it.
#[macro_export]
macro_rules! docs_site {
    () => {
        "https://ascribed-dev.com"
    };
}

/// Where the docs site publishes its permalinks' addresses: [`docs_site!`]
/// and `[consumer] permalink-path` in `docs/ascribe.toml`, which a test in
/// `ascribe-cli` checks.
#[macro_export]
macro_rules! docs_permalinks {
    () => {
        concat!($crate::docs_site!(), "/go/")
    };
}

/// The address of a permalink of the docs, which forwards to wherever its
/// page or section is: `docs_permalink!("cli-check")` is
/// `https://ascribed-dev.com/go/cli-check/`. It ends in `/`, as the docs
/// site's `trailing-slash` says. A test in `ascribe-cli` checks every name
/// Ascribe links to against the permalinks the docs publish. For a name
/// known only at run time, see [`docs_permalink`].
#[macro_export]
macro_rules! docs_permalink {
    ($name:literal) => {
        concat!($crate::docs_permalinks!(), $name, "/")
    };
}

/// [`docs_permalink!`], for a name known only at run time, such as a
/// diagnostic's code.
pub fn docs_permalink(name: &str) -> String {
    format!(concat!(docs_permalinks!(), "{}/"), name)
}

/// Whether `text` can be an id (SPEC §4.1) or a permalink's name (§2.1):
/// letters, digits, hyphens, underscores, and periods, and at least one.
pub fn is_id(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

pub use attribute_block::{Attribute, AttributeBlock, AttributeValue, Token};
pub use attributes::{ParsedAttributes, parse_attribute_block};
pub use boundary::{SourceBoundary, Visited};
pub use consumer::{AssetPlacement, AssetUse, ConsumerProfile, Router, SlugScope, Slugger};
pub use date::Date;
pub use error::Coded;
pub use intended::{EntryKind, Place};
pub use issue::{Applicability, Arg, DiagnosticSlug, Fix, Issue, Related};
pub use line_index::{LineCol, LineIndex, WideEncoding, WideLineCol};
pub use path::{
    Destination, LocalDestination, PathError, RelPath, classify_destination, image_media_type,
    percent_decode,
};
pub use schema::{
    AttributeSchema, AttributeType, Attributes, Binding, Builtin, DefaultValue, DirectiveSchema,
    END_KEYWORD, Forms, Origin, Primary, SetMember, TitleRule, builtin_schemas,
};
pub use span::{FileId, Location, Span};
pub use text_edit::{EditError, TextEdit, apply_edits};
