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

#[test]
fn configurable_matches_the_registry() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/conformance/diagnostics.toml");
    let text = std::fs::read_to_string(&path).expect("read diagnostics.toml");
    let doc: toml::Table = text.parse().expect("diagnostics.toml is TOML");
    let registry: Vec<&str> = doc["diagnostic"]
        .as_array()
        .expect("[[diagnostic]] entries")
        .iter()
        .filter(|e| e.get("configurable").and_then(toml::Value::as_bool) == Some(true))
        .map(|e| e["slug"].as_str().expect("slug"))
        .collect();
    let constants: Vec<&str> = diagnostics::CONFIGURABLE
        .iter()
        .map(|s| s.as_str())
        .collect();
    assert_eq!(
        constants, registry,
        "ascribe_core::diagnostics::CONFIGURABLE must list every registry entry with \
         `configurable = true`, in registry order"
    );
}

#[test]
fn acknowledgeable_matches_the_registry() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/conformance/diagnostics.toml");
    let text = std::fs::read_to_string(&path).expect("read diagnostics.toml");
    let doc: toml::Table = text.parse().expect("diagnostics.toml is TOML");
    let registry: Vec<(&str, &str)> = doc["diagnostic"]
        .as_array()
        .expect("[[diagnostic]] entries")
        .iter()
        .filter(|e| e.get("next").and_then(toml::Value::as_str) == Some("review"))
        .map(|e| {
            let place = e.get("place").and_then(toml::Value::as_str).unwrap_or("");
            (e["slug"].as_str().expect("slug"), place)
        })
        .collect();
    let constants: Vec<(&str, &str)> = diagnostics::ACKNOWLEDGEABLE
        .iter()
        .map(|(s, p)| (s.as_str(), p.as_str()))
        .collect();
    assert_eq!(
        constants, registry,
        "ascribe_core::diagnostics::ACKNOWLEDGEABLE must list every registry entry with \
         `next = \"review\"` and its `place`, in registry order"
    );
}
