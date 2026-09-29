# Phase 04: Parser spike

**Track:** Parser · **Start after:** 00 · **Parallel with:** 02, 17, and wave C · **Unblocks:** 05 · **Human checkpoint after this phase (go or no-go)**

## Goal

Prove, with a small amount of code, that comrak can be forked to support Ascribe's block-level changes to CommonMark. End with a recommendation: continue with the fork, or switch to the fallback (markdown-rs).

## Read first

- [SPEC.md](../../SPEC.md): §1.4, §3.2, §3.4, §3.9.
- [PLAN.md](../PLAN.md): Parser.
- comrak's source, especially its block parser and how it adds extension block types such as tables and front matter.

## Deliverables

- `crates/comrak-tessera/`: comrak, vendored at a pinned release tag, with its license. Add `FORK.md` recording the upstream version, the reason for the fork, the procedure for merging upstream releases, and every changed location.
- A minimal Ascribe-line block in the fork.
- Spike tests in the fork's test directory.
- `crates/comrak-tessera/SPIKE.md`: findings and a recommendation.

## Tasks

1. **Vendor comrak.** Copy the source at the latest release tag into `crates/comrak-tessera`, rename the crate, keep the license, and add it to the workspace. Confirm the CommonMark baseline from phase 00 passes unchanged.
2. **Options.** Add a Ascribe option to comrak's extension options, carrying the set of known directive keywords (including `end`) and, for each keyword, whether it takes a text primary. Hardcode the built-in set in tests; later phases supply it from the content model.
3. **The Ascribe-line block.** Add a leaf block node, `TesseraLine`, holding the raw line and its source position. It's recognized at line start (after container indentation) when the line is `@` followed by a known keyword and then whitespace, `{`, `:`, or the end of the line. It must:
   - **interrupt a paragraph**, as an ATX heading does;
   - **never be a lazy continuation line**, so an unindented directive line after a list item ends the list;
   - be recognized inside list items and blockquotes according to CommonMark's container rules, and be literal text when over-indented into an indented code block;
   - when the keyword takes a text primary and the line has a non-empty one, **continue onto following lines as a paragraph does**. One approach is to give the node a child paragraph holding the primary text, so comrak's inline parser handles it.
4. **Spike tests.** Show that each of these parses correctly:
   - A directive line directly after a paragraph line starts a new block.
   - `1. Install\n@note` ends the list at `@note`.
   - A directive line indented to a list item's content column belongs to the item.
   - A directive line indented four or more spaces past the content column is code.
   - `@unknown: text` and `@astrojs/react` stay paragraph text.
   - `@note {type=caution}: Back up your database` followed by `before you upgrade.` keeps both lines in the primary.
   - A directive line inside a fenced code block is code.
5. **Rerun the CommonMark suite** against the fork with the Ascribe option on. Record every newly failing example and why. The only acceptable failures involve lines that are valid directive lines.
6. **Write SPIKE.md**: what worked, what was hard, how intrusive the change is (lines changed and where), how merging future upstream releases would go, and a clear recommendation.

## Acceptance criteria

- [x] Every spike test in task 4 passes.
- [x] The CommonMark suite passes with the Ascribe option off, and with it on apart from documented, justified exceptions.
- [x] `FORK.md` lists every changed location, each marked in the code with a `// TESSERA:` comment.
- [x] `SPIKE.md` ends with a go or no-go recommendation and its reasons.

## Out of scope

- Parsing the directive head (phase 05), structure (phase 06), and inline constructs (phase 07).

## Notes

- Keep changes to comrak small and confined. Every changed line is a future merge conflict.
- If the fork proves impractical, stop, document why in SPIKE.md, and recommend the fallback. The phase is still a success.

## Handoff notes

### Recommendation

**Go: continue with the comrak fork.** The reasons and evidence are in [`crates/comrak-tessera/SPIKE.md`](../../crates/comrak-tessera/SPIKE.md). In short: every block-level requirement works; the CommonMark 0.31.2 suite passes 652/652 with the option off and on, with no exceptions; upstream's 490 unit tests pass; the change to upstream is 44 lines of code in 7 files, each location marked; and the same patch applied cleanly to comrak releases from the previous eight months. A human confirms go or no-go before phase 05 starts.

### What was built

- **`crates/comrak-tessera`**: comrak **v0.55.0** (the latest release, tag commit `6fbe87f`), vendored unchanged in its own commit ("Vendor comrak v0.55.0 as crates/comrak-tessera"), minus the CLI's `src/main.rs`, with upstream's `COPYING` (BSD-2-Clause). The manifest is Ascribe's: package `comrak-tessera`, library `comrak_tessera`, version `0.55.0`, `publish = false`, no default features, the CLI and syntect features removed, doctests off (they import `comrak`). It's a workspace member and a workspace dependency (`comrak-tessera.workspace = true`). It doesn't use the workspace lints; see `FORK.md`.
- **`FORK.md`**: the upstream version, why the fork exists, what was and wasn't vendored, the manifest changes, a table of every changed location (file, marker count, function, what), and the procedure for merging an upstream release. `tests/fork_md.rs` fails if the table's marker counts disagree with the `// TESSERA:` markers in `src/`.
- **The Ascribe option and block**: `src/tessera.rs` (option, node, scanner, HTML rendering) and `src/parser/tessera.rs` (the parser hooks), plus 19 marked hunks in 7 upstream files and 3 in upstream's sourcepos test.
- **Spike tests**: `tests/spike.rs`, 22 tests covering every case in task 4 and the edge cases listed in `SPIKE.md`; scanner unit tests in `src/tessera.rs`.
- **CommonMark suite against the fork**: `tests/commonmark/tests/fork.rs`, with baselines `comrak-tessera-off.toml` and `comrak-tessera-on.toml`. Off: 652/652, and the test also requires it to match unmodified comrak's baseline. On (built-ins, `end`, one widget): 652/652, and no example's HTML changes. A third run puts an `@end` line before every example: 652/652. Documented in `tests/commonmark/README.md`.
- **`SPIKE.md`**: what worked, what was hard, how intrusive the change is, a dry run of upstream merges, risks for later phases, the fallback, and the recommendation.

