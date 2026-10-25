//! Every integration test of `ascribe-check` in one program. One program is
//! one link and one process, where a file per program was nine of each; the
//! tests themselves are the files in `all/`, unchanged. Each is named with
//! `#[path]`, since a plain `mod` here would look beside this file, in
//! `tests/`, where Cargo builds every file as a program of its own. To run
//! one file's tests: `cargo test -p ascribe-check --test all parity`.

#[path = "all/across.rs"]
mod across;
#[path = "all/addresses.rs"]
mod addresses;
#[path = "all/addresses_settings.rs"]
mod addresses_settings;
#[path = "all/checks.rs"]
mod checks;
#[path = "all/commands.rs"]
mod commands;
#[path = "all/file_system.rs"]
mod file_system;
#[path = "all/identity.rs"]
mod identity;
#[path = "all/ids.rs"]
mod ids;
#[path = "all/inert_marks.rs"]
mod inert_marks;
#[path = "all/link_cache.rs"]
mod link_cache;
#[path = "all/link_retry.rs"]
mod link_retry;
// `links.rs` panics on a link it can't make, as a test helper may.
#[allow(clippy::expect_used, clippy::panic)]
#[path = "../../../tests/support/links.rs"]
mod links;

#[path = "all/page.rs"]
mod page;
#[path = "all/page_index.rs"]
mod page_index;
#[path = "all/parity.rs"]
mod parity;
#[path = "all/permalinks_list.rs"]
mod permalinks_list;
#[path = "all/prompt.rs"]
mod prompt;
#[path = "all/snippets.rs"]
mod snippets;
#[path = "all/type_files.rs"]
mod type_files;
#[path = "all/vale.rs"]
mod vale;
