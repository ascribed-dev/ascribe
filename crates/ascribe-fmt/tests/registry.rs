//! The formatter's list of diagnostics that don't stop it agrees with the
//! diagnostics registry.

#![allow(clippy::expect_used, clippy::panic)]

mod support;

#[test]
fn the_non_blocking_diagnostics_are_warnings() {
    let path = support::repo_root().join("tests/conformance/diagnostics.toml");
    let text = std::fs::read_to_string(path).expect("the registry is readable");
    let registry: toml::Table = text.parse().expect("the registry is TOML");
    let entries = registry["diagnostic"]
        .as_array()
        .expect("diagnostic entries");
    let severity = |slug: &str| {
        entries
            .iter()
            .find(|e| e["slug"].as_str() == Some(slug))
            .map(|e| e["severity"].as_str().expect("a severity").to_owned())
    };
    for slug in ascribe_fmt::NON_BLOCKING {
        assert_eq!(severity(slug).as_deref(), Some("warning"), "{slug}");
    }
}
