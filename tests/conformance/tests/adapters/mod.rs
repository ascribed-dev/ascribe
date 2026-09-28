//! Adapters that connect the conformance harness to the Tessera crates.
//!
//! No adapters exist yet. Each implementation phase adds a module here (or
//! extends an existing one), registers it below, and removes the skip entries
//! for the tags it now handles from `SKIPS.toml`.

use tessera_conformance::Registry;

/// Registers every adapter. Earlier registrations are asked first.
pub fn register(registry: &mut Registry) {
    let _ = registry;
}
