# Phase 5: Content-model actions

Part of [Editor UI](README.md). Requires phases 2 to 4. Server and extension.

## Goal

Actions that change the content model, reachable like every other action (palette, context menu, lightbulb where natural, and the actions bar):

- **Make the selection a phrase:** declare it in `[phrases]` and replace the selection with `{key}`, optionally its other occurrences too.
- **Add the selection to the glossary:** declare a term in `[glossary.terms]`.
- **Promote a feature:** change a feature's availability in `[features]` ("Promote audit-log to GA").
- **Rename a phrase key** or **a dimension value**, everywhere it's used. Renaming a phrase key already works with F2; this phase makes it an action, and adds dimension values to the same rename (README decision 9).

`ascribe.toml` keeps its comments, blank lines, and order (README decision 6).

## Context

- Phase 2's `src/edit.rs` (`ascribe/edit`) and phase 3's registry, runner, and `ProjectServer.requestEdit`; these become more operations and actions.
- `crates/ascribe-model`: how `ascribe.toml` is loaded and validated (phrase key rules, the one-role rule for names, availability spec syntax, feature rules). Reuse its validation for arguments.
- `crates/ascribe-lsp/src/refactor.rs`: the existing `textDocument/rename`. `rename` handles F2 on a `{key}`, an `@id`, or a heading in a page; `rename_model_key` handles F2 on a phrase's key in `ascribe.toml`; `rename_phrase` edits the declaration and every use the snapshot's index records (`file.phrases`). The server advertises no `prepareRename` (`prepare_provider: Some(false)` in `server.rs`).
- `crates/ascribe-lsp/src/definition.rs`: `find_entry`, and phase 1's declaration ranges in `ascribe/targets`.
- `docs/content/reference/content-model.md`: `[phrases]` (§11), `[glossary]` (§12), `[features]` (§9), `[dimensions]` (§6), `[builds]` (§16).
- `crates/ascribe-lsp/src/core.rs`: how the server reloads the model when `ascribe.toml` changes (an open buffer wins over the disk).

## Design

### Editing `ascribe.toml`

Add `toml_edit` (workspace dependency) and make every change with it, as a `TextEdit` over the model file in the same `WorkspaceEdit` as any page edits. If `ascribe.toml` is open with unsaved changes, edit the buffer's text, not the disk. Insert new keys at the end of their table (`[phrases]`, `[glossary.terms.<id>]`), creating the table if it doesn't exist, after its last entry, with the file's existing style (quoting, spacing).

`toml_edit` writes a whole document back, and an edit that replaces the whole file would lose the cursor, mark every line changed, and break the "minimal" rule. So it's used to find and to render, never to rewrite:

- **Find the place** from the parsed document's spans: the end of a table's last entry for an insertion, or the span of the value being changed.
- **Render the new text** for that place alone (one `key = "value"` line, one table, one value) with `toml_edit`, so quoting and escaping are right.
- **Return one `TextEdit`** that inserts at that position or replaces that span.
- **Check it.** Apply the edit to the text, parse the result with `toml_edit`, and compare it with the document changed through `toml_edit`'s own API. If they differ, return an error; don't fall back to rewriting the file.

### Operations

| Operation | Args | Edits |
|---|---|---|
| `makePhrase` | `key`, `everywhere: bool` | `[phrases] key = "<selected text>"`; the selection becomes `{key}`; with `everywhere`, every whole-word occurrence of the same text in the project's prose (not code, links' destinations, or headings with a fixed id unless the id is explicit) |
| `addGlossaryTerm` | `id`, `term`, `aliases[]`, `definition`, `link?` | `[glossary.terms.<id>]` with those keys |
| `promoteFeature` | `key`, `spec` | The feature's `available` |

Rules:

- **Validate first.** A key or value that breaks the model's rules (key syntax, the one-role rule, an existing key) returns an error naming the problem, before any edit.
- **No new diagnostics.** After the edit, the project has no diagnostics it didn't have before. Test this on both example projects.
- **Minimal.** No edit to `ascribe.toml` covers more than the entry it adds or the value it changes.

### Renames

Renames are `textDocument/rename`, not `ascribe/edit` operations.

