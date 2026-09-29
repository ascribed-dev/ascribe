# Phase 10: File-level checks and `ascribe check`

**Track:** Check · **Start after:** 06, 07, 08 · **Finish after:** 03 · **Parallel with:** 11, 23 · **Unblocks:** 14, 15

## Goal

Build the diagnostics framework every tool shares, implement every file-level check, and ship the first working command: `ascribe check`.

## Read first

- [SPEC.md](../../SPEC.md): §8 in full, and the sections each check comes from.
- `tests/conformance/diagnostics.toml` and `crates/tessera-core` (phase 02).
- Handoff notes from phases 05, 06, 07, and 08.

## Deliverables

- `crates/tessera-check`: the `Diagnostic` type, the registry mapping from slugs to codes and severities, and the file-level checks.
- `crates/tessera-cli`: the `ascribe` binary's command structure and the `check` subcommand.

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
6. **`ascribe check`.** Finds `ascribe.toml` (in the current directory or a parent, or at `--config`), loads the model, parses every file under the content root, and reports diagnostics.
   - Readable output with source snippets (via `miette` or `ariadne`), and `--format json` with a documented, versioned schema.
   - Exit codes: `0` with no errors, `1` with errors, `2` for usage or configuration failures. `--deny-warnings` turns warnings into failure.
7. **Conformance.** Implement the adapter's `diagnostics` for file-level cases. Remove their skip entries, and make them pass.

## Acceptance criteria

- [ ] Every file-level row of §8.2 has a passing conformance case, reported with the registry's code and severity.
- [ ] `ascribe check` on `examples/quill` reports no errors at file level.
- [ ] The JSON output schema is documented in `crates/tessera-cli/README.md`, with an example.
- [ ] Exit codes behave as specified; integration tests cover each.

## Out of scope

- Page-level checks (phase 14).
- The language server (phase 15).

## Notes

- Messages are part of the product. Say what's wrong and what to do, in plain words, and suggest the likely fix where there is one.

## Handoff notes

### What was built

- **`crates/tessera-check`**:
  - `Diagnostic` (`code`, `slug`, `severity`, `message`, primary `Location`, `related`, `fixes` as `tessera_core::Fix`es of `TextEdit`s), built from an `Issue` by `Diagnostic::from_issue`.
  - `Registry` (`registry.rs`): `tests/conformance/diagnostics.toml`, embedded with `include_str!` and parsed once. It gives each slug's code, severity, level, and message templates (`{name}` placeholders, `{{`/`}}` braces, variants by `Issue::variant`). A `debug_assert!` fails a test when an issue lacks an argument its template uses, and a unit test keeps the registry equal to `tessera_core::diagnostics::ALL`.
  - `Project` (`project.rs`): the content model (file id 0), the source files (ids from 1, in path order, texts in memory), the project root and the content root (relative to it). `Project::load(config)` reads `ascribe.toml` and every `.md` under the content root; `Project::from_parts` and `Project::from_sources` build one from buffers or a test. Source discovery, the exact-case probe, the boundary, and every rule about what a reference names come from `tessera-resolve` (`DiskFs`, `Layout`, `tessera_resolve::references`), since the consolidation with phase 11; see phase 11's handoff notes. An in-memory source counts as existing (`Project` implements `SourceSet`).
  - `check_files(&Project) -> Vec<Diagnostic>`: the model's warnings, then `check_file` for each source, in file order and source order. `check_file` parses with the model's directive schemas, turns the parser's issues (phases 05 to 07) into diagnostics, and walks the tree.
  - The checks, one module each under `src/checks/`: `attrs` (unknown keys, value types, required attributes, `@variant` dimensions and values, for directives, widgets, and images), `avail` (`@available` and the `available` key: syntax, unknown targets and states, versionless targets and dimension names with versions, history order), `frontmatter` (content type, `validate_frontmatter`, reserved keys, `available`, `variant`), `refs` (`@include` targets, link destinations, image sources, alt text, image attributes). `mod.rs` also has undeclared phrases, headings with a phrase and no `@id`, and `@id` values.
  - `yaml.rs`: `YamlIndex`, which finds where a frontmatter path (`author.name`, `tags[1]`) is, with `yaml-rust2` events, so a frontmatter problem is reported at its key or value.
