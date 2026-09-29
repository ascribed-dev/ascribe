# Phase 05: Tessera lines and syntax tree

**Track:** Parser · **Start after:** 02, 04 (with a go decision) · **Finish after:** 03 · **Parallel with:** 08, 09, 19 · **Unblocks:** 06, 07

## Goal

Build `tessera-syntax`'s public parsing API and its own syntax tree, and parse each directive line's head (name, attributes, primary, trailing colon) with exact source spans.

## Read first

- [SPEC.md](../../SPEC.md): §3.1–§3.4, §8.3, and Appendix A.
- `crates/comrak-tessera/SPIKE.md` and `FORK.md` (phase 04).
- `crates/tessera-core` and `tests/conformance/diagnostics.toml` (phase 02).

## Deliverables

- `crates/tessera-core/src/attributes.rs`: the attribute-block parser for SPEC §3.3, producing phase 02's attribute-value type. Phase 07 reuses it for images.
- In `crates/tessera-syntax`:
  - `parse(source, &ParseOptions) -> ParsedDocument`, where `ParseOptions` carries the directive schemas (built-ins plus project widgets, as `tessera-core` `DirectiveSchema`s).
  - Tessera's own syntax tree, independent of comrak's types. It covers every CommonMark block and inline kind Tessera needs, including images of every form, plus Tessera nodes: directive lines, end lines, and the node kinds for containers, groups, titles, phrase candidates, and image attributes, which phases 06 and 07 fill in.
  - Conversion from comrak's tree to Tessera's.
  - The directive-head parser.

## Tasks

1. **Attribute parser.** Implement SPEC §3.3 and the Appendix A rules exactly: tokens, quoted strings with `\"` and `\\`, value sets, any spaces or tabs, empty `{}`. Return each pair with spans for its key and value. Report malformed input as `Issue`s using registry slugs, never by panicking.
2. **The syntax tree.** Design node types for the whole of Tessera. Every node has a span. Tessera nodes keep sub-spans (name, attribute block and each pair, colon, primary). The formatter (23) and language server (15) depend on these, so document them. Define the node kinds phases 06 and 07 fill in now, so those phases can run in parallel without both editing the tree types.
3. **Directive heads.** For each `TesseraLine`, parse `@name`, optional attributes, optional `:` and primary. A `:` with no primary after it, only whitespace to the end of the line, is a container opener (SPEC §3.1, §3.5); a text primary that ends in `:` is still a primary. Classify the primary by schema: an identifier ends at whitespace, and text is the child paragraph from the fork. Report issues for a primary where the schema allows none, a missing required primary, and malformed attributes. Leave form and container errors to phase 06.
4. **Misspelled directives.** Lines shaped like directives with unknown names (`@word` followed by `{`, `:`, or the end of the line; SPEC §3.2) stay text in the tree. Record an issue with the closest known name for the "did you mean" warning.
5. **Conformance.** Implement the adapter's `outline` for what this phase produces (directive lines and CommonMark blocks, before containers exist). Remove the skip entries for the directive-head and attribute cases, and make them pass.

## Acceptance criteria

- [ ] `cargo test -p tessera-core -p tessera-syntax` passes.
- [ ] Conformance cases for recognition, attributes, and primaries (SPEC §3.1–§3.4) run and pass, including spacing tolerance, every quoting rule, and `@note: Important:`.
- [ ] Property tests (for example with `proptest`) show the attribute parser and head parser never panic on arbitrary input.
- [ ] Every span in the tree covers exactly its source text; a test checks this across all conformance inputs.

## Out of scope

- Containers, groups, titles, and bindings (phase 06).
- Phrases and image attributes (phase 07); leave the node kinds in place.
- Diagnostic codes, severities, and rendering (phase 10). Report slugs only.

## Notes

- Downstream crates use Tessera's tree, never comrak's. Keep comrak an implementation detail of `tessera-syntax`.

## Handoff notes

### What was built

- **`tessera_core::attributes`** (`parse_attribute_block(text, offset, file) -> Option<ParsedAttributes>`): SPEC §3.3 and Appendix A. Tokens, quoted strings with `\"` and `\\`, value sets, any spaces or tabs, `{}`. It returns the `AttributeBlock` (a span for the block, every pair, key, value, and set member), how many bytes it covered, whether it was closed, and `Issue`s. It recovers from every error, so the rest of a block is still read. It reports `attribute-syntax`, `attribute-bare-key`, `attribute-unquoted-reserved`, and `attribute-duplicate-key` (at the second use). Phase 07 reuses it for images.
- **`tessera-syntax`**:
  - `parse(source, &ParseOptions) -> ParsedDocument`. `ParseOptions` holds the file id, the `DirectiveSchema`s (`ParseOptions::default()` is the built-ins), and the note types (for the `@note {type=warning}:` suggestion).
  - `tree`: Tessera's own tree, documented. Blocks (`Heading`, `Paragraph`, `CodeBlock`, `BlockQuote`, `List`/`ListItem`, `HtmlBlock`, `ThematicBreak`, `Table`), inlines (`Text`, `Code`, breaks, `Html`, `Emphasis`, `Strong`, `Link`, `Image` in every form with `LinkForm` and `label`), and the Tessera nodes: `DirectiveLine` (name, `@name` span, attribute block, colon, primary, form) and `EndLine`. Primaries are `PrimaryValue::{Identifier, Text, Line, Unexpected}`. `raw_text(source, span)` reads source text with container prefixes removed.
  - **Node kinds for later phases**, defined and unused: `Container`, `Group`, `Arm`, `TitleLine` (phase 06), `Phrase` and `ImageAttributes`/`Image::attributes` (phase 07). Phase 05 never produces them (a test checks).
  - The directive-head parser (`head.rs`), the conversion from comrak (`convert.rs`), and misspelled-directive detection (`unknown.rs`).
