//! The `github` slugger: a port of `github-slugger` (see the module docs of
//! [`super`] for the version).
//!
//! Upstream's `slug(value)` lowercases the text (JavaScript's
//! `toLowerCase`), deletes every character its big Unicode regular expression
//! matches, and turns each space into a hyphen. The regular expression
//! removes control characters, ASCII and Unicode punctuation and symbols,
//! separators other than the ASCII space, and unassigned code points, and
//! keeps letters, marks, digits, `-`, and `_`. Emoji are removed, but some
//! of their invisible parts survive: U+FE0F (the emoji variation selector)
//! stays, while the zero-width joiner goes. The exact set is in `table.rs`. Odd results are intentional: the hyphen is kept, so `a - b`
//! is `a---b`, and `a & b` is `a--b`.
//!
//! Numbering is upstream's: a slug that's already been given out gets the
//! suffix `-1`, and if that has been given out too, `-2`, and so on, with the
//! count kept per original slug. Because upstream records every result,
//! `a`, `a`, `a-1` gives `a`, `a-1`, `a-1-1`.

use std::collections::HashMap;

use ascribe_core::{SlugScope, Slugger};

use super::table::REMOVED;

/// The slug of one piece of text, without numbering: what upstream's
/// standalone `slug(value)` returns. Repeated calls give the same result.
pub(crate) fn github_slug(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    // JavaScript's `toLowerCase` and Rust's `str::to_lowercase` both apply
    // the full Unicode mappings, including the final-sigma rule; the
    // fixtures check that they agree.
    for c in text.to_lowercase().chars() {
        if c == ' ' {
            out.push('-');
        } else if !is_removed(c) {
            out.push(c);
        }
    }
    out
}

