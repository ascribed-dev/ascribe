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
3. **Go to definition.** Links and includes go to the target file or heading; `@id` references go to the heading; phrases and feature keys go to their entry in `tessera.toml`.
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

_To be filled in by the implementing agent._
