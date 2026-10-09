//! Each example in the diagnostics registry shows what it says: its wrong
//! page has the diagnostic, under the explain model with the example's own
//! model laid over it, and nothing else; its right page has no diagnostic at
//! all.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use ascribe_check::Registry;
use ascribe_query::explain::{Base, example_slugs};

#[test]
fn every_example_shows_its_diagnostic() {
    let mut failures = Vec::new();
    let mut count = 0;
    for entry in Registry::global().entries() {
        let Some(example) = &entry.example else {
            continue;
        };
        count += 1;
        let wrong = example_slugs(example, &example.wrong, Base::Explain)
            .unwrap_or_else(|e| panic!("{}: {e}", entry.slug));
        if !wrong.contains(&entry.slug) {
            failures.push(format!(
                "{}: the wrong page doesn't have it; it has {wrong:?}",
                entry.slug
            ));
        } else if wrong.iter().any(|s| *s != entry.slug) {
            failures.push(format!(
                "{}: the wrong page has others too: {wrong:?}",
                entry.slug
            ));
        }
        let right = example_slugs(example, &example.right, Base::Explain)
            .unwrap_or_else(|e| panic!("{}: {e}", entry.slug));
        if !right.is_empty() {
            failures.push(format!("{}: the right page has {right:?}", entry.slug));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    // The diagnostics an agent meets most have one.
    assert!(count >= 30, "only {count} diagnostics have an example");
}

#[test]
fn examples_have_both_pages() {
    for entry in Registry::global().entries() {
        if let Some(example) = &entry.example {
            assert!(
                !example.wrong.is_empty() && !example.right.is_empty(),
                "{}: an example needs `wrong` and `right`",
                entry.slug
            );
            assert_ne!(example.wrong, example.right, "{}", entry.slug);
        }
    }
}
