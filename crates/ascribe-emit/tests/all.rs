//! Every integration test of `ascribe-emit` in one program. One program is
//! one link and one process, where a file per program was twelve of each; the
//! tests themselves are the files in `all/`, unchanged. Each is named with
//! `#[path]`, since a plain `mod` here would look beside this file, in
//! `tests/`, where Cargo builds every file as a program of its own. To run
//! one file's tests: `cargo test -p ascribe-emit --test all zod`.

mod support;

#[path = "all/agents.rs"]
mod agents;
#[path = "all/assets.rs"]
mod assets;
#[path = "all/formatted.rs"]
mod formatted;
#[path = "all/plain.rs"]
mod plain;
#[path = "all/quill.rs"]
mod quill;
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
