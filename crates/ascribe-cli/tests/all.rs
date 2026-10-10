//! Every integration test of `ascribe-cli` in one program, apart from
//! `determinism.rs`, which CI runs in a job of its own. One program is one
//! link and one process, where a file per program was fourteen of each; the
//! tests themselves are the files in `all/`, unchanged. To run one file's
//! tests: `cargo test -p ascribe-cli --test all agents::`. The filter is a
//! substring of a test's full name, so the `::` keeps it to that file.
//!
//! This file is a crate root, so it looks for its modules beside itself;
//! each one names its file.

#[path = "all/agents.rs"]
mod agents;
#[path = "all/answers.rs"]
mod answers;
#[path = "all/build.rs"]
mod build;
#[path = "all/build_view_parity.rs"]
mod build_view_parity;
#[path = "all/check.rs"]
mod check;
#[path = "all/diff.rs"]
mod diff;
#[path = "all/drift.rs"]
mod drift;
#[path = "all/fmt.rs"]
mod fmt;
#[path = "all/hook.rs"]
mod hook;
#[path = "all/lsp_parity.rs"]
mod lsp_parity;
#[path = "all/lsp_project_log.rs"]
mod lsp_project_log;
#[path = "all/mcp.rs"]
mod mcp;
#[path = "all/network.rs"]
mod network;
#[path = "all/output.rs"]
mod output;
#[path = "all/report.rs"]
mod report;
#[path = "all/sources.rs"]
mod sources;
