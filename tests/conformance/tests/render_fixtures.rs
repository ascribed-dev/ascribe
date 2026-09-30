//! Checks the site-render fixtures in `tests/render/` against the constructs
//! the site-render contract defines (`project-docs/contracts/site-render.md`
//! §6): every construct has a fixture, and every fixture is complete.
//!
//! The fixtures themselves run against the two implementations:
//! `tessera-emit`'s `render_site_html` and `@ascribed/astro`'s markdown
//! plugin.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Index {
    fixture: Vec<Fixture>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    name: String,
    covers: Vec<String>,
    description: String,
}

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The construct names in the first column of the contract's §6 table.
fn constructs() -> BTreeSet<String> {
    let contract =
        std::fs::read_to_string(repo().join("project-docs/contracts/site-render.md")).unwrap();
    let table = contract
        .split("## 6. Constructs and fixtures")
        .nth(1)
        .unwrap();
    table
        .lines()
        .filter_map(|l| l.strip_prefix("| `"))
        .map(|l| l.split('`').next().unwrap().to_owned())
        .collect()
}

#[test]
fn every_construct_has_a_fixture() {
    let dir = repo().join("tests/render");
    let index: Index =
        toml::from_str(&std::fs::read_to_string(dir.join("fixtures.toml")).unwrap()).unwrap();
    let constructs = constructs();
    assert!(constructs.len() >= 10, "found {constructs:?}");

    let mut covered = BTreeSet::new();
    let mut names = BTreeSet::new();
    for f in &index.fixture {
        assert!(names.insert(f.name.clone()), "{} is listed twice", f.name);
        assert!(!f.description.is_empty(), "{} needs a description", f.name);
        for file in ["input.md", "expected.html"] {
            assert!(
                dir.join(&f.name).join(file).is_file(),
                "{}/{file} is missing",
                f.name
            );
        }
        for c in &f.covers {
            assert!(
                constructs.contains(c),
                "{}: `{c}` isn't a construct in the contract",
                f.name
            );
            covered.insert(c.clone());
        }
    }
    let missing: Vec<&String> = constructs.difference(&covered).collect();
    assert!(
        missing.is_empty(),
        "constructs with no fixture: {missing:?}"
    );

    let dirs: BTreeSet<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        dirs, names,
        "every fixture directory is listed in fixtures.toml"
    );
}
