//! Every integration test of `ascribe-diff` in one program. One program is
//! one link and one process, where a file per program was seven of each; the
//! tests themselves are the files in `all/`, unchanged. Each is named with
//! `#[path]`, since a plain `mod` here would look beside this file, in
//! `tests/`, where Cargo builds every file as a program of its own. To run
//! one file's tests: `cargo test -p ascribe-diff --test all compare`.

#[path = "../../ascribe-resolve/tests/incremental_support/mod.rs"]
mod incremental_support;

#[path = "all/command.rs"]
mod command;
#[path = "all/compare.rs"]
mod compare;
#[path = "all/drift.rs"]
mod drift;
#[path = "all/git.rs"]
mod git;
#[path = "all/html.rs"]
mod html;
#[path = "all/prompt.rs"]
mod prompt;
#[path = "all/reach.rs"]
mod reach;
