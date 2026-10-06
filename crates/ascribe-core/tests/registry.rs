//! Keeps `ascribe_core::diagnostics` equal to the registry,
//! `tests/conformance/diagnostics.toml`: the same slugs, in the same order.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use ascribe_core::{DiagnosticSlug, diagnostics};

fn registry_slugs() -> Vec<String> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/conformance/diagnostics.toml");
    let text = std::fs::read_to_string(&path).expect("read diagnostics.toml");
    let doc: toml::Table = text.parse().expect("diagnostics.toml is TOML");
    doc["diagnostic"]
        .as_array()
        .expect("[[diagnostic]] entries")
        .iter()
        .map(|e| e["slug"].as_str().expect("slug").to_owned())
        .collect()
}

#[test]
fn constants_match_the_registry() {
    let registry = registry_slugs();
    let constants: Vec<&str> = diagnostics::ALL.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        constants, registry,
        "ascribe_core::diagnostics must list every registry slug, in registry order"
    );
}

#[test]
fn constant_names_follow_their_slugs() {
    // The generated source names each constant after its slug.
    let source = include_str!("../src/diagnostics.rs");
    for slug in diagnostics::ALL {
        let name = slug.as_str().to_uppercase().replace('-', "_");
        assert!(
            source.contains(&format!("pub const {name}: DiagnosticSlug")),
            "no constant named {name} for `{slug}`"
        );
        assert_eq!(DiagnosticSlug::from_name(slug.as_str()), Some(*slug));
    }
}
