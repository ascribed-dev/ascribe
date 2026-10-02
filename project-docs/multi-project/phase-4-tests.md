# Phase 4: Integration tests for multi-project workspaces

Part of [multi-project workspaces](README.md). Requires phases 1-3.

## Goal

End-to-end coverage that fails if a workspace with several projects regresses to one server, the wrong project, or the wrong preview.

## Context

- `packages/vscode/test/integration/run.ts` and `test/integration/suite/`: the VS Code extension host tests (`pnpm --filter ascribe-vscode test:integration`; headless on Linux with `test:integration:headless`). Read how the existing suites open a workspace folder and wait for the server (`whenSettled`).
- `packages/vscode/test/fixtures/`: `stub-project`, `broken-quill-docs`, `no-model`. `test/stub-server/ascribe` is a stand-in server used by some tests; check whether the new tests should use the real bundled binary (as the smoke tests do) or the stub.
- `scripts/release/smoke-vsix.ts` and `smoke/suite.cts`: how the packaged extension is tested against the real binary. Do not change them in this phase unless the registry's API changed what they read.
- Windows pitfalls are listed in the README's rules. The e2e watcher can drop a second change within about 50 ms; re-touch the file in tests that depend on a rebuild.

## Fixture

Add `packages/vscode/test/fixtures/monorepo/` (committed, small), shaped like a real repository:

```
monorepo/
  code/                       # no ascribe.toml; has a README.md that must not be checked
  docs/                       # project A
    ascribe.toml
    docs/index.md             # one broken link: ASC036
  handbook/                   # project B
    ascribe.toml
    pages/index.md            # clean
    pages/nested/             # project C, nested inside B
      ascribe.toml
      content/index.md        # one different problem
```

Use real, minimal content models copied from `examples/quill/ascribe.toml` (adjust `content-root`). Keep each project to a few pages.

## Tests

Open `monorepo/` as the workspace folder, with the bundled binary, and assert:

1. **Ownership and isolation.** After opening `docs/docs/index.md`, only that project's diagnostics appear for it (the `ASC036`), and `vscode.languages.getDiagnostics` for `handbook/pages/index.md` is empty because its server hasn't started.
2. **Lazy start.** At activation no server is running. After opening a file in B, exactly B's server is running; A and C are not.
3. **Nested projects.** A file in C is diagnosed by C only (not duplicated by B); a file in B but outside C is diagnosed by B only.
4. **Preview routing.** Preview a page in A, then one in B; each renders, with independent build selection (use the preview API in `AscribeApi.preview`).
5. **Not in a project.** Previewing `code/README.md` gives the "not part of an Ascribe project" message.
6. **Restart.** `ascribe.restartServer` restarts the running servers and does not start idle ones.
7. **`startServers: "all"`.** With the setting changed, every project's server runs.
8. **Discovery.** Create an `ascribe.toml` in a new folder during the test: a server becomes available for its files once one is opened. Delete it: the server stops.
9. **Single project unchanged.** The existing suites still pass with their fixtures.

Also add a Rust-side check that the server's log names its project, if phase 1 didn't already.

## Out of scope

Performance measurements; new product behaviour. If a test exposes a bug, fix it in the phase that owns the code and note it in the PR.

## Acceptance criteria

- The new suite passes locally and in CI on all three operating systems (`VS Code extension integration tests` runs on Ubuntu; check that the Windows and macOS smoke jobs in `release.yml` still pass when run as a dry run).
- No test depends on timing beyond `whenSettled`/explicit waits with generous timeouts; none uses a fixed sleep except the documented watcher re-touch.

## Verify

```sh
pnpm --filter ascribe-vscode test:integration
pnpm -r test && pnpm lint && pnpm format:check
```

## Commit

"Test multi-project workspaces end to end".
