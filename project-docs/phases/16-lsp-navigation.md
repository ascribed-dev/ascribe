# Phase 16: Language server navigation

**Track:** Editor · **Start after:** 15, 21 · **Parallel with:** 22, 25 · **Unblocks:** 24

## Goal

Make authoring fast: completion, hover, go to definition, document links, CodeLens, and inlay hints, all backed by the project index. This phase starts after the Astro end-to-end slice (phase 21) passes, so the full editor experience builds on a pipeline proven end to end.

## Read first

- [SPEC.md](../../SPEC.md): §10, plus §4 and §5 for what each feature describes.
- [PLAN.md](../PLAN.md): the language-server feature table.
- Handoff notes from phases 11, 12, and 15.

## Deliverables

- Feature modules in `crates/tessera-lsp`, and the matching capabilities advertised at initialization.

## Tasks

1. **Completion.** Context-aware, from the cursor's position in the syntax tree:
   - `@` at line start: built-in directives and project widgets, with a short description of each.
   - Inside an attribute block: the directive's attribute keys; after `=`, allowed values, including dimension values with their display labels.
   - In an `@available` primary or `available:` frontmatter: targets, dimension names, lifecycle states, and feature keys.
   - After `{` in prose: declared phrase keys, showing each value.
   - In an `@include` primary: files and source ids.
   - In a link destination: pages and headings searched by **title**, inserting the file path and source id. Show the path in the item's details, which disambiguates identical titles.
2. **Hover.**
   - Links and includes: the full target path and a short preview of the target (its title and first paragraph).
   - Phrases: the value.
   - `@available` and feature keys: the resolved availability in words.
   - Directives: what the directive does, from its schema.
3. **Go to definition.** Links and includes go to the target file or heading; `@id` references go to the heading; phrases and feature keys go to their entry in `ascribe.toml`.
4. **Document links and CodeLens.** Document links make every link and include destination clickable. A CodeLens above each `@include` names the target file and opens it. SPEC §10 allows a CodeLens "or equivalent"; document links are the equivalent for links in prose, where a CodeLens on every link would be noise.
5. **Inlay hints.** For empty-text links (`[](keys.md#rotate-keys)`), show the resolved title.
6. **Tests.** Scripted LSP tests for each feature against `examples/quill`.

## Acceptance criteria

