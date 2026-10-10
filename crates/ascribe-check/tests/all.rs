//! Every integration test of `ascribe-check` in one program. One program is
//! one link and one process, where a file per program was nine of each; the
//! tests themselves are the files in `all/`, unchanged. Each is named with
//! `#[path]`, since a plain `mod` here would look beside this file, in
//! `tests/`, where Cargo builds every file as a program of its own. To run
//! one file's tests: `cargo test -p ascribe-check --test all parity`.

#[path = "all/checks.rs"]
mod checks;
#[path = "all/commands.rs"]
mod commands;
#[path = "all/file_system.rs"]
mod file_system;
#[path = "all/ids.rs"]
mod ids;
#[path = "all/page.rs"]
mod page;
#[path = "all/page_index.rs"]
mod page_index;
#[path = "all/parity.rs"]
mod parity;
#[path = "all/prompt.rs"]
mod prompt;
#[path = "all/snippets.rs"]
mod snippets;
#[path = "all/vale.rs"]
mod vale;
