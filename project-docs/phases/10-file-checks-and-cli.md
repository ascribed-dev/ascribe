# Phase 10: File-level checks and `tessera check`

**Track:** Check · **Start after:** 06, 07, 08 · **Finish after:** 03 · **Parallel with:** 11, 23 · **Unblocks:** 14, 15

## Goal

Build the diagnostics framework every tool shares, implement every file-level check, and ship the first working command: `tessera check`.

## Read first

- [SPEC.md](../../SPEC.md): §8 in full, and the sections each check comes from.
- `tests/conformance/diagnostics.toml` and `crates/tessera-core` (phase 02).
- Handoff notes from phases 05, 06, 07, and 08.

## Deliverables

- `crates/tessera-check`: the `Diagnostic` type, the registry mapping from slugs to codes and severities, and the file-level checks.
- `crates/tessera-cli`: the `tessera` binary's command structure and the `check` subcommand.

## Tasks

1. **Diagnostic type.** Code, slug, severity, message, primary span (with file), related spans, and optional fixes as `tessera-core` `TextEdit`s. The language server (15) and quick fixes (24) consume these, so fixes are data, not closures.
2. **Registry.** Load `diagnostics.toml` at build time (for example through `build.rs` or `include_str!`) so codes, severities, and message templates have one source. Turn `Issue`s from every crate into `Diagnostic`s.
3. **File-level checks.** Implement every §8.2 row whose level is `file` that the parser didn't already report. This includes:
   - attribute keys and value types against each directive's schema and the image-attribute schema;
   - `@variant` dimensions and values; `@available` targets, states, histories, and versionless targets (using phase 08's parser and model queries);
   - frontmatter against the content type's schema, including the reserved `available` and `variant` keys;
   - undeclared phrase candidates in prose (a warning);
   - referenced files that don't exist: include targets, link targets, and local asset sources (heading ids are checked in phase 14);
   - missing image alt text;
   - headings without `@id` that contain a phrase.
4. **One entry point.** Expose `check_files(&project) -> Vec<Diagnostic>` as the single file-level entry point. The CLI, the build command, and the language server all call it, which is what makes their results identical.
5. **CLI structure.** Set up `tessera-cli` with `clap`, with one module per subcommand so phases 15, 18, and 23 can add `lsp`, `build`, and `fmt` without conflicts. Add `--version`.
6. **`tessera check`.** Finds `tessera.toml` (in the current directory or a parent, or at `--config`), loads the model, parses every file under the content root, and reports diagnostics.
   - Readable output with source snippets (via `miette` or `ariadne`), and `--format json` with a documented, versioned schema.
   - Exit codes: `0` with no errors, `1` with errors, `2` for usage or configuration failures. `--deny-warnings` turns warnings into failure.
7. **Conformance.** Implement the adapter's `diagnostics` for file-level cases. Remove their skip entries, and make them pass.

## Acceptance criteria

- [ ] Every file-level row of §8.2 has a passing conformance case, reported with the registry's code and severity.
- [ ] `tessera check` on `examples/quill` reports no errors at file level.
- [ ] The JSON output schema is documented in `crates/tessera-cli/README.md`, with an example.
- [ ] Exit codes behave as specified; integration tests cover each.

## Out of scope

- Page-level checks (phase 14).
- The language server (phase 15).

## Notes

- Messages are part of the product. Say what's wrong and what to do, in plain words, and suggest the likely fix where there is one.

## Handoff notes

_To be filled in by the implementing agent._
