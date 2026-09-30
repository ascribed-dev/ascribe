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
   - Declare an undeclared phrase key in `ascribe.toml`, or escape it as `\{`.
2. **File rename and move.** Handle `workspace/willRenameFiles`: update every link, include, and asset reference that points at the moved file, including relative paths inside the moved file itself.
3. **Id rename.** Renaming an `@id`, or a heading whose source id comes from its slug, updates every link and include that names it. Offer to add an `@id` when renaming a heading would change its slug.
4. **Phrase key rename.** Renaming a phrase key updates `ascribe.toml` and every use.
5. **Formatting.** Provide document formatting through `tessera-fmt`, and format on save when `ascribe.formatOnSave` is enabled.
6. **Tests.** Scripted LSP tests for each action and refactoring against copies of `examples/quill`, asserting the resulting workspace edits and that the project still checks clean afterward.

## Acceptance criteria

- [x] Each quick fix above has a test that applies it and shows the diagnostic gone.
- [x] Moving `keys.md` into a subdirectory updates every link to it, and the project still checks clean.
- [x] Renaming an `@id` updates every link and include that names it.
- [x] Renaming a phrase key updates `ascribe.toml` and every use.
- [x] Format on save applies `tessera-fmt`'s edits and nothing else.

## Out of scope

- Refactorings the spec doesn't mention, such as extracting a fragment. Record ideas in `future-things.md`.

## Handoff notes

### Interfaces

- `textDocument/codeAction` reads the current diagnostic context. Diagnostic
  `data` carries the checker's same-file `Fix` objects; the route-to-file action
  therefore uses the resolver's reverse-route suggestion. The additional
  repairs edit the source or add a phrase entry to `ascribe.toml`.
- `textDocument/rename` handles explicit IDs, heading-derived IDs, and phrase
  keys. It uses the current `nav::Ctx` snapshot, parsed phrase uses, heading
  index, includes, and references. Phrase renames include parsed destination and
  opted-in-fence occurrences; escaped text and ordinary code are not indexed.
- `workspace/willRenameFiles` uses indexed local targets and their source spans
  to update links, reference definitions, includes, assets, and relative
  references in moved sources. Moves outside the content root or to an existing
  destination return no edits. External targets are left alone.
- `textDocument/formatting` returns `tessera-fmt`'s minimal edits only. The VS
  Code extension applies them before save when `ascribe.formatOnSave` is true
  and requests file-operation edits before workspace renames.

### Validation

- `cargo fmt --all -- --check` — passed.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `cargo test --workspace` — passed; the conformance harness reports 366
  passed, 0 failed, 0 skipped. The LSP's 16 scripted Phase 24 tests all pass.
- `cargo test -p tessera-conformance` — passed.
- `corepack pnpm --filter @ascribed/elements build` — passed.
- VS Code ESLint, TypeScript typecheck, Prettier check, and the non-browser
  Vitest suite — passed (76 tests).
- Full VS Code Vitest cannot launch its Playwright webview suite because the
  installed Chromium headless shell is absent. The real VS Code integration
  bundle builds, but the extension host could not be downloaded:
  `getaddrinfo ENOTFOUND update.code.visualstudio.com`.
- `target/debug/ascribe check --config examples/quill/ascribe.toml --format json`
  — passed with no diagnostics.

### Limitations and evidence

- The scripted tests run the language server on temporary copies of
  `examples/quill`; they apply returned workspace edits, notify the server, and
  check the resulting diagnostics. The file-move case includes a page, a
  referenced asset, inline and reference-style links, an include, outgoing
  relative references, and percent-encoded paths.
- The workspace file-operation filter is for files, not directory rename
  operations. Moves are confined to the configured content root.
- Platform-specific Windows/macOS VS Code behavior is not established by the
  Linux test run. The browser-dependent webview and real extension-host gates
  remain pending due to the missing browser and blocked VS Code download. No
  workflow was dispatched.
