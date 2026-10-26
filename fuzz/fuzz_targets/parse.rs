//! `ascribe_syntax::parse` on arbitrary text, with the built-in directives:
//! it never panics, and every span in the tree covers exactly the source
//! text it names. The span checker is the one the parser's own tests use.

#![no_main]

use ascribe_syntax::{ParseOptions, parse};
use libfuzzer_sys::fuzz_target;

#[path = "../../crates/ascribe-syntax/tests/support/mod.rs"]
mod support;

fuzz_target!(|source: &str| {
    let doc = parse(source, &ParseOptions::default());
    let problems = support::check_tree(source, &doc);
    assert!(problems.is_empty(), "{problems:#?}");
});
