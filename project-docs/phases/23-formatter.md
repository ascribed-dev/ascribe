# Phase 23: Formatter

**Track:** Format · **Start after:** 06, 08 · **Parallel with:** 10, 11, and later waves · **Unblocks:** 24

## Goal

Build `tessera-fmt` and `ascribe fmt`: rewrite Ascribe constructs into canonical form with minimal text edits, never touching the author's ordinary markdown.

## Read first

- [SPEC.md](../../SPEC.md): §8.3, §3.1, §3.3, §3.8, and §10's note on general-purpose formatters.
- [PLAN.md](../PLAN.md): Formatter.
- Phase 05's handoff notes, for the sub-spans on directive lines.

## Deliverables

- `crates/tessera-fmt`: `format(source, &ParseOptions, &ContentModel) -> Vec<TextEdit>`.
- `crates/tessera-cli/src/fmt.rs`: `ascribe fmt [paths] [--check]`.

## Tasks

1. **Rules.** Implement every canonical-form rule in SPEC §8.3, each as its own function producing edits:
   - one space between a directive name and `{`;
   - no space inside braces, none around `=` or `|`, and `, ` between pairs;
   - attributes in the order the directive's schema declares them (built-ins from `tessera-core`, widgets from the model);
   - values quoted only when necessary;
   - no empty `{}`;
   - `:` directly after the name or attribute block, then one space before a primary; nothing, including trailing whitespace, after a container's colon;
   - no blank line between a following-block directive and its block;
   - directive lines in list items indented to the item's content column.
2. **Minimal edits.** Only change bytes inside Ascribe constructs, the blank line between a directive and its block, and indentation of directive lines. Never reflow or re-render markdown.
3. **Safety.** Skip formatting a construct that has parse errors, rather than guessing.
4. **`ascribe fmt`.** Formats files in place; `--check` reports files that would change and exits `1`, for CI.
5. **Tests.** For every conformance input, formatting is idempotent (formatting twice equals formatting once), and the formatted file parses to the same outline as the original. Add focused tests for each rule.

## Acceptance criteria

- [ ] Every §8.3 rule has a test.
- [ ] Idempotence and outline preservation hold across all conformance inputs.
- [ ] A test with hand-wrapped prose, tables, and code shows ordinary markdown is untouched.
- [ ] `ascribe fmt --check` exits `1` on unformatted input and `0` on formatted input.

## Out of scope

- Format on save in the editor (phase 24).

## Handoff notes

### What was built

