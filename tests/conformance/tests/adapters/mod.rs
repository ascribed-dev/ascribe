//! Adapters that connect the conformance harness to the Ascribe crates.
//!
//! A new adapter is a module here (or an extension of an existing one),
//! registered below; the skip entries for the tags it handles come out of
//! `SKIPS.toml`.

mod check;
mod format;
pub mod include;
mod inline;
mod model;
mod page_check;
pub mod resolve;
mod structure;
mod syntax;

use ascribe_conformance::Registry;
#[allow(unused_imports)]
pub use check::{file_level_diagnostics, to_conformance};

/// Registers every adapter. Earlier registrations are asked first.
pub fn register(registry: &mut Registry) {
    registry.register(syntax::SyntaxAdapter);
    registry.register(format::FormatAdapter);
    registry.register(check::CheckAdapter);
    registry.register(model::ModelAdapter);
    registry.register(include::IncludeAdapter);
    registry.register(resolve::ResolveAdapter);
    registry.register(page_check::PageCheckAdapter);
}
