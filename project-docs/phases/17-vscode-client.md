# Phase 17: VS Code extension client

**Track:** Editor · **Start after:** 00 · **Finish after:** 15 · **Parallel with:** most of waves B–H · **Unblocks:** 25

## Goal

Build the VS Code extension's client: find the `tessera` binary, start the language server, and provide immediate syntax highlighting. The extension stays thin; the language intelligence lives in `tessera lsp`.

## Read first

- [PLAN.md](../PLAN.md): Key decisions (extension's binary), VS Code extension.
- [SPEC.md](../../SPEC.md): §3, §5.1, and §10.
- warp-writer's TextMate grammar injection (`warp-writer-directives.tmLanguage` in the warp-writer repository) for the technique.

## Deliverables

- `packages/vscode`: the extension, bundled with esbuild.

## Tasks

1. **Scaffold.** A TypeScript extension that activates when the workspace contains `tessera.toml` (`workspaceContains:**/tessera.toml`).
2. **Binary resolution.** In order: the `tessera.path` setting if set; the project's `node_modules/.bin/tessera`; the binary bundled in the extension under `bin/<platform>-<arch>/`. Run `tessera --version`, and warn when the project's binary is older than the version the extension expects. Show a clear error with next steps when no binary is found.
3. **Language client.** Start `tessera lsp` with `vscode-languageclient` over stdio, for markdown documents in the workspace, and forward file-watching registrations (phase 15 relies on them for files that aren't open). Commands: restart the server, show its output. After a configured number of crashes, stop and explain.
4. **Highlighting.** A TextMate grammar injected into markdown for immediate highlighting before the server responds: directive lines (sigil, name, attribute block, colon, primary), `@end`, and `{key}` phrases. TextMate can't look ahead to the next line, so title lines are left to the server's semantic tokens (phase 15). Map the server's semantic token types to theme scopes.
5. **Settings.** `tessera.path`, `tessera.trace.server`, and, for phase 24, `tessera.formatOnSave`.
6. **Before phase 15 exists.** Develop against a stub server that returns canned diagnostics, so this phase isn't blocked. Replace it with the real server for the end-to-end tests that gate finishing.
7. **Tests.** Unit tests for binary resolution. Integration tests with `@vscode/test-electron` that open `examples/quill` and assert that the real server's diagnostics arrive, including after a file changes on disk.

## Acceptance criteria

- [ ] The extension activates only in workspaces with `tessera.toml`.
- [ ] Binary resolution follows the order above; unit tests cover each branch, including "none found".
- [ ] Directive lines, `@end`, and phrases are highlighted by the TextMate grammar alone.
- [ ] With phase 15's server, the integration tests see diagnostics for `examples/quill`, and updated diagnostics after an on-disk change.

## Out of scope

- Packaging platform-specific extensions and publishing (phase 27).
- The preview (phase 25), and quick fixes and refactoring (phase 24).
- Extension UI beyond what's listed here (see `future-things.md` at the repository root).

## Handoff notes

### What was built

- **`packages/vscode`** (`tessera-vscode`, private), bundled with esbuild to `dist/extension.cjs` (`pnpm --filter tessera-vscode build`). It activates on `workspaceContains:**/tessera.toml`.
  - `src/binary.ts`, `version.ts`, `crash.ts`: pure logic, no `vscode` import, unit-tested. `resolveBinary` tries the `tessera.path` setting, then `node_modules/.bin/tessera` under each folder that holds a `tessera.toml` (and its parents up to the workspace folder), then `bin/<platform>-<arch>/tessera[.exe]` in the extension. Each candidate must run `--version`; one older than `tessera.minServerVersion` (package.json) yields a warning. Nothing found gives an error with next steps and every place tried.
  - `src/client.ts`, `extension.ts`, `environment.ts`: `ServerController` starts `tessera lsp` with `vscode-languageclient` over stdio for `file` markdown documents and `tessera.toml`, with the commands `tessera.restartServer` and `tessera.showOutput`, a custom error handler that stops after `tessera.maxCrashes` crashes and explains, and a restart when `tessera.path` changes. `activate` returns `{ binary(), state(), whenSettled() }` for tests.
  - `syntaxes/`: two TextMate injections into `text.html.markdown` (top level, and inside lists and quotes): directive lines, `@end`, `{key}` phrases. See Q121.
  - `package.json` maps phase 15's semantic token legend (all ten types and the `unknown` modifier) to the scopes phase 15 suggests, and turns semantic highlighting on for markdown.
- **Tests.** Unit (vitest, 63): binary resolution (every branch: setting, project, bundled, none, unrunnable, versions, platforms), versions, crash counter, the manifest (private, activation event, settings), a contract test that reads the legend table in `crates/tessera-lsp/README.md` and checks `package.json` covers it (skipped while that README has no legend), and the grammar, tokenized with `vscode-textmate` against VS Code's own markdown grammar (`test/fixtures/markdown.tmLanguage.json`). Integration (`@vscode/test-electron`, three suites in `test/integration/`): `activation`, `stub` (a stub server, `test/stub-server/tessera`: canned diagnostics, dynamic watcher registration, crashes on demand), and `quill` (the real `tessera lsp` on a copy of `examples/quill` plus a broken page).
- **Root files touched**, per Q126: `pnpm-workspace.yaml` (`allowBuilds: esbuild: false`) and a `vscode` job in `.github/workflows/js.yml` (still `workflow_dispatch` only).

### Interfaces later phases use

- **24 (format on save):** the `tessera.formatOnSave` setting exists and is unused. The client needs no change to offer formatting (`vscode-languageclient` wires a server's `documentFormattingProvider`); on-save formatting is phase 24's.
- **25 (preview):** `ServerController` (`src/client.ts`) owns the client and the resolved binary (`binary`), and `activate` returns them through `TesseraApi`; add a webview provider in `extension.ts` next to the commands. The binary resolution (`resolveBinary`) can be reused to run `tessera build`.
- **Phase 15:** the client relies on the server registering `workspace/didChangeWatchedFiles` itself (Q124), reading the legend from its `initialize` result, and logging to stderr.

### Decisions

Q121 to Q126 (`project-docs/questions.md`), all with the proposed behavior implemented.

### Left open

- **The end-to-end gate is met.** With phase 15 merged, all three integration suites pass in VS Code 1.139.1, headless: `cargo build -p tessera-cli`, then `TESSERA_BIN=$PWD/target/debug/tessera xvfb-run -a pnpm --filter tessera-vscode test:integration` (`activation` 1, `stub` 8, `quill` 5). The `quill` suite has diagnostics for a page that isn't open, no diagnostics for the example's correct pages, updated diagnostics after an on-disk change, and updated diagnostics for an edit in an open document. The legend contract test runs against phase 15's README (it no longer skips).
- **Windows and macOS.** The stub server is an extensionless script with a shebang, so the stub suite runs on Linux and macOS only; the `.cmd` shim handling in `environment.ts` is untested.
- **Activation depth versus server discovery.** `workspaceContains:**/tessera.toml` matches at any depth, but the server looks at most four levels below a workspace folder (`find_config` in `crates/tessera-lsp/src/core.rs`), so a `tessera.toml` nested deeper activates the extension while the server loads no project.
- **Workspace trust.** The extension declares `untrustedWorkspaces.supported: false`, since it runs the project's own binary; in Restricted Mode it stays off.
- **Bundled binaries** (`bin/<platform>-<arch>/`) don't exist yet; phase 27 packages them.
