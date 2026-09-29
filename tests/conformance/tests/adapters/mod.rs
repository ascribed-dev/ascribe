//! Adapters that connect the conformance harness to the Tessera crates.
//!
//! Each implementation phase adds a module here (or
//! extends an existing one), registers it below, and removes the skip entries
//! for the tags it now handles from `SKIPS.toml`.

mod check;
mod format;
pub mod include;
mod inline;
mod model;
mod page_check;
pub mod resolve;
mod structure;
mod syntax;

#[allow(unused_imports)]
pub use check::{file_level_diagnostics, to_conformance};
use tessera_conformance::Registry;

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
