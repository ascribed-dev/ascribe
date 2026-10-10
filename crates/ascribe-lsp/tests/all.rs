//! Every integration test of `ascribe-lsp` in one program. One program is
//! one link and one process, where a file per program was fourteen of each;
//! the tests themselves are the files in `all/`, unchanged, over the scripted
//! client in `support/`. To run one file's tests:
//! `cargo test -p ascribe-lsp --test all navigation::`. The filter is a
//! substring of a test's full name, so the `::` keeps it to that file.
//!
//! This file is a crate root, so it looks for its modules beside itself;
//! each test file names its own.

mod support;

#[path = "all/across.rs"]
mod across;
#[path = "all/agent_prompt.rs"]
mod agent_prompt;
#[path = "all/build_view.rs"]
mod build_view;
#[path = "all/context.rs"]
mod context;
#[path = "all/differential.rs"]
mod differential;
#[path = "all/edit.rs"]
mod edit;
#[path = "all/edit_examples.rs"]
mod edit_examples;
#[path = "all/model_edits.rs"]
mod model_edits;
#[path = "all/navigation.rs"]
mod navigation;
#[path = "all/preview.rs"]
mod preview;
#[path = "all/quick_fixes.rs"]
mod quick_fixes;
#[path = "all/references.rs"]
mod references;
#[path = "all/review.rs"]
mod review;
#[path = "all/scenarios.rs"]
mod scenarios;
#[path = "all/tokens.rs"]
mod tokens;
