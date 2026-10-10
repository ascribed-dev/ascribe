//! Every integration test of `ascribe-fmt` in one program. One program is
//! one link and one process, where a file per program was five of each; the
//! tests themselves are the files in `all/`, unchanged. Each is named with
//! `#[path]`, since a plain `mod` here would look beside this file, in
//! `tests/`, where Cargo builds every file as a program of its own. To run
//! one file's tests: `cargo test -p ascribe-fmt --test all rules`.

mod support;

#[path = "all/files.rs"]
mod files;
#[path = "all/fuzz.rs"]
mod fuzz;
#[path = "all/inputs.rs"]
mod inputs;
#[path = "all/registry.rs"]
mod registry;
#[path = "all/rules.rs"]
mod rules;
