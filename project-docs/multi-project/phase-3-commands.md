# Phase 3: Commands, preview, and format on save work per project

Part of [multi-project workspaces](README.md). Requires phase 2. Extension only.

## Goal

Everything that talked to "the server" talks to the server that owns the file in question: the preview, "Select Preview Build", format on save, "Restart Language Server", and "Show Server Output".

## Context

- `packages/vscode/src/extension.ts`: format on save calls `server.request("textDocument/formatting", ...)` for any Markdown document; `ascribe.restartServer` checks `hasProject()` then `server.restart()`; `ascribe.showOutput` calls `server.showOutput()`.
- `packages/vscode/src/preview/controller.ts` (about 600 lines): `PreviewController(context, server)` sends `ascribe/preview` through `this.server.request(PREVIEW_REQUEST, params)` (around line 278), subscribes to `this.server.onDidStart` (around line 134), keeps one `selectedBuild`, and shows the message "isn't a source of the project (it is outside the content root)" when the server answers that the file isn't a page. `selectPreviewBuild` (around line 467-491) lists `result.builds`.
- `packages/vscode/src/preview/protocol.ts`: the preview request and result types.
- `ProjectRegistry` from phase 2: `projectFor(uri)`, `ensureStartedFor(uri)`.

## Tasks

1. **Preview routing.** `PreviewController` takes the registry. For the document being previewed, find the owner with `projectFor`, call `ensureStartedFor` (the preview opening is a trigger for lazy start), and send the request to that project's server. When the previewed editor changes to a file in another project, switch servers and reload.
   - Keep `selectedBuild` per project (a `Map` keyed by project folder). A build chosen in one project must not leak into another.
   - Re-render on that project's `onDidStart` (restart), not on any server's.
2. **Preview messages.** Distinguish three cases in the user-facing text:
   - The file is in no project: "This file isn't part of an Ascribe project (no ascribe.toml above it)."
   - The file is in a project's folder but outside its content root: name the project (`<folder>`) and its content root.
   - The owner's server failed to start: say so, and offer "Show Output" for that project.
     The server's answer for "not a source" already exists; take the content root from the project's model if the preview result carries it, otherwise from a small addition to `ascribe/preview`'s error (a server change is allowed here only if unavoidable; if so, add it to `crates/tessera-lsp` with a test and document it in `crates/tessera-lsp/README.md`).
3. **Format on save.** Send the formatting request to the owning project's server; for a file with no running server owner, do nothing (no error).
4. **Commands.**
   - `ascribe.restartServer`: restart every _running_ server; do not start idle ones. If there are no projects, keep today's "this workspace has no ascribe.toml" message.
   - `ascribe.showOutput`: show the channel of the active editor's project; with no active Ascribe file and several projects, show a quick-pick of projects; with one, show its channel.
   - `ascribe.openPreview` / `ascribe.selectPreviewBuild`: act on the active editor's project.
5. **Settings.** `ascribe.path` change restarts every running server; `ascribe.maxCrashes` updates every crash counter.
6. **Context keys.** `ascribe.active` stays true when the workspace has any project. Keep the `when` clauses in `package.json` working.
7. Update unit tests for the preview controller (`test/unit/preview.test.ts`) and add unit tests for the per-project build map and message selection.

## Out of scope

Cross-project rename or links; new fixtures (phase 4); docs (phase 5).

## Acceptance criteria

- Previewing a page in `examples/quill`, then one in `examples/astro-site`, shows each in its own project, with an independent build choice.
- A file outside every project gets the "not part of an Ascribe project" message, not the content-root message.
- Format on save in project A never contacts project B's server.
- "Restart Language Server" restarts running servers and leaves idle ones idle.
- `pnpm --filter ascribe-vscode test`, `typecheck`, and `test:integration` pass.

## Verify

```sh
pnpm --filter ascribe-vscode typecheck
pnpm --filter ascribe-vscode test
pnpm --filter ascribe-vscode test:integration
pnpm lint && pnpm format:check
```

## Commits

1. "Route the preview to the project that owns the page".
2. "Make commands, format on save, and settings per project".
