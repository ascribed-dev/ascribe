# Phase 1: Context and targets

Part of [Editor UI](README.md). Rust only (`crates/ascribe-lsp`).

## Goal

Two read-only requests the extension builds every action on:

- **`ascribe/context`:** what is at a position or selection in a page, so the extension knows which actions apply.
- **`ascribe/targets`:** what an action can point at or use in the page's project (pages, headings, fragments, images, snippets, phrases, note types, dimensions, widgets, features, builds), so wizards can offer choices instead of asking for syntax.

## Context

- `crates/ascribe-lsp/src/nav.rs`: `Ctx` (a request's view of one file and its project), `hit_at` (what's at an offset: links, includes, phrases, ids), `directive_at`, `identifier_primary`, `relative_path`, `encode_destination`. Most of what `ascribe/context` needs is here or one step from it.
- `crates/ascribe-lsp/src/code_action.rs`: how a request turns an LSP range into offsets (`ctx.encoding.offset_lenient`), and finds the directive at a position.
- `crates/ascribe-lsp/src/preview.rs` and `preview_request` in `server.rs`: the pattern for a custom request (a module with `METHOD`, `Deserialize` params and `Serialize` result in camelCase, a handler that reads the snapshot).
- `crates/ascribe-lsp/src/complete.rs`: already lists pages, ids, phrases, note types, dimension values, and widget attributes for completion; reuse its sources rather than duplicating them.
- `crates/ascribe-syntax`: the block tree (`Block`, `DirectiveLine`, containers, groups, lists, headings) with spans.
- `crates/ascribe-model`: the content model (`ContentModel`: types, dimensions with labels, phrases, features, notes, glossary, widgets with attribute schemas, sources, builds). It keeps no source spans.
- `crates/ascribe-lsp/src/definition.rs`: `find_entry` finds an entry's span by searching `ascribe.toml`'s text. It covers phrases and features today.
- `crates/ascribe-resolve/src/fs.rs`: `FileSystem`, which every read and listing of a project's files goes through (the README's rule).
- `crates/ascribe-lsp/src/preview.rs` and `crates/ascribe-cli/src/shapes.rs`: how a result type derives `JsonSchema` and gets its schema, TypeScript type, and docs fragment (the README's rule).
- [The agents plan](README.md#the-agents-plan): the lists `ascribe/targets` returns are shared with `ascribe model` and `ascribe outline`.
- `crates/ascribe-lsp/README.md`: document both requests next to "The preview request".

## Design

### `ascribe/context`

Params: `{ textDocument, range }` (a selection; an empty range is a cursor).

Result, all positions as LSP ranges in the negotiated encoding:

```jsonc
{
  "project": { "root": "…", "editorBuild": "site" },
  // The innermost-first chain of what contains the start of the range.
  "at": [
    { "kind": "note", "range": …, "type": "tip", "form": "line" | "block" | "container" },
    { "kind": "listItem", "range": … },
    { "kind": "list", "range": …, "ordered": true, "steps": true },
    { "kind": "variantGroup", "range": …, "dimension": "pm", "arms": [{ "value": "npm", "range": … }], "arm": 0 },
    { "kind": "heading", "range": …, "level": 2, "id": "install-cli", "explicitId": true },
    { "kind": "section", "range": …, "headingId": "install-cli" }
  ],
  // What the selection is, when the range isn't empty.
  "selection": { "kind": "prose" | "blocks" | "mixed" | "code" | "other", "text": "…", "inline": true },
  // The token under the start of the range, if any.
  "token": { "kind": "link" | "image" | "include" | "phrase" | "directiveName" | "attribute", "range": …, … },
  // Whether the line at the cursor is blank, between blocks: where a block can be inserted.
  "insertable": true
}
```

- Node kinds to cover: heading, section, paragraph, list (with whether `@steps` applies), list item, note (type, form), details, steps, variant group (dimension, arms, which arm the cursor is in), availability line, include, snippet (its address), image (src, alt, attributes), link (destination, whether its text is empty), phrase, widget (name, attributes), code block, table, table row (whether it's the header row, and its `available` spec if it has one), frontmatter (the ranges and values of its `variant` and `available` keys, when present).
- `selection.kind`: `prose` when it's inside one paragraph or heading's text (`inline: true`) or spans whole prose blocks; `blocks` when it covers whole blocks of any kind; `code` inside code; `mixed` otherwise. Actions use this to decide what applies ("Wrap in a note" needs whole blocks or one paragraph; "Make it a phrase" needs inline prose).
- Keep the result small: no text beyond `selection.text`, and no project-wide data (that's `ascribe/targets`).

### `ascribe/targets`

Params: `{ textDocument, kinds: ["pages", "headings", "fragments", "images", "snippets", "phrases", "notes", "dimensions", "widgets", "features", "builds"] }`, so a wizard asks only for what it needs.

Result: one list per requested kind, each entry with what a picker shows and what an edit needs:

- `pages`: content path, title, type, and the link path from the requesting page (`relative_path` + `encode_destination`).
- `headings`: page, heading text, id, level, and the link destination (`page.md#id`).
- `fragments`: content path, the include path from the requesting page, and whether it starts with a heading.
- `images`: project-relative path of each image file under the content root, and the path from the requesting page. List them through `FileSystem`, never with `std::fs`.
- `snippets`: for each source in `[sources.<name>]`, the files its `include` and `ignore` take in, each with its regions and the address to write (`code:scripts/install.sh#install`). Files are listed through `FileSystem`, and regions are read by the code that resolves `@snippet`, so the list and the check agree. A source in another repository lists its copies under `sources/<name>/`; nothing reaches the network.
- `phrases`: key, value, and the range of its declaration in `ascribe.toml`. `notes`: type and label. `dimensions`: name, label, values with their labels and the range of each value's declaration, versionless values. `widgets`: name, description, forms, primary kind, attributes with types. `features`: key, name, availability. `builds`: name, and which one is the editor build.

Declaration ranges come from `find_entry`, extended to dimension values. Phase 5's renames start from them, and phase 7's inventory reuses them.

All of it comes from the current snapshot and model, so it includes unsaved edits and works with no files on disk changed.

## Tasks

1. Add `src/context.rs` with `ascribe/context`, and `src/targets.rs` with `ascribe/targets`, wired into `server.rs` like `ascribe/preview`. Return an empty result (not an error) for a document outside the project's sources.
2. Reuse `nav.rs` and `complete.rs` rather than walking the tree a second way. If a helper needs to move to be shared, move it.
3. Scenario tests in `crates/ascribe-lsp/tests/` (a new `context.rs` is fine), for each node kind and selection kind above, including:
   - a cursor inside a variant arm inside a list item inside steps (the full `at` chain);
   - a selection inside one paragraph (`prose`, inline), across two paragraphs (`blocks`), and across a paragraph and a code block (`mixed`);
   - a blank line between blocks (`insertable`) and a blank line inside a code block (not insertable);
   - unsaved edits: change the buffer, then ask;
   - `ascribe/targets` link paths from a page in a subfolder, and fragments' include paths;
   - UTF-16 positions on a line with non-ASCII text.
   - `images` and `snippets` listed with `crates/ascribe-resolve/tests/file_reads.rs` still passing, and a snippet file's regions.
4. Derive `JsonSchema` on both results behind the `json-schema` feature, add them to `SHAPES`, and bless. Phase 3 imports `ContextResult` and `TargetsResult` from `packages/vscode/src/shapes.ts`.
5. Document both requests in `crates/ascribe-lsp/README.md`: params, result, and that other clients can use them.

## Out of scope

Edits (phase 2) and any extension change other than the generated `shapes.ts`.

## Acceptance criteria

- Both requests answer correctly for every node kind and selection kind listed, with tests.
- A request for a page in a nested project goes to, and is answered by, that project's server only (the existing routing; add a test if one doesn't cover a custom request).
- `cargo test --workspace --locked` passes, including the shapes and file-reads tests, and the README documents both requests.

## Verify

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
```

## Commits

1. "Answer what's at a position: ascribe/context"
2. "List what actions can point at: ascribe/targets"
