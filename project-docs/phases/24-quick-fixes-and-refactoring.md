# Phase 24: Quick fixes and refactoring

**Track:** Editor · **Start after:** 14, 16, 23 · **Parallel with:** 22, 25 · **Unblocks:** 27

## Goal

Let the editor fix problems and keep references intact: code actions for common diagnostics, rename refactorings that update links and includes across the project, and formatting on save.

## Read first

- [SPEC.md](../../SPEC.md): §5.5 (source ids) and §10.
- [PLAN.md](../PLAN.md): the language-server feature table (quick fixes, refactoring, formatting).
- Handoff notes from phases 10 (fixes attached to diagnostics), 13, 16, and 23.

## Deliverables

- Code-action, rename, and formatting modules in `crates/tessera-lsp`.
- Updates to `packages/vscode` for file-rename handling and the format-on-save setting.

## Tasks

1. **Quick fixes.** Surface the fixes attached to diagnostics as code actions, and add these:
   - "Did you mean `@note {type=warning}:`?" for misspelled directives.
   - Add a missing trailing colon to a container, or remove a stray one.
   - Remove the blank line between a following-block directive and its block.
   - Quote an attribute value that needs quotes.
   - Convert a route-style link to a file path, using the router in reverse.
   - Declare an undeclared phrase key in `tessera.toml`, or escape it as `\{`.
2. **File rename and move.** Handle `workspace/willRenameFiles`: update every link, include, and asset reference that points at the moved file, including relative paths inside the moved file itself.
3. **Id rename.** Renaming an `@id`, or a heading whose source id comes from its slug, updates every link and include that names it. Offer to add an `@id` when renaming a heading would change its slug.
4. **Phrase key rename.** Renaming a phrase key updates `tessera.toml` and every use.
5. **Formatting.** Provide document formatting through `tessera-fmt`, and format on save when `tessera.formatOnSave` is enabled.
6. **Tests.** Scripted LSP tests for each action and refactoring against copies of `examples/quill`, asserting the resulting workspace edits and that the project still checks clean afterward.

## Acceptance criteria

- [ ] Each quick fix above has a test that applies it and shows the diagnostic gone.
- [ ] Moving `keys.md` into a subdirectory updates every link to it, and the project still checks clean.
- [ ] Renaming an `@id` updates every link and include that names it.
- [ ] Renaming a phrase key updates `tessera.toml` and every use.
- [ ] Format on save applies `tessera-fmt`'s edits and nothing else.

## Out of scope

- Refactorings the spec doesn't mention, such as extracting a fragment. Record ideas in `future-things.md`.

## Handoff notes

_To be filled in by the implementing agent._
