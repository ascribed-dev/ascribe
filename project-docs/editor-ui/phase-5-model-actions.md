# Phase 5: Content-model actions

Part of [Editor UI](README.md). Requires phases 2 to 4. Server and extension.

## Goal

Actions that change the content model, reachable like every other action (palette, context menu, lightbulb where natural, and the actions bar):

- **Make the selection a phrase:** declare it in `[phrases]` and replace the selection with `{key}`, optionally its other occurrences too.
- **Add the selection to the glossary:** declare a term in `[glossary.terms]`.
- **Promote a feature:** change a feature's availability in `[features]` ("Promote audit-log to GA").
- **Rename a phrase key** or **a dimension value**, everywhere it's used.

`ascribe.toml` keeps its comments, blank lines, and order (README decision 6).

## Context

- Phase 2's `src/edit.rs` (`ascribe/edit`) and phase 3's registry; these become more operations and actions.
- `crates/ascribe-model`: how `ascribe.toml` is loaded and validated (phrase key rules, the one-role rule for names, availability spec syntax, feature rules). Reuse its validation for arguments.
- `crates/ascribe-lsp/src/refactor.rs`: the existing rename (pages and ids), for multi-file edits.
- `docs/content-model.md`: `[phrases]` (§11), `[glossary]` (§12), `[features]` (§9), `[dimensions]` (§6), `[builds]` (§16).
- `crates/ascribe-lsp/src/core.rs`: how the server reloads the model when `ascribe.toml` changes (an open buffer wins over the disk).

## Design

### Editing `ascribe.toml`

Add `toml_edit` (workspace dependency) and make every change through it, as a `TextEdit` over the model file in the same `WorkspaceEdit` as any page edits. If `ascribe.toml` is open with unsaved changes, edit the buffer's text, not the disk. Insert new keys at the end of their table (`[phrases]`, `[glossary.terms.<id>]`), creating the table if it doesn't exist, after its last entry, with the file's existing style (quoting, spacing).

### Operations

| Operation | Args | Edits |
|---|---|---|
| `makePhrase` | `key`, `everywhere: bool` | `[phrases] key = "<selected text>"`; the selection becomes `{key}`; with `everywhere`, every whole-word occurrence of the same text in the project's prose (not code, links' destinations, or headings with a fixed id unless the id is explicit) |
| `addGlossaryTerm` | `id`, `term`, `aliases[]`, `definition`, `link?` | `[glossary.terms.<id>]` with those keys |
| `promoteFeature` | `key`, `spec` | The feature's `available` |
| `renamePhraseKey` | `from`, `to` | The key in `[phrases]`, and every `{from}` in the project (prose, link text and destinations, `phrases=true` code blocks and frontmatter fields) |
| `renameDimensionValue` | `dimension`, `from`, `to` | The value in `[dimensions.<name>]` (and `labels`, `versionless`), and every use: `@variant` attributes, `variant` frontmatter, availability specs in pages and frontmatter, `[features]` specs, and `[builds]` (`variants`, `filter`) |

Rules:

- **Validate first.** A key or value that breaks the model's rules (key syntax, the one-role rule, an existing key) returns an error naming the problem, before any edit.
- **No new diagnostics.** After the edit, the project has no diagnostics it didn't have before. Test this on both example projects.
- **Preview for renames.** When a rename touches more than the current page, its edits carry an LSP change annotation with `needsConfirmation`, and the extension applies them with `workspace.applyEdit(edit, { isRefactoring: true })`, so VS Code shows its refactor preview listing every change before anything is written. Check both APIs against the current versions of `vscode-languageclient` and VS Code.

### Actions

Add each to the registry with writer-facing titles ("Make this a phrase", "Add to the glossary", "Change a feature's availability", "Rename this phrase everywhere", "Rename a dimension value everywhere"), a **Content model** group in the actions bar, and wizards:

- `makePhrase`: a suggested key from the text (lowercase, hyphenated), editable, validated as you type; then "Replace other occurrences too?" with the count.
- `addGlossaryTerm`: term (the selection), aliases, definition, and an optional link (a heading picker from `ascribe/targets`).
- `promoteFeature`: pick a feature, then edit its spec, pre-filled with the current one.
- Renames: offered on a phrase (`{key}`) or a dimension value in an attribute, and from the palette with a picker.

## Tasks

1. `toml_edit` round-trip editing in the server, with tests that comments, blank lines, and order survive each kind of change.
2. The five operations in `ascribe/edit`, with tests including the "no new diagnostics" check on `examples/quill` and `examples/monorepo`, and an unsaved `ascribe.toml` buffer.
3. The actions, wizards, and bar group in the extension, with unit and integration tests (rename preview shown; cancel leaves nothing changed).
4. Document the operations in `crates/ascribe-lsp/README.md`, the actions in `docs/editor.md`, and a line in `CHANGELOG.md`.

## Out of scope

Removing model entries; editing types, widgets, or builds by form; anything across projects (a rename stays in the file's project).

## Acceptance criteria

- Each action works from every path, and `ascribe.toml` keeps its formatting and comments.
- Renames reach every kind of use listed, with a preview, and leave no new diagnostics.

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
3. "Rename phrase keys and dimension values everywhere"
4. "Offer content-model actions in the editor"
