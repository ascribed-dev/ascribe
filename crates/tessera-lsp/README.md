# tessera-lsp

The Tessera language server, run as `tessera lsp` (LSP over stdio). It keeps a
project in memory, follows every change to it, and publishes the diagnostics
`tessera check --build <editor build>` reports, as the author types. It
computes nothing itself: every diagnostic comes from `tessera_check`, over a
`tessera_resolve::IncrementalProject`, so the editor and the command line can't
disagree (SPEC §8, §10).

This is phase 15: initialization, document and file synchronization,
diagnostics, and semantic tokens. Completion, hover, navigation, CodeLens, and
inlay hints are phase 16; code actions, rename, and formatting are phase 24.

## Semantic token legend

**This legend is a contract with the VS Code client (phase 17). The order of the
types and modifiers is what the wire format indexes, so entries are only ever
appended, never reordered or renamed.** The server sends the legend in its
`initialize` result; a client that reads it from there needs nothing from here
except the suggested theme scopes.

Token types, in legend order:

| # | Type | What it marks | Suggested TextMate scope (for `semanticTokenScopes`) |
|---|---|---|---|
| 0 | `tesseraDirective` | The `@` and name of a built-in directive (`@note`, `@include`, `@variant`, `@id`, …) | `keyword.control.directive.tessera` |
| 1 | `tesseraWidget` | The `@` and name of a project widget (declared in `[widgets]`) | `entity.name.function.widget.tessera` |
| 2 | `tesseraAttributeKey` | An attribute key in a directive's or an image's `{…}` block | `entity.other.attribute-name.tessera` |
| 3 | `tesseraAttributeValue` | An attribute value: a token, a quoted string (with its quotes), or one member of a value set | `string.unquoted.attribute-value.tessera` |
| 4 | `tesseraColon` | The `:` that ends a directive's head: the container colon, or the colon before a primary | `punctuation.separator.directive.tessera` |
| 5 | `tesseraEnd` | The `@end` of an end line | `keyword.control.end.tessera` |
| 6 | `tesseraTitle` | A title line: the `.` and the title's text (SPEC §3.7). Its own type, so a paragraph that accidentally became a title stands out (SPEC §10) | `markup.heading.title.tessera` |
| 7 | `tesseraPhrase` | A declared phrase, `{key}` with its braces (SPEC §5.1) | `variable.other.phrase.tessera` |
| 8 | `tesseraPhraseUndeclared` | A `{key}` whose key the content model doesn't declare: literal text, marked so a typo shows | `invalid.illegal.phrase-undeclared.tessera` |
| 9 | `tesseraAvailability` | An availability spec: the primary of `@available` (`cloud, self-managed preview 3.3`) | `constant.other.availability.tessera` |

Token modifiers, in legend order (bit *n* of a token's modifier set is
modifier *n*):

| # | Modifier | Set on |
|---|---|---|
| 0 | `unknown` | A `tesseraAttributeKey` the directive's schema (or, for an image, `[images.attributes]`) doesn't declare |

A token never overlaps another: a phrase inside a title line splits the title
token around it. Tokens that span lines are split per line. Positions and
lengths use the negotiated position encoding (below). The server serves
`textDocument/semanticTokens/full` and `…/range`; there is no delta.

## Position encoding

The server uses UTF-16 unless the client offers UTF-8 in
`general.positionEncodings`, and answers with the one it chose in
`positionEncoding`. Every conversion between a byte offset and a position goes
through `tessera_core::LineIndex`. (The command line's JSON counts columns in
characters; the two differ for a line with an astral-plane character, and the
parity test converts before comparing.)

## Capabilities

Advertised: incremental text document sync (open/close, no save), semantic
tokens (full and range), and the position encoding. Registered dynamically after
`initialized`, when the client allows it: `workspace/didChangeWatchedFiles`.
Diagnostics are pushed (`textDocument/publishDiagnostics`); the server doesn't
advertise pull diagnostics.

## How it works

- **The project** is a `tessera_resolve::IncrementalProject` built from the
  `tessera.toml` in a workspace folder (the nearest one at or above the
  folder, or the first one found below it). Ids follow phase 13: source files
  have ids from 1, `tessera.toml` is 0, an id names a path and is never reused.
- **Changes** reach it as `Change`s from three sources: open documents
  (`didOpen`, `didChange`, `didClose`; an open document's text wins over the
  file on disk, and closing one reverts to the disk), the file watcher (files
  and directories created, changed, deleted, or moved by anything else, sources
  and assets alike), and `tessera.toml` (open or on disk).
- **The model.** A `tessera.toml` that loads is applied as `Change::Model`
  (phase 13's tiers decide what is re-parsed, re-indexed, or re-resolved). One
  that doesn't load has its problems published on `tessera.toml`, and the
  project keeps the last model that did (Q131). A change to the content root or
  output directory can't be applied in place (`ApplyError::LayoutChanged`, Q92),
  so the server reloads the whole project and republishes everything.
- **Diagnostics** are `tessera_check::check_file` for each file in
  `Affected::recheck`, plus the page-level diagnostics of the editor's build
  (`[editor] build`; `ContentModel::editor_default_build`) located in those
  files, published for every affected file, open or not. A deleted file's
  diagnostics are cleared. The file-level checks probe the disk with the files
  the editor and the watcher have reported layered over it
  (`Project::from_parts_with_fs`), as the source index does.
- **Stale results.** A worker thread computes from a `Snapshot`. Before it
  publishes a file's diagnostics it checks, under the lock that serializes
  every change, that `IncrementalProject::is_file_current` holds for the file
  and that an open document still has the version the result was computed for.
  A result that fails either check is dropped, and the file stays queued for the
  newer snapshot. Work overtaken by a newer edit is abandoned between stages.
- **Robustness.** A panic while handling a request or a notification, or while
  computing diagnostics, is caught, logged to stderr, and answered with an
  internal error where a response is owed; the server keeps running. Nothing is
  ever written to stdout except LSP messages.

## Library choice

Chosen: **`lsp-server` with `lsp-types`** (rust-analyzer's synchronous
scaffold and the protocol types), over `tower-lsp-server`.

- The work is CPU-bound and has to be cancellable and strictly ordered: a
  computation overtaken by a newer edit must never publish, and a publish must
  be atomic with the check that it's still current. That is a lock and a
  worker thread, which `lsp-server`'s plain channels fit directly; under
  `tower-lsp-server` every handler is an async task on a runtime, and the same
  guarantees need explicit ordering around its concurrent request handling.
- `lsp-server` owns the message loop, the `initialize` handshake, and the
  shutdown protocol, and nothing else: no async runtime (`tokio`) in the
  dependency tree of the binary that also runs `check` and `build`.
- Handling a panic per request is a `catch_unwind` around a plain function call.
- `tower-lsp-server`'s advantages (typed request handlers, less boilerplate)
  matter more for the many small requests of phases 16 and 24. Those phases can
  add a dispatch table over the same loop; if a later phase's requests turn out
  to need async, the state and the worker in this crate don't depend on the
  loop.

## Testing

- `tests/` drives the server in-process over `Connection::memory()`: the
  scripted scenarios of the acceptance criteria, multi-byte positions, stale
  computations (a hook holds a computation until a newer edit lands), file
  watching, model changes.
- `crates/tessera-cli/tests/lsp_parity.rs` starts the real `tessera lsp` binary
  over stdio and compares its published diagnostics with
  `tessera check --build <name> --format json`, for every build of
  `examples/quill` and fixture projects with known problems.
- `benches/keystroke.rs` records the time from an edit to its diagnostics.
