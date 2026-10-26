//! `ascribe_syntax::parse` on arbitrary text, with the built-in directives:
//! it never panics, and every span in the tree lies inside the input, on
//! character boundaries. The span checker is the one the parser's own tests
//! use; its other checks, that each span covers the text it names, aren't
//! held here yet (#1121, #1122).

#![no_main]

use ascribe_syntax::{ParseOptions, parse};
use libfuzzer_sys::fuzz_target;

#[path = "../../crates/ascribe-syntax/tests/support/mod.rs"]
mod support;

fuzz_target!(|source: &str| {
    let doc = parse(source, &ParseOptions::default());
    let outside: Vec<String> = support::check_tree(source, &doc)
        .into_iter()
        .filter(|problem| problem.ends_with(support::NOT_IN_SOURCE))
        .collect();
    assert!(outside.is_empty(), "{outside:#?}");
});
