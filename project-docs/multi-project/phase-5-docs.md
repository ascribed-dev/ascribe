# Phase 5: Documentation

Part of [multi-project workspaces](README.md). Requires phases 1-4 to be final.

## Goal

The docs describe how the extension and server find projects, and say plainly what a repository with several projects gets.

## Context

- `docs/editor.md`: the VS Code guide. Line 13 says the extension activates in a workspace containing an `ascribe.toml`; line 20 describes binary lookup "in each folder that holds an `ascribe.toml`, and its parents up to the workspace folder"; line 29 says diagnostics cover "the whole project"; line 108 describes other editors ("needs the workspace folder that holds `ascribe.toml`").
- `crates/tessera-lsp/README.md`: the server contract; the "one project per server" statement and how the project is found.
- `packages/vscode/README.md` and `packages/vscode/DEVELOPMENT.md`: user and contributor docs for the extension.
- `CHANGELOG.md` at the repository root (the extension packages it as `changelog.md`; `scripts/release/version.ts --check` requires the version's heading).
- `docs/README.md`: index of docs.

## Tasks

1. `docs/editor.md`: add a section "Workspaces with several projects":
   - Each `ascribe.toml` is a project with its own server; a file belongs to the nearest one above it; nested projects are allowed.
   - A server starts the first time you open one of the project's files, or its preview; `ascribe.startServers: "all"` starts every project at once.
   - The Problems panel lists diagnostics only for projects whose server is running. `ascribe check` covers every project (run it from each project, or with `--config`).
   - Each project uses its own `node_modules/.bin/ascribe` when there is one; `ascribe.path` overrides all.
   - Commands act on the active file's project; "Restart Language Server" restarts all running servers.
   - Memory: about 6 MB per server plus about 70 KB per page. Keep this short and give the figure, not the measurement method.
     Update line 20 and line 29 to match, and line 108 for other editors: start one `ascribe lsp` per project, rooted at the folder that holds its `ascribe.toml`; the server doesn't search below the folder.
2. `crates/tessera-lsp/README.md`: state the contract precisely: one project per server; the project is the nearest `ascribe.toml` at or above a workspace folder; no downward search; what it logs. Remove any statement the old behaviour made true.
3. Add the `ascribe.startServers` setting to the extension README's settings list if there is one. Don't duplicate prose already in `docs/editor.md`; link to it.
4. Changelog entry for the next version: servers per project, lazy start, the setting, the server no longer guessing a project below the folder (call out as behaviour change for other editors).
5. If the CLI docs (`docs/cli.md`) say anything about finding `ascribe.toml`, make sure they agree (upward from the current directory; `--config`).
6. Run the docs-related tests: a conformance test checks that generated docs match; this phase should not change generated files. If `docs/diagnostics.md` or other generated docs change, you edited the wrong file.

## Style

Write for someone using Ascribe, not for someone who read the design discussion. No phase history, no "previously"/"now". Short paragraphs; one idea each. Match the tone of the surrounding docs.

## Acceptance criteria

- Every behaviour in this plan's decisions list is stated once, in the place a reader would look.
- Links resolve. `pnpm format:check` passes.
- A reader can answer from the docs alone: "Why does my other project show no problems?" and "How do I use Ascribe in Neovim with several projects?"

## Verify

```sh
pnpm format:check
cargo test --workspace --locked   # includes the conformance test that guards generated docs
```

## Commit

"Document projects in a multi-project workspace".