- **`crates/tessera-cli`**: `clap` derive; `cli.rs` (options every command shares: `--config`, `--color`; the `Command` enum and its one `match`), one module per subcommand under `commands/` (`check.rs`), `context.rs` (find the model and load the project), `report/` (`text.rs` with `ariadne` snippets, `json.rs`, and a `FileTable` that turns byte offsets into lines and columns for both), `exit.rs`. `ascribe --version`, `ascribe check [--format text|json] [--deny-warnings]`. The README documents the JSON schema (version 1), the exit codes, and an example.
- **Conformance**: `tests/adapters/check.rs` (tag `check`: `check_files` on a project built from the case), `tests/adapters/model.rs` (tag `model`: the loader's diagnostics; it makes the two `model-name-multiple-roles` cases run), the `check` and `model` skip entries removed, `structure_rows.rs` deleted (see below), and `tests/file_checks.rs`.
- **`tessera-core`**: `parse_attribute_block` reports only the unclosed quote when a quote is what left the block unclosed (`{lab="first sync}` gave two `attribute-syntax` diagnostics, and the case `attributes/unclosed-quote` expects one; SPEC §3.3: an unclosed block is reported once).

### Interfaces later phases use

- **14, 15, 18:** `tessera_check::check_files(&Project)`. The language server builds the `Project` from its buffers with `Project::from_parts` and `Project::from_sources`; `Project::file(id)` gives a file's text and display path for any `Location`. Page-level checks (phase 14, `src/page/`) can call `Diagnostic::from_issue` and use the `Registry` the same way.
- **15, 24:** every `Diagnostic` has its fixes as `TextEdit`s in the file's own byte offsets. These carry one: `attribute-unknown-key` (suggestion), `frontmatter-unknown-key` (suggestion), `directive-unknown` (suggestion; `@warning:` becomes `@note {type=warning}:`), `directive-extra-text` (remove the text), and `link-route` (when the page exists). Fixes that need judgment (a missing value, a wrong type) have none. Tests apply each fix and check the diagnostic is gone.
- **14:** for `link-route` the crate uses the conventional route-to-page mapping (Q55). When phase 12's router exists, ask it instead.
- **18:** `tessera_cli::report` is private to the binary crate today. When `build` needs to show diagnostics, move `report/` into a library target, or call it from a `build` module in the same crate (it's already in the crate).
- **23:** add `commands/fmt.rs`, `mod fmt;` in `commands/mod.rs`, one `Command::Fmt` variant and one arm in `cli.rs`. Nothing else in `tessera-cli` needs to change.

### Decisions

- **The parser's issues aren't repeated.** `check_file` starts from `ParsedDocument::issues` and adds only what needs the model or the file system. `widget-schema` is the parser's for a missing title and this crate's for a missing required attribute (its `missing-attribute` variant).
- **Locations.** Diagnostics point at the smallest span that shows the problem: the attribute key or value, a set member, the destination of an inline link or image, the `@name` for a missing required attribute, the whole node for a reference form (Q53), the frontmatter key or value, and the first line of the file for a frontmatter problem with no line of its own (Q20).
- **Asset rules** (Q10, the asset contract): a reference resolves from the file it's written in, must be inside the project root or the content root, not inside the output directory (`outside` variant), and match exactly (`case` variant). A local destination that names no file and looks like a route gets `link-route` instead of `link-target-missing` (Q22).
- **Phrases in destinations are substituted before the file checks** (Q54), so Appendix B's `[…]({api}streaming)` is a URL.
- **New questions**, all with the conservative behavior implemented: Q51 (frontmatter that isn't YAML), Q52 (which files are sources), Q53 (where reference diagnostics are reported), Q54 (phrases in destinations), Q55 (route to page), Q56 (where `phrase-undeclared` applies), Q57 (`available`/`variant` values of the wrong shape), Q58 (a model with errors: exit code 2, its warnings in the list), Q59 (empty and fragment-only destinations). None affects a case, so no case is `provisional`.
- **`tests/file_checks.rs` replaces `structure_rows.rs`.** With the `check` tag handled, the runner covers every case `structure_rows.rs` covered (the outlines and the diagnostics of the eight `check`+`structure` cases run whole). `structure_rows.rs` also compared only the structural slugs; `check_files` reports the whole list, and the runner compares it exactly. `file_checks.rs` is what the runner can't do yet: cases that carry `check` and a tag whose phase isn't built (`include`, `resolve`, `slug`, `page-check`) are skipped by the runner, but their top-level (file-level) diagnostics don't depend on those phases, so that test checks them now. Delete it when the last of those adapters lands, if it's redundant by then.

### Fixes to inputs from earlier phases

- **`role` in the shared model** (Q27): `tests/conformance/_model/ascribe.toml` still declared the widget attribute `role`, which the loader rejects (`model-attribute-reserved`), so no case could load the shared model. It's `audience` now, in the model and in the three cases that use it (`attributes/value-set`, `attributes/set-on-set-valued-key`, `widgets/audience-heading-or-block`).
- **`directives/include/bare-filename-is-not-searched`**: `files/prerequisites.md` is a page, and had no `title`, so it drew `frontmatter-missing-field`. It has one now; the case's expectation is unchanged.
- **`attribute-syntax` for an unclosed quote** (above), a two-line change in `tessera-core/src/attributes.rs` with its unit test.

### Acceptance criteria

See the pull request for the status and evidence of each.

### Left open

- **Q51 to Q59** were resolved on 2026-09-28 (`project-docs/questions.md`): Q51 added `frontmatter-syntax`, Q52 made an unreadable or non-UTF-8 source a `source-unreadable` error on that file (the rest is still checked), and Q59 added the `empty` message variant for an image with no source.
- **Page-level rows** (`id-duplicate`, `include-id-missing`, `include-cycle`, `variant-no-arm-survives`, `available-exceeds-scope`, `link-id-missing`, `link-id-in-fragment`, `link-id-removed`, `heading-duplicate-without-id`, `link-page-dropped`) are phase 14's, on phase 11's index and phase 12's resolution.
- **One YAML parser would be safer.** Values come from `serde_yaml` and positions from `yaml-rust2`, which accept slightly different inputs. When the position index misses a path (a non-string key, or a parse that only one accepts), the diagnostic falls back to the file's first line (never an empty span), and a test covers it. Using one parser for both would remove the risk; `serde_yaml` is deprecated upstream anyway. Not this PR's job.
- **Q59** is implemented as proposed for images (an empty or `#id`-only source is `image-source-missing`, worded as a missing file); a dedicated message needs a registry change for the human to approve. The case `images/source-empty` is `provisional` on it.
- **Performance.** `FileSystem::probe` reads each directory of a path on every reference (no cache, so a `Project` never serves a stale listing). Phase 26 measures it; a per-check cache of directory listings is the first thing to try.
- **Definitions.** A reference-style link's destination is in a definition that isn't a node, so its diagnostics are at the link (Q53), and `[ref]: {api}x` can't be checked for phrases in the definition's own position.
- **`tessera-cli`'s JSON schema** is version 1. A field added later doesn't change the version.