### Interfaces later phases use

All in `comrak_tessera` (depend on `comrak-tessera.workspace = true`; the rest of comrak's API is unchanged):

- `tessera::TesseraOptions`: `new()`, `keyword(name, text_primary) -> Self`, `insert(name, text_primary)`, `get(name) -> Option<TesseraKeyword>`, `len()`, `is_empty()`. Names are given without `@`. The caller supplies every keyword, including `end`; the fork knows no built-ins. Phases 05 and 08 build the set from the built-in directives and the content model's widgets.
- `tessera::TesseraKeyword { text_primary: bool }`.
- `options.extension.ascribe: Option<Arc<TesseraOptions>>`. `None` (the default) is plain comrak.
- `nodes::NodeValue::TesseraLine(Box<tessera::NodeTesseraLine>)`, with:
  - `raw: String`: the line from `@` to the end of the line, without the line ending; trailing whitespace is kept. Container indentation and blockquote markers aren't included.
  - `name: String`: the keyword, such as `note` or `end`.
  - `text_primary: Option<usize>`: the byte offset in `raw` where a non-empty text primary starts. `Some` exactly when the node has a child.
  - Children: none, or one `Paragraph` holding the text primary from its first character through its last continuation line, already parsed into inlines.
  - Sourcepos: starts at the `@`; ends at the end of the line, or of the primary's last line. Columns are comrak's (1-based bytes, unless `parse.sourcepos_chars` is on).
- Recognition: `@`, a known keyword (a greedy run of `[a-z0-9-]` looked up in the set), then a space, tab, `{`, `:`, or the end of the line, at a block start that isn't indented four or more columns past its container. A text primary starts after an optional attribute block (quoted strings honored) and a `:`, and must be non-empty; a head the scanner can't read gets no primary. Parsing the head properly is phase 05's job.
- The HTML renderer emits `<div data-tessera-line="RAW">…</div>`, and the XML renderer a `tessera_line` element with a `raw` attribute. Both exist only so comrak's renderers handle every node.

### Decisions

- **Vendored from the upstream tag rather than the crates.io package**, so upstream's unit tests (`src/tests/`) come along. They run in the workspace, and they caught a real bug during the spike.
- **The primary is a real `Paragraph`**, the approach the phase suggested. It inherits every paragraph rule (interruption, lazy continuation, indented lines) with no extra code, at the cost of three guards for the paragraph conversions that don't apply to inline content (Q2).
- **New match arms sit next to old, stable neighbors** (`Document`, `FrontMatter`, `Paragraph`), not at the end next to upstream's newest variant, so upstream's appended variants don't conflict. `handle_ascribe_line` sits between block quotes and ATX headings in the block-start chain.
- **Up to three spaces of extra indentation are allowed** before a directive line, as for a heading (Q1).
- **`is_text_primary` uses `try_borrow`**, because some upstream containers finalize their children while holding their own `RefCell` borrow.
- **Doctests are off** in the fork. The one Ascribe doc example is duplicated as the test `module_example`.
- **Three lints are allowed in the fork's manifest** (`deprecated`, `clippy::vec_init_then_push`, `clippy::write_with_newline`), all tripped only by upstream's tests, so upstream code stays unchanged.
- **The root `Cargo.toml`** gained one workspace member and one workspace dependency (`comrak-tessera`), nothing else. The crates.io `comrak` dependency stays, for phase 00's unmodified baseline.

### Left open

- **Spec questions Q1 and Q2** were resolved on 2026-09-28 as implemented, and SPEC §1.5, §3.4, and §3.9 now state them. The go decision on the comrak fork was approved at the same checkpoint.
- **For phase 05**: positions are comrak's line and byte column; convert them with the line index, and compute the head's sub-spans from `raw`, the node's start column, and `text_primary`. Test phase 05's head parser against the fork's scanner: they must agree on where a text primary starts. The Ascribe line's child paragraph is its primary, not content.
- **For phase 07**: inline extensions go into `src/parser/inlines.rs`. comrak's `attributes` feature already parses `{…}` after images and links (with a Pandoc grammar, not Ascribe's), and `{` is already dispatched there for Phoenix HEEx; both are models to follow. Mark every change `// TESSERA:` and add it to `FORK.md`'s table (the test enforces the counts).
- **For phase 23**: comrak's CommonMark renderer drops the escape in `\@note`, so its output would be a directive. The formatter must escape a line-initial `@keyword`, `.` title lines, and phrases itself.
- **Not guarded**: comrak's description-lists extension could still turn a text primary into a term. Ascribe doesn't enable it.
- **Conformance**: this phase added no conformance cases and no adapter; the `parser` tag stays skipped until phase 05.
- **CI** ran on the phase 04 pull request (#3) and passed: Rust fmt, then clippy, build, and test on Linux, macOS, and Windows, and the JS workflow. The logs show the spike tests, the `FORK.md` check, and all three CommonMark runs against the fork (652/652 each) on all three platforms.
