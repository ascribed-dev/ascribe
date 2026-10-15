//! Every integration test of `ascribe-syntax` in one program. One program is
//! one link and one process, where a file per program was six of each; the
//! tests themselves are the files in `all/`, unchanged. Each is named with
//! `#[path]`, since a plain `mod` here would look beside this file, in
//! `tests/`, where Cargo builds every file as a program of its own. To run
//! one file's tests: `cargo test -p ascribe-syntax --test all agreement`.

mod support;

#[path = "all/agreement.rs"]
mod agreement;
#[path = "all/definitions.rs"]
mod definitions;
#[path = "all/inline.rs"]
mod inline;
#[path = "all/parse.rs"]
mod parse;
#[path = "all/spans.rs"]
mod spans;
#[path = "all/structure.rs"]
mod structure;
