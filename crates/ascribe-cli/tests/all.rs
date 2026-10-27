//! Every integration test of `ascribe-cli` in one program, apart from
//! `determinism.rs`, which CI runs in a job of its own. One program is one
//! link and one process, where a file per program was fourteen of each; the
//! tests themselves are the files in `all/`, unchanged. To run one file's
//! tests: `cargo test -p ascribe-cli --test all agents::`. The filter is a
//! substring of a test's full name, so the `::` keeps it to that file.
//!
//! This file is a crate root, so it looks for its modules beside itself;
//! each one names its file.

#[path = "support/counting.rs"]
mod counting;
// `links.rs` panics on a link it can't make, as a test helper may.
#[allow(clippy::expect_used, clippy::panic)]
#[path = "../../../tests/support/links.rs"]
mod links;

#[path = "all/addresses.rs"]
mod addresses;
#[path = "all/agents.rs"]
mod agents;
#[path = "all/answers.rs"]
mod answers;
#[path = "all/baseline.rs"]
mod baseline;
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
#[path = "all/init.rs"]
mod init;
#[path = "all/lsp_parity.rs"]
mod lsp_parity;
#[path = "all/lsp_project_log.rs"]
mod lsp_project_log;
#[path = "all/mcp.rs"]
mod mcp;
#[path = "all/navigation.rs"]
mod navigation;
#[path = "all/network.rs"]
mod network;
#[path = "all/new.rs"]
mod new;
#[path = "all/observe.rs"]
mod observe;
#[path = "all/openapi.rs"]
mod openapi;
#[path = "all/output.rs"]
mod output;
#[path = "all/page_parity.rs"]
mod page_parity;
#[path = "all/permalinks.rs"]
mod permalinks;
#[path = "../../ascribe-diff/tests/planted/mod.rs"]
mod planted;
#[path = "all/preview.rs"]
mod preview;
#[path = "all/program_starts.rs"]
mod program_starts;
#[path = "all/read_once.rs"]
mod read_once;
#[path = "all/rename.rs"]
mod rename;
#[path = "all/report.rs"]
mod report;
#[path = "all/sources.rs"]
mod sources;
#[path = "all/templates.rs"]
mod templates;
#[path = "all/vale.rs"]
mod vale;
#[path = "all/versions.rs"]
mod versions;
