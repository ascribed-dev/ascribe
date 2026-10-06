# Phase 2: Page edits

Part of [Editor UI](README.md). Requires phase 1. Rust only (`crates/ascribe-lsp`, and `crates/ascribe-fmt` if a helper belongs there).

## Goal

`ascribe/edit`: given an action, its arguments, and a page position or selection, the server returns the `WorkspaceEdit` that performs it, written in canonical form. Every page action in phases 3 and 4 is one of these operations. Content-model actions (which edit `ascribe.toml`) are phase 5.

## Context

- Phase 1's `src/context.rs`: the node kinds and ranges an operation acts on. Operations should find their target the same way, so `ascribe/context` and `ascribe/edit` never disagree about what's at a position.
- `crates/ascribe-fmt`: `format`, `format_parsed`, `format_source`, and `options_from_model`. Canonical form for directives, attributes, and their order (SPEC §8.3).
- `crates/ascribe-lsp/src/code_action.rs`: how existing quick fixes build `TextEdit`s and `WorkspaceEdit`s.
- `crates/ascribe-lsp/src/refactor.rs`: multi-file edits (rename), for operations that touch more than one file.
- `docs/directives.md`: the syntax each operation writes (`@note`, `@steps`, `@details`, `@variant` groups, `@available`, `@include`, images and their attributes, project widgets, `@id`), including how directives are indented inside list items.

## Design

Params:

```jsonc
{
  "textDocument": { "uri": "…" },
  "range": { … },              // the cursor or selection the action was invoked on
  "action": "wrapNote",
  "args": { "type": "tip" },   // what the wizard collected; each operation documents its own
  "version": 12                // the document version the client saw
}
```

Result: `{ "edit": WorkspaceEdit }`, or `{ "error": "…" }` with a plain-language message when the action no longer applies (the document changed under it, or the arguments are invalid). A stale `version` is an error, not a guess.

Operations, by where they apply:

| Operation | Applies to | Args | Writes |
|---|---|---|---|
| `wrapNote` | One paragraph, or whole blocks | `type` | `@note {type=…}` before one paragraph, or a container (`@note {type=…}:` … `@end`) around several blocks; no attributes for type `note` |
| `setNoteType` | A note | `type` | The note's `type` attribute, or none for `note` |
| `unwrapNote` | A note | — | The note's content, without the directive |
| `noteToDetails` | A note | `title` | `.Title` and `@details`, keeping the content |
| `wrapDetails` | One block, or whole blocks | `title` | `.Title` and `@details` (or the container form) |
| `makeSteps` | An ordered list | — | `@steps` above it |
| `addHeadingId` | A heading | `id` (defaults to its current slug) | `@id: …` after the heading |
| `insertNote` | An insertable line | `type`, `text?` | A note with placeholder text selected for typing |
| `insertSteps` | An insertable line | `count` | `@steps` and a numbered list |
| `insertVariantGroup` | An insertable line | `dimension`, `values[]` | One `@variant {dim=value}:` arm per value, and `@end` |
| `addVariantArm` | A variant group | `value` | A new arm, in the dimension's declared value order |
| `removeVariantArm` | An arm | — | The group without it; a group left with one arm stays a group |
| `insertDetails` | An insertable line | `title` | `.Title`, `@details:`, `@end` |
| `insertInclude` | An insertable line | `path` | `@include: …` |
| `insertImage` | An insertable line | `path`, `alt`, `attributes?` | `![alt](path){…}` |
| `insertWidget` | An insertable line | `name`, `primary?`, `attributes` | The widget in its declared form, attributes in declared order |
| `markAvailable` | A heading (its section) or a block | `spec` (a spec or a feature key) | `@available: …` at the top of the section, or touching the block |
| `linkSelection` | Inline prose | `destination` | `[selection](destination)` |
| `setLinkTarget` | A link | `destination` | The link's destination |
| `useTargetTitle` | A link | — | Empty link text (`[](…)`), so the target's title fills it |
| `setImageWidth` | An image | `width` | The image's `width` attribute, if the model declares it |
| `setImageAlt` | An image | `alt` | The image's alt text |

Rules for every operation:

- **Canonical.** After applying the edit, `ascribe_fmt::format` on the result makes no change in the edited range. Build the new text with the formatter's own functions where they exist, rather than by hand.
- **Lists.** Inside a list item, written directives are indented to the item's content (directives.md, "Lists and block quotes").
- **Minimal.** The edit touches only what the action changes, so a reviewer's diff shows only that.
- **Validated against the model.** `type`, `dimension`, `values`, widget names and attributes, and `spec` are checked against the content model; an invalid argument returns an error naming the valid choices, never an edit that creates a diagnostic.
- **Placeholder text** in inserted blocks (for example "Write the note here.") is returned with a snippet tab stop or a selection range, so the client can leave it selected for typing. Use LSP's `SnippetTextEdit` only if the client supports it; otherwise return the range to select alongside the edit.

## Tasks

1. Add `src/edit.rs` with `ascribe/edit` and the operations above, wired into `server.rs`.
2. A test for every operation: the edit applied to a page produces the expected text, and formatting the result changes nothing. Add a property-style test that runs every applicable operation at every position of the `examples/quill` and `examples/monorepo` pages (with valid default arguments) and checks the result is canonical and has no new diagnostics.
3. Errors: stale version, an action that doesn't apply at the position, and invalid arguments, each with a test.
4. Lists: wrapping a list item's paragraph in a note, and inserting a variant group inside a step, indent correctly.
5. Document `ascribe/edit` in `crates/ascribe-lsp/README.md`: the params, the result, the operation table with each one's arguments, and the guarantee that results are canonical.

## Out of scope

Edits to `ascribe.toml` (phase 5); any extension change; deciding which operations appear where (phase 3).

## Acceptance criteria

- Every operation in the table works and is tested, including inside list items.
- The canonical-form test passes on both example projects.
- No operation can return an edit that introduces a diagnostic, given valid arguments.

## Verify

```sh
cargo test -p ascribe-lsp --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
```

## Commits

One per group if that's clearer: "Edit notes, details, and steps", "Insert and edit variant groups", "Edit links, images, includes, widgets, and availability", or one "Make page edits for actions: ascribe/edit".