- **Phrase keys.** The rename exists. Check that it reaches every kind of use: prose, link text and destinations, `phrases=true` code blocks and `@snippet`s, and frontmatter fields. Where the index (`file.phrases`) misses one, extend the index, with a test for each kind. Don't add a second search.
- **Dimension values.** Extend the same handlers. In a page, the rename starts from a value in a `@variant` attribute; in `ascribe.toml`, from the value in `[dimensions.<name>]`'s `values`. It edits the value there and in `labels` and `versionless`, and every use: `@variant` attributes, `variant` frontmatter, availability specs in pages, table rows, and frontmatter, `[features]` specs, and `[builds]` (`variants`, `filter`). A new name that breaks the model's rules, or that the dimension already has, returns no edit, as the phrase rename does.
- **`prepareRename`.** Add it, so F2 on anything that can't be renamed says so, and so the range it returns shows the writer what will be renamed. It answers for everything `rename` accepts today, with no change to what those renames do.
- **Preview from an action.** F2 keeps VS Code's own behavior, with its preview on Shift+Enter. A rename run as an action touches files the writer isn't looking at, so the extension asks first: it sends `textDocument/rename` through `ProjectServer.requestEdit`, and when the edit changes any file other than the active one, it marks every entry `needsConfirmation` and applies it with `workspace.applyEdit(edit, { isRefactoring: true })`. VS Code then shows its refactor preview, listing every change, before anything is written. Both APIs are in VS Code 1.138 and `vscode-languageclient` 10.1, the versions the extension requires.

### Actions

Add each to the registry with writer-facing titles ("Make this a phrase", "Add to the glossary", "Change a feature's availability", "Rename this phrase everywhere", "Rename a dimension value everywhere"), a **Content model** group in the actions bar, and wizards:

- `makePhrase`: a suggested key from the text (lowercase, hyphenated), editable, validated as you type; then "Replace other occurrences too?" with the count.
- `addGlossaryTerm`: term (the selection), aliases, definition, and an optional link (a heading picker from `ascribe/targets`).
- `promoteFeature`: pick a feature, then edit its spec, pre-filled with the current one.
- Renames: registry actions with `run`, not an `operation`. On a phrase (`{key}`) or a dimension value in an attribute, the action asks for the new name and renames at the cursor. From the palette anywhere else, it asks which phrase or value with a picker, then renames at that entry's declaration in `ascribe.toml`, whose range `ascribe/targets` gives.

## Tasks

1. `toml_edit` round-trip editing in the server, with tests that comments, blank lines, and order survive each kind of change.
   Test too that each change's `TextEdit` covers only its entry or value, and that a table the file doesn't have yet is created.
2. The three operations in `ascribe/edit`, with tests including the "no new diagnostics" check on `examples/quill` and `examples/monorepo`, and an unsaved `ascribe.toml` buffer.
3. Renames in `refactor.rs`: the phrase rename's missing use kinds, if any; dimension values, from a page and from `ascribe.toml`; and `prepareRename`. Scenario tests for every kind of use listed, the "no new diagnostics" check on both example projects, and that the renames F2 does today return the same edits as before.
4. The actions, wizards, and bar group in the extension, with unit and integration tests (rename preview shown; cancel leaves nothing changed; a rename from the palette's picker with the cursor nowhere near the key).
5. Document the operations and what rename now covers in `crates/ascribe-lsp/README.md`. A description for every new command in `docs.test.ts`'s `commands`, then bless the commands table. The actions in `docs/content/guides/editor.md` (its "Refactoring" section already describes F2), and a line in `CHANGELOG.md`.

## Out of scope

Removing model entries; editing types, widgets, or builds by form; anything across projects (a rename stays in the file's project).

## Acceptance criteria

- Each action works from every path, and `ascribe.toml` keeps its formatting and comments. An action that changes `ascribe.toml` and a page undoes in one step in each.
- Renames reach every kind of use listed, from F2 and from the actions, and leave no new diagnostics. An action's rename shows the preview when it changes another file.
- The repository has one implementation of each rename.

## Verify

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
pnpm --filter ascribe-vscode test
cargo build -p ascribe-cli && ASCRIBE_BIN=$PWD/target/debug/ascribe pnpm --filter ascribe-vscode test:integration
pnpm lint && pnpm format:check && cargo fmt --all --check
```

## Commits

1. "Edit ascribe.toml in place, keeping its comments"
2. "Make phrases and glossary terms, and promote features"
3. "Rename dimension values, and say what can be renamed"
4. "Offer content-model actions in the editor"
