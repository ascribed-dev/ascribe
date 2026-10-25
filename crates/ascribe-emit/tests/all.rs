//! Every integration test of `ascribe-emit` in one program. One program is
//! one link and one process, where a file per program was twelve of each; the
//! tests themselves are the files in `all/`, unchanged. Each is named with
//! `#[path]`, since a plain `mod` here would look beside this file, in
//! `tests/`, where Cargo builds every file as a program of its own. To run
//! one file's tests: `cargo test -p ascribe-emit --test all zod`.

// `links.rs` panics on a link it can't make, as a test helper may.
#[allow(clippy::expect_used, clippy::panic)]
#[path = "../../../tests/support/links.rs"]
mod links;
mod support;

#[path = "all/agents.rs"]
mod agents;
#[path = "all/archive.rs"]
mod archive;
#[path = "all/assets.rs"]
mod assets;
#[path = "all/formatted.rs"]
mod formatted;
#[path = "all/help.rs"]
mod help;
#[path = "all/lines.rs"]
mod lines;
#[path = "all/openapi.rs"]
mod openapi;
#[path = "all/pages.rs"]
mod pages;
#[path = "all/permalinks.rs"]
mod permalinks;
#[path = "all/plain.rs"]
mod plain;
#[path = "all/quill.rs"]
mod quill;
#[path = "all/reading.rs"]
mod reading;
#[path = "all/redirects.rs"]
mod redirects;
#[path = "all/render_fixtures.rs"]
mod render_fixtures;
#[path = "all/site.rs"]
mod site;
#[path = "all/site_anchors.rs"]
mod site_anchors;
#[path = "all/site_assets.rs"]
mod site_assets;
#[path = "all/site_quill.rs"]
mod site_quill;
#[path = "all/store.rs"]
mod store;
#[path = "all/write.rs"]
mod write;
#[path = "all/zod.rs"]
mod zod;
