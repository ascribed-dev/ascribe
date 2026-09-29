# Ascribe for VS Code

The client for Ascribe's language server (`ascribe lsp`), plus immediate syntax
highlighting. The extension stays thin: language intelligence (diagnostics,
semantic tokens, and later completion and navigation) comes from the server.

It activates when the workspace contains a `ascribe.toml`.

## The `ascribe` binary

The extension looks for the binary, in order:

1. the `ascribe.path` setting, if set (a path that doesn't work is an error; it
   doesn't fall back);
2. the project's `node_modules/.bin/ascribe`, looking in each folder that holds
   a `ascribe.toml` and its parents up to the workspace folder;
3. the binary bundled in the extension, `bin/<platform>-<arch>/ascribe`
   (`ascribe.exe` on Windows). Phase 27 packages these.

It runs `ascribe --version` on the candidate, and warns when the version is
older than `ascribe.minServerVersion` in `package.json`. When nothing is found
it says how to fix that.

## Settings and commands

| | |
|---|---|
| `ascribe.path` | Path to the `ascribe` binary. |
| `ascribe.trace.server` | `off`, `messages`, or `verbose`. |
| `ascribe.formatOnSave` | Used by the formatter (phase 24); no effect yet. |
| `ascribe.maxCrashes` | Crashes (since the last manual restart) after which the server isn't restarted again. Default 5. |
| **Ascribe: Restart Language Server** | Stops and starts the server; forgets earlier crashes. |
| **Ascribe: Show Server Output** | Opens the output channel. |

## Highlighting

`syntaxes/` holds two TextMate injections into markdown (one for top level, one
for lists and quotes, where the item's indentation is unknown): directive lines
(sigil, name, attribute block, colon, and the first line of the primary), `@end`,
and `{key}` phrases. TextMate can't see past a line, and doesn't know the content
model, so the rest is the server's semantic tokens: title lines, declared and
undeclared phrases, project widgets, and a text primary's later lines. The
`semanticTokenTypes` and `semanticTokenScopes` in `package.json` map the server's
legend (`crates/tessera-lsp/README.md`) to theme scopes; a unit test keeps them
in step.

## Development

```
pnpm --filter ascribe-vscode build              # bundle to dist/extension.cjs
pnpm --filter ascribe-vscode test               # unit tests (vitest)
pnpm --filter ascribe-vscode test:integration   # VS Code integration tests
```

The integration tests download VS Code into `out/vscode-test` and need a
display: on Linux without one, use `pnpm --filter ascribe-vscode
test:integration:headless` (it runs under `xvfb-run -a`). They have three
suites: `activation` (no `ascribe.toml`: the extension stays off), `stub` (a
stub server in `test/stub-server`), and `quill` (the real `ascribe lsp` on a
copy of `examples/quill` with a broken page added), which runs only when
`ASCRIBE_BIN` names a built `ascribe`. `ASCRIBE_SUITE` runs one suite.

`test/fixtures/markdown.tmLanguage.json` is VS Code's markdown grammar (MIT,
microsoft/vscode), so the grammar tests see the scopes it really produces.
