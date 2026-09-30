//! Checks the site-render fixtures in `tests/render/`: every fixture is listed
//! in `fixtures.toml` with a description, and has its input and expected HTML.
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

#[test]
fn every_fixture_is_listed_and_complete() {
    let dir = repo().join("tests/render");
    let index: Index =
        toml::from_str(&std::fs::read_to_string(dir.join("fixtures.toml")).unwrap()).unwrap();

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
        assert!(!f.covers.is_empty(), "{} covers no construct", f.name);
    }

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
