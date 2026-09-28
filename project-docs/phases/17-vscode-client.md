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

_To be filled in by the implementing agent._