- **`crates/tessera-fmt`**: `format(source, &ParseOptions, &ContentModel) -> Vec<TextEdit>` (edits sorted, non-overlapping, none for a canonical file), `format_parsed` (for a tree already parsed), `format_source` (the edits applied), `options_from_model` (the `ParseOptions` a model gives: its directive schemas and note types), and `NON_BLOCKING`. Each SPEC §8.3 rule is its own function in its own file:
  - `head.rs`: `name_gap`, `empty_block`, `colon`, `primary_gap`, `container_trailing`;
  - `attributes.rs`: the attribute block (spacing, schema order, quoting), for directives and images;
  - `indent.rs`: indentation of directive lines and end lines (Q1, Q19), for the document, list items, and block quotes;
  - `blank.rs`: the blank line between a following-block directive and its block (a gap between the blocks of a list item stays, since closing it can change the list's tightness, Q74);
  - `skip.rs`: which reported issues make a construct untouchable.
- **`crates/tessera-cli/src/commands/fmt.rs`**: `ascribe fmt [paths] [--check]` as a subcommand in phase 10's structure (`Args`, `run(&Global, Args)`; `--config` is honored). Exit codes: 0, 1 under `--check` when a file would change, 2 for a problem.
- **Link reference definitions** (the extra task, and Q43):
  - **The fork**: `comrak_tessera::parse_document_with_definitions` returns the document and a `Vec<tessera::LinkDefinition>` (positions of the definition, label, destination, and title, the cleaned URL and title, the normalized label). `parse_document` delegates to it. The change is in `parser/mod.rs` (5 marked hunks, 68 lines added, 7 changed) and `lib.rs` (1), listed in `FORK.md`; `LinkDefinition` and `locate` are in `src/tessera.rs`. No node and no `NodeValue` variant was added, and nothing that was parsed changes: the CommonMark suite is 652/652 in every mode.
  - **The tree**: `ParsedDocument::definitions: Vec<LinkDefinition>` (a side list, not a `BlockKind`): `span`, `label` and `label_text`, `normalized_label`, `destination` (as written, `<>` included) and `url` (decoded), `destination_phrases`, `title: Option<DefinitionTitle { span, text }>`. `\{key}` in a destination is text and is in `escaped_phrases`. Code: `tessera-syntax/src/inline/definition.rs`, plus `convert.rs`. Tests: `tests/definitions.rs` (21), the span checker (`tests/support`, run by `tests/spans.rs` over every input) checks every span of every definition, `comrak-tessera/tests/definitions.rs`.
  - The formatter never changes a definition, and never deletes a line in a gap that holds one (Q74).
- **The conformance harness**: `expect.yaml` has `formatted: <file>` (what formatting `input.md` gives); `ConformanceAdapter::format(case, source)`; the runner compares, and formats the result again to check idempotence; `Check::Format` for skips. `tests/adapters/format.rs` handles the `format` tag with `tessera-model` and `tessera-fmt`. The `format` skip entry is gone. 41 cases in `cases/format/` cover every §8.3 rule, what the formatter leaves alone, and constructs with errors. The harness's own tests have three fixture cases for the new check.
- **Q71 to Q77** (below), and `Q43`'s resolution and phases 05 and 07's notes now say definitions are implemented.

### Interfaces later phases use

- **24 (quick fixes, format on save)**: call `tessera_fmt::format(source, &options_from_model(&model), &model)` and convert the edits' spans with `LineIndex`; they are already sorted and disjoint. The formatter needs a loaded `ContentModel`, which the language server has. It doesn't need the parse: `format_parsed` takes one if the server already has it.
- **10, 12, 14**: `ParsedDocument::definitions` and `LinkDefinition::destination_phrases`. A reference link's destination is on the `Link`; the definition's phrase candidates are on the definition. The first definition of a normalized label wins; later ones are in the list, marked by nothing (compare `normalized_label`).
- **The conformance adapter** for `format` uses the shared model through `tessera-model`, which is the first use of that model by an implementation crate. It didn't load (Q27 renamed the `role` attribute in `examples/content-models/full.toml` but not here), so `tests/conformance/_model/ascribe.toml` and the four case files that use it say `audience` now (`attributes/value-set`, `attributes/set-on-set-valued-key`, `widgets/audience-heading-or-block`). **Phases 10 and 11 will meet the same failure**; if they made the same change, the merge is trivial.

### Decisions

- **Edit only Ascribe constructs, from spans.** The formatter never renders markdown. Everything it touches is a gap between two parts of a directive head, an attribute block, a directive line's indentation, or a run of blank lines; prose, tables, code, definitions, HTML, frontmatter, and title lines are byte for byte as written (a test with hand-wrapped prose, tables, code, and definitions checks it, in `tessera-fmt/tests/rules.rs` and `format/markdown-untouched`).
- **Errors stop a construct, warnings don't** (Q71). "On" means the issue's location is within the directive line (with its title) or end line, or the image's attribute block. The list of warnings that don't stop it is checked against `diagnostics.toml`.
- **An attribute block is rewritten as one edit, and only after its canonical text is parsed again** with `parse_attribute_block` and found to have the same keys, values, and forms (Q75). Values: a quoted string whose text could be a token loses its quotes (types come from the schema, not from spelling); one that needs them keeps its source exactly.
- **Order comes from the schemas**: `ParseOptions::schemas` for a directive (built-ins from core, widgets from the model), `ContentModel::dimensions` for `@variant`, `ContentModel::image_attributes` for images. An undeclared key keeps the whole block's order.
- **Indentation comes from the tree, not from column arithmetic on the line**: a directive line's owner is the innermost list item, block quote, or the document (directive containers don't indent). A list item's content width is computed from its marker. A line whose prefix holds anything but spaces and `>` is left alone.
- **`\{key}` and the other escapes need no code**: the formatter doesn't touch prose.
- **Checks** (all in `crates/tessera-fmt/tests/`): `rules.rs` (60 tests, one group per rule), `inputs.rs` (every input under `tests/conformance/cases`, the examples, the project docs, the SPEC, and the CommonMark examples, each also made ugly in eight ways; idempotent, and the outline, which ignores spacing and attribute order, is unchanged), `fuzz.rs` (random documents, and arbitrary text), `registry.rs`. I ran `fuzz.rs` at 60,000 and 30,000 cases once; it runs 3,000 and 1,000 in CI.

### Left open

- **Q71 to Q77** were resolved on 2026-09-28 as implemented (Q74 in its revised, tightness-preserving form), SPEC §8.3 now states them, and their cases are no longer `provisional`.
- **The formatter can't check what needs the model**: an unknown key or a wrong value type is phase 10's diagnostic; the formatter formats such a block as far as it is safe to.
- **Format on save and range formatting** are phase 24's. `format` formats the whole file.
- **`BlockKind::Title`** is still never produced (phase 06's note); nothing here matches on it except to skip it.
- **The phase 05 stand-in model loader** in `tests/conformance/tests/adapters/syntax.rs` is still there for `parser`, `structure`, and `inline`; the `format` adapter uses `tessera-model`.

