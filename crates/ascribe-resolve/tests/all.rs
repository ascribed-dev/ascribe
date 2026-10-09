//! Every integration test of `ascribe-resolve` in one program. One program is
//! one link and one process, where a file per program was eleven of each; the
//! tests themselves are the files in `all/`, unchanged. Each is named with
//! `#[path]`, since a plain `mod` here would look beside this file, in
//! `tests/`, where Cargo builds every file as a program of its own. To run
//! one file's tests: `cargo test -p ascribe-resolve --test all includes`.

mod build_support;
mod incremental_support;
// `links.rs` panics on a link it can't make, as a test helper may; each file
// that included it allowed that at its root, and this root does the same.
#[allow(clippy::expect_used, clippy::panic)]
#[path = "../../../tests/support/links.rs"]
mod links;
mod support;

#[path = "all/build_content.rs"]
mod build_content;
#[path = "all/build_modes.rs"]
mod build_modes;
#[path = "all/build_quill.rs"]
mod build_quill;
#[path = "all/file_reads.rs"]
mod file_reads;
#[path = "all/headings.rs"]
mod headings;
#[path = "all/includes.rs"]
mod includes;
#[path = "all/incremental.rs"]
mod incremental;
#[path = "all/incremental_differential.rs"]
mod incremental_differential;
#[path = "all/references.rs"]
mod references;
#[path = "all/snippets.rs"]
mod snippets;
#[path = "all/uses.rs"]
mod uses;