/// Whether upstream's regular expression deletes this character.
fn is_removed(c: char) -> bool {
    let c = u32::from(c);
    // Every character below U+0080 is either kept or in the first few ranges;
    // the table is small enough that binary search is all this needs.
    REMOVED
        .binary_search_by(|&(first, last)| {
            if c < first {
                std::cmp::Ordering::Greater
            } else if c > last {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

/// The `github` slugger, which Astro's heading ids use.
#[derive(Clone, Copy, Debug, Default)]
pub struct GithubSlugger;

impl GithubSlugger {
    /// The name the content model uses for this slugger.
    pub const NAME: &'static str = "github";

    /// Starts an empty scope, without going through the trait object.
    pub fn scope(&self) -> GithubScope {
        GithubScope::default()
    }
}

impl Slugger for GithubSlugger {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn new_scope(&self) -> Box<dyn SlugScope + Send + '_> {
        Box::new(self.scope())
    }
}

/// One scope of `github` slugs: upstream's `GithubSlugger` instance, whose
/// `occurrences` map it reproduces.
#[derive(Clone, Debug, Default)]
pub struct GithubScope {
    /// For each slug given out (originals and numbered results alike), the
    /// highest suffix tried for it: `0` until it has been repeated.
    occurrences: HashMap<String, u32>,
}

impl SlugScope for GithubScope {
    fn slug(&mut self, text: &str) -> String {
        let original = github_slug(text);
        let mut result = original.clone();
        while self.occurrences.contains_key(&result) {
            // The counter is upstream's JavaScript number; a scope would need
            // billions of repeats of one slug to reach the limit.
            let count = self.occurrences.entry(original.clone()).or_insert(0);
            *count = count.saturating_add(1);
            result = format!("{original}-{count}");
        }
        self.occurrences.insert(result.clone(), 0);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::collections::HashMap;

    fn fixture(name: &str) -> Value {
        let path = format!("{}/src/slug/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        let text = std::fs::read_to_string(&path).unwrap();
        serde_json::from_str(&text).unwrap()
    }

    fn scope_slugs(inputs: &[&str]) -> Vec<String> {
        let mut scope = GithubSlugger.new_scope();
        inputs.iter().map(|t| scope.slug(t)).collect()
    }

    /// Every fixture was produced by the real package; see `generate.mjs`.
    #[test]
    fn fixtures_record_the_pinned_version() {
        for name in ["slugs.json", "scalars.json"] {
            assert_eq!(
                fixture(name)["version"],
                super::super::GITHUB_SLUGGER_VERSION
            );
        }
    }

    #[test]
    fn single_texts_match_upstream() {
        let fx = fixture("slugs.json");
        let cases = fx["cases"].as_array().unwrap();
        assert!(cases.len() > 500);
        for case in cases {
            let input = case["input"].as_str().unwrap();
            assert_eq!(
                github_slug(input),
                case["slug"].as_str().unwrap(),
                "input {input:?}"
            );
        }
    }

    /// Upstream's own test fixtures (`test/fixtures.json` in
    /// `Flet/github-slugger` at 2.0.0), run in order through one scope as
    /// upstream's test does, so later entries depend on earlier ones.
    #[test]
    fn upstream_test_fixtures_pass() {
        let fx = fixture("upstream.json");
        let cases = fx.as_array().unwrap();
        assert_eq!(cases.len(), 78);
        let mut scope = GithubSlugger.new_scope();
        for case in cases {
            let input = case["input"].as_str().unwrap();
            assert_eq!(
                scope.slug(input),
                case["expected"].as_str().unwrap(),
                "{}",
                case["name"]
            );
        }
    }

    #[test]
    fn duplicate_numbering_matches_upstream() {
        let fx = fixture("slugs.json");
        for seq in fx["dupes"].as_array().unwrap() {
            let inputs: Vec<&str> = seq["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect();
            let expected: Vec<&str> = seq["slugs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect();
            assert_eq!(scope_slugs(&inputs), expected, "inputs {inputs:?}");
        }
    }

    /// Every Unicode scalar value, one at a time: removed, kept as is, or
    /// mapped (lowercased, or the space). Context-dependent lowercasing such
    /// as the final sigma is covered by the case fixtures.
    #[test]
    fn every_scalar_matches_upstream() {
        let fx = fixture("scalars.json");
        let mapped: HashMap<u32, &str> = fx["mapped"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| (e[0].as_u64().unwrap() as u32, e[1].as_str().unwrap()))
            .collect();
        for cp in 0..=0x10FFFFu32 {
            let Some(c) = char::from_u32(cp) else {
                continue;
            };
            let got = github_slug(&c.to_string());
            match mapped.get(&cp) {
                Some(want) => assert_eq!(got, *want, "U+{cp:04X}"),
                None if is_removed(c) => assert_eq!(got, "", "U+{cp:04X}"),
                None => assert_eq!(got, c.to_string(), "U+{cp:04X}"),
            }
        }
    }

    #[test]
    fn documented_examples() {
        assert_eq!(github_slug("Getting Started!"), "getting-started");
        assert_eq!(github_slug("a - b"), "a---b");
        assert_eq!(github_slug("a & b"), "a--b");
        assert_eq!(github_slug("🎉 Party"), "-party");
        assert_eq!(github_slug("你好，世界"), "你好世界");
        assert_eq!(github_slug("!!!"), "");
        assert_eq!(github_slug("a\u{FE0F}b"), "a\u{FE0F}b");
        assert_eq!(github_slug("a\u{200D}b"), "ab");
        assert_eq!(
            scope_slugs(&["Intro", "intro", "INTRO"]),
            ["intro", "intro-1", "intro-2"]
        );
        assert_eq!(scope_slugs(&["a", "a", "a-1"]), ["a", "a-1", "a-1-1"]);
    }

    #[test]
    fn a_new_scope_starts_empty() {
        let slugger = GithubSlugger;
        let mut first = slugger.new_scope();
        assert_eq!(first.slug("intro"), "intro");
        assert_eq!(first.slug("intro"), "intro-1");
        let mut second = slugger.new_scope();
        assert_eq!(second.slug("intro"), "intro");
        assert_eq!(first.slug("intro"), "intro-2");
    }
}