- **The conformance adapter** (`tests/conformance/tests/adapters/syntax.rs`): handles the `parser` tag, with an `outline` (flat directives and CommonMark blocks) and `diagnostics` (the parser's issues, with columns in Unicode scalar values). It reads `[widgets]` and `[notes]` from the case's `ascribe.toml` itself, as a stand-in for phase 08's loader.
- **A fix in the fork.** comrak leaves a paragraph's or setext heading's start line on its link reference definitions when it takes them from the start, so every inline position after them was wrong. `parser/mod.rs` now moves the start line down (two marked hunks, listed in `FORK.md`).

### Interfaces later phases use

- **Positions.** `Span` is a byte range into the whole file (frontmatter included); every span covers exactly its source text and never includes a line ending. A test checks this over the 652 CommonMark examples (alone, and with a directive line before, after, and around each), the SPEC, project docs, examples, and every `.md` under `tests/conformance` (so phase 03's inputs are covered as soon as they merge).
- **Directive lines.** `BlockKind::Directive(DirectiveLine)` and `BlockKind::End(EndLine)`, as flat siblings of the blocks around them. `DirectiveLine::form` is decided by the line alone (`Container` iff there's a colon and nothing after it). A container's content is *not* inside it; phase 06 nests it. `span` runs from `@` to the end of the line, or of the primary's last line; trailing whitespace isn't included.
- **Text primaries.** `TextPrimary { span, lines, inlines }`. `lines` are the primary's lines with container indentation and `>` markers removed; use `raw_text(source, primary.span)` for the text. The fork's child paragraph is not a separate block in Tessera's tree.
- **Where the head parser and the block parser agree.** A text primary starts where `comrak-tessera` says (`NodeTesseraLine::text_primary`). Both find the end of an attribute block with the same rule (the first `}` outside a quoted string; a backslash inside quotes hides the next character). `tests/agreement.rs` checks it on 3,000 generated lines per run against the fork.
- **Issues** (`ParsedDocument::issues`, in source order): `attribute-syntax`, `attribute-bare-key`, `attribute-unquoted-reserved`, `attribute-duplicate-key`, `directive-primary` (a primary where none is taken, and, with the `missing` variant, a required one missing; located at the primary and at `@name`), and `directive-unknown` (with the `suggestion` variant when there's a close name). **Phase 10 shouldn't report these again**; it reports what needs a schema (unknown keys, value types).
- **Misspelled directives** stay `Paragraph`s. Any line of a paragraph (including a text primary's continuation lines) that is `@` and a lowercase name followed, after optional spaces and tabs, by `{`, `:`, or the end of the line, and whose name isn't known, is reported at `@name`.
- **Bindings and titles.** Not in the tree. The adapter works out `binding` for the outline from the schema and the siblings before a directive, as a stand-in; phase 06 replaces that. A title line is still a one-line `Paragraph`.

### Decisions

- **The tree owns the text.** `Text` and `Code` values are comrak's decoded values; the span is the source. Adjacent `Text` nodes that touch are merged, so phase 07 finds `{key}` in one node.
- **comrak positions are corrected, not trusted.** Block and inline spans come from comrak's line and byte column, converted with `LineIndex`, then corrected where comrak is wrong: containers grow to cover their last child (a list ending in an indented code block after a tab, and a table, end short); a heading or paragraph loses trailing spaces; table cells put back the byte comrak drops for each `\|` before a position; a line break's span is its line ending; and the fork change above.
- **A directive line's Tessera-specific parts are computed from the raw line**, anchored at the `@` (`NodeTesseraLine::raw`), not from comrak's columns.
- **Reporting places.** A missing `}` and an unclosed quote are reported at the `{` and the opening quote. A missing required primary is at `@name`; an unexpected primary is at the primary.
- **A recovered `key=` with no value** keeps the pair with `value: None`, as `AttributeBlock` documents, and reports `attribute-syntax`; a bare key reports `attribute-bare-key`. Unquoted values with whitespace (`{label=Using other images}`) report `attribute-unquoted-reserved` and keep the whole value; a missing comma before another `key=` reports `attribute-syntax`. Only `attribute-syntax` is reported for an unclosed block.
- **Invalid keys** (`Type`, `my_key`) are reported as `attribute-syntax` and kept, so their values still parse.
- **`proptest`** is a workspace dependency now (dev-dependency of `tessera-core` and `tessera-syntax`).

### Left open

- **Q30** (`project-docs/questions.md`): text on a directive line that fits no part of the directive (`@note hello: text`, `@steps foo`, `@end: x`). Resolved with Q15 on 2026-09-28: all five shapes are reported as `directive-extra-text` (variants `head` and `end`, and the base message for text after an identifier), and text after an identifier is still kept in `IdentifierPrimary::trailing`.
- **Conformance.** Phase 03 has merged. All 36 `parser`-tagged cases run and pass against the adapter (the `parser` skip entry is removed); cases tagged with other areas stay skipped until their phases land. Q14's proposal (option 1) is what the attribute parser implements, and Q15's is what the identifier primary does.
- **Link reference definitions** aren't nodes (comrak doesn't produce them); the destinations of reference-form links and images are on the `Link` and `Image` nodes. Phase 23 exposed the definitions themselves as a side list, `ParsedDocument::definitions`, with a small fork change (`comrak_tessera::parse_document_with_definitions`).
- **Setext headings and paragraphs after definitions** have their start line corrected, but the column is the paragraph's original one, which is right unless the definitions were indented differently from the text after them.
- **Trailing whitespace inside an unclosed attribute block's quote** is left out of the block's span.

