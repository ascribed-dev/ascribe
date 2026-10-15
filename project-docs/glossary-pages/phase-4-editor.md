# Phase 4: The editor

Part of [Glossary pages](README.md). Needs phase 3.

## Goal

A term goes to its section, `@term` completes, a misplaced one has a fix, and renaming a heading keeps everything that names its term.

In VS Code, `F12` on "rollouts" in `getting-started.md` opens `glossary.md` at "Rollout". Typing `@te` under a heading on a glossary page offers `@term`, and inside its braces `match=` offers `first`, `every`, and `marked`. On a page that isn't in `pages`, `@term` is underlined, and **Add this page to the glossary** lists it.

## Context

- README decisions 1 to 6, and phase 1's decision 3.
- `crates/ascribe-lsp/src/`: `definition.rs`, which goes to `[glossary.terms.<id>]` in `ascribe.toml` today; `complete.rs`; `code_action.rs`; `references.rs`; `rename.rs`; `hover.rs`; and the document symbols the outline is built from.
- `crates/ascribe-cli/src/commands/rename.rs`.
- The quick fix **Add a stable @id**, for a term whose heading has none.

## Design

- **Go to definition** on a linked term, in a page, goes to its heading. On a term's id in an `@intended` or anywhere else that names it, the same.
- **Find all references** on a term's heading lists its occurrences that are linked, as `ascribe refs term:<id>` does, beside the links to the heading.
- **Completion** offers `@term` at the top of a section on a glossary page, and `@term`'s attributes and their values.
- **Hover** on a `@term` heading shows the definition for the editor's build and how many pages use the term.
- **Quick fixes:** `term-off-glossary-page`'s, which adds the page to `pages`; `glossary-definition-missing`'s, which puts a paragraph with the cursor in it under the heading; and, on a term with no `@id`, **Add a stable @id**, since the term's id is its heading's.
- **Rename.** Renaming a term's heading id, from the editor or `ascribe rename`, renames the term: every link to the heading changes, as now, and so does `[[intended]] term` if phase 1's decision 3 keeps it. Rewording the heading's text changes the term's text, and the editor says so when the heading has no `@id`, since its id changes too.
- **The outline.** A term's heading is a symbol of its own kind in the editor's outline, so a long glossary can be searched by term.

## Tasks

1. Go to definition and find all references, with tests of the server.
2. Completion and hover.
3. The quick fixes.
4. Rename, from the editor and the command, with tests.
5. The outline's symbols.
6. The editor's page, `changes/`, and the surface map.

## Out of scope

- A view of every term in the sidebar.
- Anything in the site.

## Acceptance criteria

- The actions in the goal work in the extension's tests.
- Renaming `rollout` to `rollouts-explained` with `F2` on the heading's `@id` changes the links to it, and the build's `data-ascribe-term` is the new id.
- Every quick fix leaves no diagnostic behind.

## Verify

```sh
cargo test -p ascribe-lsp --locked
pnpm --filter ascribe-vscode test
```

## Commits

1. "Go from a term to its heading, and find its uses"
2. "Complete @term, and fix one off a glossary page"
3. "Rename a term with its heading's id"
