# Phase 2: One lazily started server per project

Part of [multi-project workspaces](README.md). Requires phase 1. Extension only (`packages/vscode`).

## Goal

The extension finds every `ascribe.toml` in the workspace and runs one language server for each, started the first time a file of that project is needed. A file belongs to the nearest `ascribe.toml` above it.

## Context

- `packages/vscode/src/client.ts`: `ServerController` is today's single server. It resolves the binary (`resolveBinary`), builds `LanguageClient("ascribe", "Ascribe", serverOptions(binary), clientOptions(...))`, tracks `status`, a `CrashCounter`, one `LogOutputChannel` named "Ascribe", `restart()`, `request()`, `whenSettled()`, and `onDidStart`. The document selector is `{scheme:"file", language:"markdown"}` plus `**/ascribe.toml`. `projectRoots()` already finds each `ascribe.toml` and its parents up to the workspace folder.
- `packages/vscode/src/binary.ts`: `resolveBinary`, `ancestorsWithin`. Already pure and unit-tested in `test/unit/binary.test.ts`.
- `packages/vscode/src/extension.ts`: creates one `ServerController` and one `PreviewController`, activates only when `hasProject()`.
- `packages/vscode/package.json`: activation is `workspaceContains:**/ascribe.toml`; settings under `contributes.configuration`.
- The server takes its project from the workspace folder it is initialised with. The language client sends the workspace folders of the window. To root a server at a project, pass `workspaceFolder` in `LanguageClientOptions` (a `WorkspaceFolder` for the project folder); verify the server receives exactly that as its only workspace folder.

## Design

New file `src/projects.ts` (pure where possible, unit-testable without `vscode`):

```ts
interface Project {
  readonly config: string; // absolute path of ascribe.toml
  readonly folder: string; // its directory
}
/** The project that owns a file: the nearest ancestor ascribe.toml. */
function owningProject(file: string, projects: readonly Project[]): Project | undefined;
/** The projects nested inside `project`, whose files it doesn't own. */
function nestedProjects(project: Project, projects: readonly Project[]): Project[];
```

`ServerController` becomes `ProjectServer`: one per `Project`, with its own state, crash counter, binary resolution (`projectRoots` is the project folder up to its workspace folder), output channel, and client. A new `ProjectRegistry`:

- Discovers projects with `vscode.workspace.findFiles("**/ascribe.toml", "**/node_modules/**", 50)`, and watches `**/ascribe.toml` (`createFileSystemWatcher`) to add, remove, or restart projects on create, delete, and change of location. It starts no process while discovering.
- `projectFor(uri)` returns the owning `ProjectServer`, if any.
- `ensureStarted(project)` starts a server, once; concurrent calls share one start.
- Starts a server when a Markdown or `ascribe.toml` file under a project becomes relevant: for every already open document at activation, on `onDidOpenTextDocument`, and on `onDidChangeActiveTextEditor`. Also exposes `ensureStartedFor(uri)` for the preview (phase 3).
- `ascribe.startServers`: `"onDemand"` (default) or `"all"`; with `"all"`, every discovered project starts at discovery. Add the setting to `package.json` with an enum, descriptions, and `scope: "window"`.
- A running server stays running until the window closes, its `ascribe.toml` is deleted, or it is restarted. No idle shutdown.
- Output channels are named `Ascribe (<folder relative to its workspace folder>)`; with only one project in the workspace the name is `Ascribe`. Each server logs the binary it uses on start, as today.
- `whenSettled()` settles when every server that is starting has finished. The `AscribeApi` in `extension.ts` keeps `binary()`/`state()` working for tests: with several projects, accept an optional project folder argument, defaulting to the only or first project. Keep the existing signatures usable by current tests.

### Scoping documents to a project

Each client's `documentSelector` is limited to its folder: `{ scheme: "file", language: "markdown", pattern: "<folder>/**" }` plus `{ scheme: "file", pattern: "<folder>/ascribe.toml" }`. Escape glob metacharacters in the folder path, and use forward slashes in the pattern on Windows.

A selector can't exclude a nested project's folder. So that the parent doesn't also receive nested projects' files, guard in `middleware`: the document sync notifications (`didOpen`, `didChange`, `didClose`, `didSave`, `willSave`) and the feature providers (hover, completion, definition, code action, formatting, rename, semantic tokens, inlay hints, code lens, document links, and so on) skip any document whose owning project isn't this server. Check what `vscode-languageclient` lets you intercept; if a clean guard isn't possible for some feature, report it and propose the smallest alternative rather than silently leaving files double-served.

Before writing the registry, spend a short spike confirming (a) `workspaceFolder` in client options makes the server see only the project folder, and (b) a file in a nested project is not sent to the parent. Record the result in the PR description.

## Tasks

1. Add `src/projects.ts` with `owningProject` and `nestedProjects`, and `test/unit/projects.test.ts`: nearest ancestor wins, nested projects, siblings, a file outside every project, path-prefix lookalikes (`/a/docs` vs `/a/docs-old`), Windows drive-letter case and separators.
2. Rename and generalise `ServerController` to `ProjectServer`; add `ProjectRegistry`. Keep crash handling, restart, and error messages as they are, per server.
3. Wire `extension.ts` to the registry: activation discovers projects, sets the `ascribe.active` context, starts servers per the setting. `ascribe.path`, `ascribe.maxCrashes` and `ascribe.trace.server` changes apply to every server.
4. Add `ascribe.startServers` and update `test/unit/manifest.test.ts` so it still passes (it checks `package.json`).
5. Keep the existing integration tests passing unchanged for a single-project workspace (`test/fixtures/stub-project`, `broken-quill-docs`, `no-model`).

## Out of scope

Commands, preview routing, format on save (phase 3); new integration fixtures (phase 4); documentation (phase 5). Until phase 3 lands, the commands may target the only or first running server so the build stays green.

## Acceptance criteria

- Opening the repo root with `examples/quill` and `examples/astro-site` gives each its own server and diagnostics; neither shows the other's problems.
- A project's server is not running until one of its files is open, in `"onDemand"` mode; all start in `"all"` mode.
- A nested project's files are served by the nested server only.
- A single-project workspace behaves as before.
- `pnpm --filter ascribe-vscode test`, `typecheck`, and `pnpm test:integration` (headless on Linux: `pnpm test:integration:headless`) pass.

## Verify

```sh
pnpm --filter ascribe-vscode typecheck
pnpm --filter ascribe-vscode test
pnpm --filter ascribe-vscode test:integration
pnpm lint && pnpm format:check
```

## Commits

1. "Add project ownership for multi-project workspaces" (`projects.ts` and unit tests).
2. "Run one language server per ascribe.toml, started on demand".