- [ ] Each feature above has a scripted test that passes.
- [ ] Link completion finds a heading by title and inserts the correct relative path and source id from the current file.
- [ ] Completion responses arrive within 50 ms on a 3,000-page project (phase 26's synthetic project if it exists; otherwise a generated one).

## Out of scope

- Code actions, rename, and formatting (phase 24).
- The preview (phase 25).

## Handoff notes

### What was built

All in `crates/tessera-lsp` (the README's [Navigation](../../crates/tessera-lsp/README.md#navigation) section is the reference):

- `complete.rs` (completion), `hover.rs`, `definition.rs`, `links.rs` (document links, CodeLens, inlay hints, and the `ascribe.openFile` command), and `nav.rs` (what they share). Registered in `server.rs`: one `navigation` helper turns a request into a `Ctx` under the lock and computes without it; `Core::nav_target` (in `core.rs`) builds the `Ctx` from the current snapshot.
- Capabilities advertised: `completionProvider` (trigger characters `@ { ( # / = , |` and a space), `hoverProvider`, `definitionProvider`, `documentLinkProvider`, `codeLensProvider`, `inlayHintProvider`, `executeCommandProvider` (`ascribe.openFile`).
- `tests/navigation.rs` (24 scripted tests over a copy of `examples/quill` with a features registry), unit tests in each module, and phase 15's capabilities test updated.
- `benches/completion.rs`, and `benches/synthetic/mod.rs`: the project generator and the bench client moved out of `keystroke.rs` so both benchmarks share them (`keystroke.rs` behaves as before).
- Q161 to Q166 in `questions.md`, each implemented as proposed and marked `SPEC-QUESTION`.

### Interfaces phase 24 builds on

- **`nav::Ctx`** is what a request works from: the `Snapshot`, the file's content path, the model and its text, the config path, the content directory, and the encoding. `Core::nav_target(&uri)` builds it (under the lock; compute after releasing it), and `server::navigation` is the request wrapper: add a method to `handle_request`, a closure that reads the params and returns `(uri, |ctx| answer)`, and the capability in `serve`. Code actions, rename, and the formatter's requests fit it as they are.
- **`nav::hit_at(file, offset) -> Option<Hit>`** finds what is under a cursor: a phrase, a link or image (with its index into `FileIndex::references`, parallel to `Project::resolutions`), an include's path, an availability primary, the frontmatter's `available`, or a directive's name or attribute key. The smallest span wins, so a phrase inside a link's text is the phrase. Rename of a phrase key, an id, or a file starts from it (and from `Project::links_to_id` / `includers`).
- **`nav::Lines`** converts spans of any file of the snapshot to LSP ranges with one `LineIndex` per file per request. **`nav::relative_path(target, from)`** is the path as written in a link or include from a file (no `./`), and **`nav::encode_destination`** percent-encodes it: quick fixes that write paths (route to file path) and the refactorings that rewrite links use them, so they can't disagree with completion.
- **`definition::resolution_location` / `source_location`** give the `Location` a link or include names.
- **Reading the line.** Completion decides its context from the text of the line up to the cursor (`complete.rs`: `strip_container`, `scan_head`, `link_destination`, `is_image_before`), not from the tree, because a file mid-edit rarely parses into what's being typed. A code action that needs the same (a misspelled directive name) can reuse them.
- **`ascribe.openFile`** is the server's own command (`workspace/executeCommand` → `window/showDocument`), so a CodeLens needs no client code. Any other server-side command can follow the same pattern in `execute_command`.
- **Labels.** `tessera-lsp` now depends on `tessera-emit` for `labels::availability_display` and `plain_text` (element contract §0 and §4), rather than a copy.

### Decisions

- **Search on the server, cut at 100, `isIncomplete`** (Q161): the full list at 3,000 pages is tens of thousands of items, and ranking is what makes "search by title" work. A title-like query may contain spaces; a destination with a `"` gets none.
- **Links offer pages and their own headings, never fragments** (a fragment can't be linked to; a fragment's own `#id` links are offered inside that fragment). `@include` offers every source file.
- **Open-file goes through the server** (Q164), because no extension command exists and `vscode.open` can't be called from JSON. It needs the client's `window.showDocument` (VS Code has it); document links cover every include without it.
- **Hover paths are project-relative** (`docs/keys.md#rotate-keys`) (Q162).
- **No new checks or resolution**: nothing in `tessera-resolve`, `tessera-check`, or `tessera-emit` changed.

### Changes outside `crates/tessera-lsp/src`

- `crates/tessera-lsp/Cargo.toml`: the `tessera-emit` dependency and the `completion` bench.
- `crates/tessera-lsp/tests/support/mod.rs`: the scripted client advertises `window.showDocument`.
- `crates/tessera-lsp/tests/scenarios.rs`: the advertised-capabilities list.
- `crates/tessera-lsp/benches/keystroke.rs`: uses the shared generator (`benches/synthetic`).
- `project-docs/questions.md` (Q161 to Q166) and the crate README.
- No change to `tessera-resolve`, `tessera-check`, `tessera-emit`, `packages/`, `examples/`, or `tests/corpora`.

### Benchmark: completion at 3,000 pages

`cargo bench -p tessera-lsp --bench completion` (release build, 4-core 2.1 GHz Xeon container, the keystroke benchmark's generated project, 100 requests per context, `didChange` to the response), median / 95th percentile:

| Context | 3,000 pages |
|---|---|
| directive name | 0.12 / 0.21 ms |
| attribute values | 0.08 / 0.34 ms |
| phrase | 0.06 / 0.24 ms |
| include path | 3.6 / 4.7 ms |
| include id | 0.79 / 2.7 ms |
| link, nothing typed | 1.7 / 2.0 ms |
| link, page title | 5.9 / 8.1 ms |
| link, heading | 2.8 / 3.2 ms |

The target is 50 ms; the worst 95th percentile is 8 ms. The README has every size from 20 to 3,000 pages. Phase 26's shared synthetic project hadn't merged (`main` had no such thing when this was written); the benchmark uses the keystroke generator, and can switch when it lands.

### Acceptance criteria

See the pull request for the status and evidence of each.

### Left open

- **Q161 to Q166** (each implemented as proposed).
- **Reference-form links** (`[text][label]`) have no document link, because a definition's destination has no span in the tree (Q165). Hover, definition, and hints work for them.
- **Asset completion** for image sources isn't offered (Q161).
- **Hints and previews use the source, not a build**: a title with a phrase shows the phrase's value, but build modes don't apply (Q166).
- **Definition into `ascribe.toml`** finds `[phrases]` keys and `[features.<key>]` tables written that way only (Q163).
- **Attribute values in `@variant`** aren't hovered (a dimension key is).
- **CodeLens refresh**: the client asks again after each change; the server doesn't send `workspace/codeLens/refresh` when a *target* changes (an included file renamed shows the old title until the next request for this file).
- **A completion isn't cached**: every request scans the snapshot. Linear in the project (8 ms at 3,000 pages); an index sorted by title would make it constant if a much larger project needs it.
