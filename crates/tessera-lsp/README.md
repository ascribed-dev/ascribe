# tessera-lsp

The Ascribe language server, run as `ascribe lsp` (LSP over stdio). It keeps a
project in memory, follows every change to it, and publishes the diagnostics
`ascribe check --build <editor build>` reports, as the author types. It
computes nothing itself: every diagnostic comes from `tessera_check`, over a
`tessera_resolve::IncrementalProject`, so the editor and the command line can't
disagree (SPEC §8, §10).

Phase 15 built initialization, document and file synchronization, diagnostics,
and semantic tokens. Phase 16 added completion, hover, go to definition,
document links, CodeLens, and inlay hints (see [Navigation](#navigation));
phase 24 adds code actions, rename, and formatting; the `ascribe/preview`
request is phase 25.

## Semantic token legend

**This legend is a contract with the VS Code client. The order of the
types and modifiers is what the wire format indexes, so entries are only ever
appended, never reordered or renamed.** The server sends the legend in its
`initialize` result; a client that reads it from there needs nothing from here
except the suggested theme scopes.

Token types, in legend order:

| # | Type | What it marks | Suggested TextMate scope (for `semanticTokenScopes`) |
|---|---|---|---|
| 0 | `ascribeDirective` | The `@` and name of a built-in directive (`@note`, `@include`, `@variant`, `@id`, …) | `keyword.control.directive.ascribe` |
| 1 | `ascribeWidget` | The `@` and name of a project widget (declared in `[widgets]`) | `entity.name.function.widget.ascribe` |
| 2 | `ascribeAttributeKey` | An attribute key in a directive's or an image's `{…}` block | `entity.other.attribute-name.ascribe` |
| 3 | `ascribeAttributeValue` | An attribute value: a token, a quoted string (with its quotes), or one member of a value set | `string.unquoted.attribute-value.ascribe` |
| 4 | `ascribeColon` | The `:` that ends a directive's head: the container colon, or the colon before a primary | `punctuation.separator.directive.ascribe` |
| 5 | `ascribeEnd` | The `@end` of an end line | `keyword.control.end.ascribe` |
| 6 | `ascribeTitle` | A title line: the `.` and the title's text (SPEC §3.7). Its own type, so a paragraph that accidentally became a title stands out (SPEC §10) | `markup.heading.title.ascribe` |
| 7 | `ascribePhrase` | A declared phrase, `{key}` with its braces (SPEC §5.1) | `variable.other.phrase.ascribe` |
| 8 | `ascribePhraseUndeclared` | A `{key}` whose key the content model doesn't declare: literal text, marked so a typo shows | `invalid.illegal.phrase-undeclared.ascribe` |
| 9 | `ascribeAvailability` | An availability spec: the primary of `@available` (`cloud, self-managed preview 3.3`) | `constant.other.availability.ascribe` |

Token modifiers, in legend order (bit *n* of a token's modifier set is
modifier *n*):

| # | Modifier | Set on |
|---|---|---|
| 0 | `unknown` | An `ascribeAttributeKey` the directive's schema (or, for an image, `[images.attributes]`) doesn't declare |

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

## The preview request: `ascribe/preview`

A custom request that renders a page the way the published site
does, for the editor's preview. It answers from the current snapshot, so it
includes unsaved edits, and it writes nothing. For a document and a build it
resolves the page for the build (`Project::resolve_page`), writes the site
markdown with `SiteEmitter` (`tessera_emit::emit_page`), and renders it with
`tessera_emit::render_site_html`, which implements the site-render contract
that the Astro plugin implements; both pass the fixtures in `tests/render/`,
so heading ids and image attributes are the site's. No capability is
advertised: a client that wants it sends the request.

**Params**

```jsonc
{
  "textDocument": { "uri": "file:///…/docs/install.md" },
  "build": "cloud"        // optional: a build name; the default is `[editor] build`
}
```

**Result.** Always an object; `page` is `null` when there is nothing to show,
and `problems` says why. Field names are camelCase.

| Field | Meaning |
|---|---|
| `build` | The build the answer is for. |
| `builds` | Every build of the content model, in order: `{ name, editor, description }`. `editor` marks `[editor] build`, the picker's default. |
| `projectRoot`, `contentRoot` | Absolute paths. |
| `assetRoots` | Directories outside the content root that the page's assets are in and the preview may read: the directory of each asset that is in the project but not in the content root. The client may serve the content root and these, and nothing wider. |
| `documentVersion` | The version of the open document the answer is from, or `null` when the file isn't open. A client that sent version *n* and gets an older one has raced its own change notification and asks again. |
| `problems` | `{ severity: "error" \| "warning" \| "info", message }`: no project, an unknown build, a file that isn't a page (a fragment names the pages that include it), a page the build drops, an emit error, an asset the preview can't show. |
| `page.path`, `page.route` | The page's content path and its route on the site. |
| `page.title` | The page's title, phrases substituted. |
| `page.frontmatter` | What the site output writes as frontmatter, as JSON. `available` is the list of targets a layout hands to `<ascribe-availability>`. |
| `page.html` | The page's content as HTML, without frontmatter and without a layout. |
| `page.assets` | Each asset the page uses: `{ reference, path, kind, servable }`. `reference` is what the HTML writes, before any `#fragment`: an image's `src` is relative to the page (`./_fragments/a.png`), a link target's `href` is the site URL. `path` is the absolute source file, **resolved from the file the reference is written in** (asset contract §7), so a fragment's image is the one beside the fragment. `servable` is `true` when the file is in the content root or in a directory of `assetRoots`; a file directly in the project root, in `node_modules` or `.git`, or in the output directory isn't served, and `problems` says so. References are percent-encoded as URLs are; compare them after normalizing (`packages/vscode/src/preview/refs.ts` does). |
| `page.links` | Each link to a page: `{ href, path, id }`, `href` as the HTML writes it, `path` the target file, `id` the heading it names. |
| `page.sections` | The headings written in the previewed file itself, in order: `{ id, line }`, `line` from 0, for following the cursor. |

Every problem is in `problems`, not in a JSON-RPC error: a malformed request
(parameters that don't parse) is the only error, `InvalidParams`. The request runs on the server's main loop, so it must not grow with the
project: it emits one page, indexes only the files a position is asked for
(`EmitContext` builds line indexes lazily), and finds pages that share a route
once per set of pages (a cache keyed by the model revision and a hash of the
page paths), not per request. See the table under Performance.

## Capabilities

Advertised: incremental text document sync (open/close, no save), semantic
tokens (full and range), the position encoding, and, from phase 16, completion
(triggered by `@ { ( # / = , |` and a space), hover, definition, document
links, CodeLens, inlay hints, code actions, document formatting, rename, and one
command (`ascribe.openFile`, below). Workspace file-rename handling is advertised
for files. Registered dynamically after `initialized`, when the client allows it:
`workspace/didChangeWatchedFiles`.
Diagnostics are pushed (`textDocument/publishDiagnostics`); the server doesn't
advertise pull diagnostics.

Code actions carry the checker's existing diagnostic fixes (including the
router's reverse route suggestion) and the Ascribe-specific repairs. Rename and
file-move edits use the current project snapshot, including open buffers, its
source index, and reverse references. Formatting returns only the minimal edits
from `tessera-fmt`; the VS Code client applies those edits on save when
`ascribe.formatOnSave` is enabled, and requests file-move edits before renaming.

## How it works

- **The project** is a `tessera_resolve::IncrementalProject` built from the
  `ascribe.toml` in a workspace folder (the nearest one at or above the
  folder, or the first one found below it). Ids follow phase 13: source files
  have ids from 1, `ascribe.toml` is 0, an id names a path and is never reused.
- **Changes** reach it as `Change`s from three sources: open documents
  (`didOpen`, `didChange`, `didClose`; an open document's text wins over the
  file on disk, and closing one reverts to the disk), the file watcher (files
  and directories created, changed, deleted, or moved by anything else, sources
  and assets alike), and `ascribe.toml` (open or on disk).
- **The model.** An `ascribe.toml` that loads is applied as `Change::Model`
  (phase 13's tiers decide what is re-parsed, re-indexed, or re-resolved). One
  that doesn't load has its problems published on `ascribe.toml`, and the
  project keeps the last model that did. A change to the content root or
  output directory can't be applied in place (`ApplyError::LayoutChanged`),
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

## Navigation

Every request is answered from the snapshot the project has when the request
is handled (taken under the lock, computed without it), so an answer never
comes from a stale project: a request that follows an edit sees the edit. A
document that isn't a source file of the project (an `untitled:` buffer, a file
outside the content root, `ascribe.toml`) gets `null`. All positions go through
the negotiated encoding and `LineIndex`.

**One implementation of the reference rules.** What a link, an image, or an
include names, and whether it's there, comes from the source index
(`Project::resolutions`, `FileIndex::includes`), which calls
`tessera_resolve::references`, the code `ascribe check` uses. The features
below only *read* those results; where a request needs to resolve text that
isn't in the index yet (a destination being typed), they call
`references::reference_target` and `references::include_target`. Availability
text and labels come from `tessera_emit::labels`, the code the emitter builds
`<ascribe-availability>` from (element contract §0 and §4).

| Module | Feature |
|---|---|
| `complete.rs` | Completion |
| `hover.rs` | Hover |
| `definition.rs` | Go to definition |
| `links.rs` | Document links, CodeLens, inlay hints, and the `ascribe.openFile` command |
| `nav.rs` | What they share: the request's state (`Ctx`), what is under the cursor (`hit_at`), previews, relative paths |

**Completion** is decided from the line up to the cursor (a file being typed
rarely parses into the construct the author is in the middle of), and, for what
each context offers, from the content model and the index:

| Cursor | Offers |
|---|---|
| `@` at the start of a line (also after `>` and list markers) | Built-in directives and project widgets, with their descriptions, and `end` |
| In `{…}` after a directive's name | Its schema's attribute keys (not those already written); after `=`, the allowed values: an enumeration, `true`/`false`, note types, or a dimension's values with their display labels; after `\|`, the members not yet chosen. `@variant`'s keys are the model's dimensions |
| In `{…}` after an image | `[images.attributes]`'s keys |
| `@available:` and frontmatter `available:` | At the start: targets (values with labels), dimension names, feature keys (only as the whole spec). After a target: lifecycle states. In a history `(…)`: states |
| `{` in prose, a title, a heading, a text primary, or a `phrases=true` fence | Declared phrase keys, each showing its value |
| `@include:` | Source files, relative to the file (or from the content root after a `/`); after `#`, the ids of the file named |
| A link destination, `](…`, or a reference definition | Pages and headings by **title**: inserts the relative path and the source id, shows the path as the detail |

Nothing is offered inside inline code or a code fence. A search is cut at 100
items and marked incomplete, so the client asks again as the author types.

**Hover**: a link or image (its project-relative path, the title, a
plain-text preview of the first paragraph), an include (the same; a fragment
says so), a phrase (its value), an availability spec or feature key (SPEC
§9.4's text), and a directive or attribute key (its schema).

**Go to definition**: links and includes to the file or heading;
`@id`'s primary to its heading; phrases and feature keys to their entries in
`ascribe.toml`.

**Document links** make every link, image, and include destination
clickable, with `#L<line>` for a heading. **CodeLens** puts
`Includes <file> › <heading>` above each `@include`. Its command,
`ascribe.openFile`, is the server's own (`workspace/executeCommand`, which the
language client wires from the capability): it answers by sending
`window/showDocument`, so **no client code is needed**. **Inlay hints**
show the resolved title inside the `[` of an empty-text link.

## Performance

`cargo bench -p tessera-lsp --bench keystroke` types into a page, and into a
fragment that pages include, in a project of each size (every page has an
include, three links, an image, and headings) and times the wait from
`didChange` to the `publishDiagnostics` for that version. Release build, one
run on a 4-core 2.1 GHz Xeon container (a developer laptop is faster):

| Pages | Load and first diagnostics | Page keystroke, median (p95) | Fragment keystroke, median (p95), pages including it |
|---|---|---|---|
| 20 | 12 ms | 0.9 ms (1.2) | 0.8 ms (1.1), 0 |
| 100 | 36 ms | 1.0 ms (1.4) | 0.8 ms (1.1), 1 |
| 300 | 78 ms | 1.0 ms (1.4) | 1.5 ms (1.9), 3 |
| 1,000 | 286 ms | 2.0 ms (2.5) | 3.8 ms (4.7), 10 |
| 3,000 | 825 ms | 3.0 ms (3.6) | 9.0 ms (11.8), 30 |

The same benchmark then sends `ascribe/preview` for the page after each
keystroke, as the editor does after its debounce (release build, same machine):

| Pages | Change to preview answer, median (p95) | Change to diagnostics with the preview open, median (p95) |
|---|---|---|
| 20 | 0.5 ms (0.8) | 0.7 ms (1.0) |
| 100 | 0.8 ms (1.0) | 0.7 ms (1.0) |
| 300 | 0.6 ms (0.7) | 0.9 ms (1.0) |
| 1,000 | 0.9 ms (1.1) | 1.4 ms (1.7) |
| 3,000 | 1.1 ms (1.7) | 3.0 ms (5.8) |

The preview's cost doesn't grow with the project, and the diagnostics numbers
are what they are without it (3.1 ms at 3,000 pages). A first version listed
every page, tested whether the build drops it, and slugged every path for the
route check, and took 9.6 ms at 3,000 pages (and slowed diagnostics to 4.0 ms);
it also built a line index of every file for each request.

Both are far under the 50 ms target at 3,000 pages. What is left that grows
with the project is small: building the checked project from the snapshot's
texts (about 1 ms per 1,000 files), and the per-file lookups.

How: the file-level checks run for the files of a round only
(`tessera_check::check_file`). The page-level checks use
`PageChecker::with_index`, over the snapshot's own index, and
`PageChecker::check_resolved`, for the pages that can have a diagnostic located
in a file of the round (the file, when it's a page, and the pages that include
it); their resolved forms come from a `ResolvedCache` that forgets what each
update's `Affected` lists. A round also covers what its files include and
included at the previous round, because a page that starts or stops including a
fragment changes the diagnostics located in that fragment (an `include-cycle`),
and `Affected::recheck` doesn't list the fragment.

### Completion

`cargo bench -p tessera-lsp --bench completion` (release build) types the text
of each completion context into a page of the same generated projects
(`benches/synthetic`, shared with `keystroke`) and times `didChange` to the
completion response, with the server checking the edit in the background as it
would in an editor. 100 requests per context, median / 95th percentile; one run
on a 4-core 2.1 GHz Xeon container:

| Context | 20 pages | 300 | 1,000 | 3,000 pages |
|---|---|---|---|---|
| directive name | 0.11 / 0.15 ms | 0.12 / 0.22 ms | 0.12 / 0.19 ms | 0.12 / 0.21 ms |
| attribute values | 0.08 / 0.11 ms | 0.06 / 0.13 ms | 0.07 / 0.28 ms | 0.08 / 0.34 ms |
| phrase | 0.06 / 0.10 ms | 0.06 / 0.14 ms | 0.06 / 0.19 ms | 0.06 / 0.24 ms |
| include path | 0.35 / 0.59 ms | 0.67 / 0.87 ms | 1.4 / 2.2 ms | 3.6 / 4.7 ms |
| include id | 0.22 / 0.43 ms | 0.58 / 0.81 ms | 0.60 / 1.1 ms | 0.79 / 2.7 ms |
| link, nothing typed | 0.20 / 0.30 ms | 0.70 / 0.86 ms | 1.1 / 1.3 ms | 1.7 / 2.0 ms |
| link, page title | 0.36 / 0.50 ms | 0.90 / 1.8 ms | 2.9 / 4.7 ms | 5.9 / 8.1 ms |
| link, heading | 0.23 / 0.30 ms | 1.1 / 1.2 ms | 1.6 / 2.2 ms | 2.8 / 3.2 ms |

Every context is far under the 50 ms target at 3,000 pages (the worst 95th
percentile is 8 ms). Link and include completion scan the snapshot's pages and
headings (no cache, so it can't be stale), rank the matches, and cut the list at
100; the cost grows linearly with the project and the ranking is a sort of the
matches.

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
- `crates/tessera-cli/tests/lsp_parity.rs` starts the real `ascribe lsp` binary
  over stdio and compares its published diagnostics with
  `ascribe check --build <name> --format json`, for every build of
  `examples/quill` and fixture projects with known problems.
- `tests/navigation.rs` scripts every phase 16 feature over a copy of
  `examples/quill` (with a features registry added): each completion context,
  hover, definition, document links, CodeLens and its command, inlay hints,
  answers after an edit or a deletion, the negotiated encodings, files outside
  the project, and a request of every kind at every position of several files
  (including multi-byte text) that must never fail.
- `benches/keystroke.rs` records the time from an edit to its diagnostics;
  `benches/completion.rs` the time from an edit to its completion.
