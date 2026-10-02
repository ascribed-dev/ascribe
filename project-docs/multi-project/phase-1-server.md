# Phase 1: The server looks only upward and says which project it chose

Part of [multi-project workspaces](README.md). Rust only; no extension changes.

## Goal

`ascribe lsp` finds its project from the workspace folders it is given by walking _up_ from each folder, never down. It logs the chosen `ascribe.toml` every time, and says plainly when there is none.

## Context

- `crates/tessera-lsp/src/core.rs`:
  - `Core::start` (around line 172) calls `find_config(&self.folders)` and logs the choice only when `self.folders.len() > 1`.
  - `find_config` (around line 740) first tries `tessera_check::Project::find_config(folder)` (upward), then searches up to four directory levels _down_, skipping hidden directories, `node_modules`, and `target`. The downward search is the bug.
- `crates/tessera-check/src/project.rs`: `Project::find_config(start)` walks `start.ancestors()` and returns the first `ascribe.toml`. Leave it as is.
- `crates/tessera-lsp/tests/support/mod.rs`: `Client::start(root)` launches the server in-process for tests. `scenarios.rs` holds general scenarios.
- The log goes through `Core::log`, which writes `tessera-lsp: <text>` to stderr; VS Code shows stderr in the Ascribe output channel.

## Tasks

1. In `find_config`, delete the downward search. Keep the upward lookup over each folder, in folder order.
2. In `Core::start`, always log `using the project at <path>` when a project is found, not only with several folders. When none is found, log `no ascribe.toml at or above <folder>` for each folder (one line listing them is fine), and make sure no diagnostics are published and requests that need a project answer as they do today when there is no project.
3. Remove now-unused helpers and constants. Update the doc comment on `find_config`, and remove the stray `// One project per server.` line above it, replacing it with a doc sentence on the server: one project per server, found upward from the workspace folder.
4. Tests, in `crates/tessera-lsp/tests/scenarios.rs` (or a new file if it fits better):
   - A workspace folder that has no `ascribe.toml` at or above it but a project one level down: the server loads no project and publishes no diagnostics for a file under that project.
   - A workspace folder inside a project (for example the content root): the server finds the parent's `ascribe.toml`.
   - A folder that is itself the project root: unchanged behaviour.
     Use a temporary directory for fixtures, not repository paths. If the logs are testable through the existing harness, assert the "using the project" line; otherwise unit-test the log text builder.
5. If any existing test relied on the downward search (search for tests that open a parent directory of a project), change it to open the project folder.

## Out of scope

Any extension (`packages/vscode`) change; the `ascribe check` CLI; supporting several projects in one server.

## Acceptance criteria

- `cargo test -p tessera-lsp --locked` passes, including the new tests.
- Opening a folder with a project only below it yields "no ascribe.toml at or above" and no project.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` and `cargo fmt --all --check` pass.

## Verify

```sh
cargo test -p tessera-lsp --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Manual check: run `ascribe lsp` against the repo root with a small stdio script and confirm stderr says no project, then against `examples/quill` and confirm it names `examples/quill/ascribe.toml`.

## Commit

One commit: "Stop the language server guessing a project below the workspace folder".
