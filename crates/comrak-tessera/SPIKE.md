# Parser spike: findings and recommendation

Phase 04 asked whether comrak can be forked to support Tessera's block-level changes to CommonMark with a small amount of code, or whether Tessera should switch to the fallback, markdown-rs. This is the answer.

**Recommendation: go.** Continue with the comrak fork. The details are at the end.

## What was built

- comrak **v0.55.0**, the latest release, vendored unchanged as `crates/comrak-tessera` in its own commit, then changed. [`FORK.md`](FORK.md) records the upstream version, the reason for the fork, the merge procedure, and every changed location.
- **The Tessera option**: `options.extension.tessera: Option<Arc<TesseraOptions>>`, the set of known keywords and, for each, whether it takes a text primary. Off (`None`) by default.
- **The Tessera-line block**: `NodeValue::TesseraLine(Box<NodeTesseraLine>)`, holding the raw line (from `@` to the end of the line), the directive's name, and the byte offset of a non-empty text primary. Its sourcepos covers the line and every line its primary continues onto. With a text primary, it has one child, a paragraph holding the primary, which comrak's inline parser parses; otherwise it has no children.
- **Spike tests** in `tests/spike.rs` (22 tests), a check that `FORK.md` matches the code's markers (`tests/fork_md.rs`), scanner unit tests in `src/tessera.rs`, and the CommonMark suite against the fork in `tests/commonmark/tests/fork.rs`.

## What worked

Every requirement in the phase fits comrak's existing block-parser model, which is a port of the CommonMark reference implementation (cmark):

| Requirement | How | Upstream code touched |
|---|---|---|
| Recognized at line start, after container indentation | One more entry in `open_new_blocks`'s chain of block starts. Container markers and indentation are already consumed there, and the chain is already skipped for lines indented four or more columns. | One line |
| Interrupts a paragraph | Adding the node closes the open paragraph, as for an ATX heading. | None |
| Never a lazy continuation line | A new block start is never lazy. An unindented `@note` after `1. Install` closes the list, because `add_child` closes every container that can't hold the node. | None |
| Follows list and blockquote container rules; over-indented is code | Inherited from the chain's position. | None |
| A text primary continues as a paragraph does | The node gets a child `Paragraph` starting at the primary. That paragraph continues, is interrupted, and continues lazily exactly as comrak's own paragraphs do, because it *is* one. `check_open_blocks` keeps the node open while the paragraph is. | Seven lines |
| The primary is inline content | Three existing paragraph conversions (setext heading, table header, link reference definitions) skip the primary's paragraph. | Three changed conditions |
| Never inside code or HTML blocks | comrak doesn't run block starts inside them. | None |

The rest of the upstream changes add the variant to `NodeValue`'s helper methods and the three renderers. The recognizer itself (the keyword lookup and a scan past the attribute block to find the primary) is in Tessera's own file, and so are the parser hooks: a child module of `parser` can add methods to comrak's `Parser` and use its private fields, so `handle_tessera_line` lives in `src/parser/tessera.rs`, not in upstream's 3,000-line `parser/mod.rs`.

### Evidence

- **Spike tests: 22 of 22 pass**, covering every case in task 4 and the edge cases below.
  - A directive line after a paragraph line starts a new block (and a title line above it stays a one-line paragraph for phase 06).
  - `1. Install\n@note` ends the list; `> Quoted\n@note` ends the blockquote.
  - A directive line at a list item's content column belongs to the item, including in nested lists and blockquotes.
  - Four or more spaces past the content column is code, and directly after a paragraph line it's paragraph continuation text; literal either way.
  - `@unknown: text`, `@astrojs/react`, `@timestamp`, `support@example.com`, `@Note:`, `@notes`, `@note-x:`, and `\@note` stay text.
  - `@note {type=caution}: Back up your database` + `before you upgrade.` keeps both lines in the primary.
  - A directive line in a fenced code block, an HTML block, or a code span is code.
  - The primary ends at a blank line, a directive line, a heading, a fence, or a list that can interrupt a paragraph, and not at `2. Not a list`, which can't. It continues lazily in blockquotes and list items, and over an indented line. Its inline content (emphasis, links) is parsed.
  - A line with no primary, a container opener (`@note:`), and identifier primaries (`@include:`, `@available:`) take one line. A project widget with a text primary continues like `@note`.
  - With the option off, or on with no keywords, the tree is exactly CommonMark's.
  - Directive lines don't make a tight list loose. CRLF line endings and tabs work. GFM tables, footnotes, and front matter work alongside.
  - Sourcepos: the node covers `2:3-3:8` for a primary spanning two lines in a list item; the primary's paragraph and its inline text nodes have correct positions.
- **Upstream's own tests: 495 of 495 pass** (490 upstream unit tests, including a sourcepos test extended with a Tessera-line case, plus 5 scanner tests), and 653 of 653 with every vendored feature on.
- **CommonMark 0.31.2** (details in [`tests/commonmark/README.md`](../../tests/commonmark/README.md)):

  | Run | Result |
  |---|---|
  | Unmodified comrak 0.55.0 (phase 00 baseline) | 652/652 |
  | Fork, Tessera option off | 652/652, identical to the baseline |
  | Fork, Tessera option on (built-in keywords, `end`, a project widget) | 652/652; no example renders differently |
  | Fork, option on, each example preceded by an `@end` line | 652/652 |

  There are **no exceptions to justify**: no example in the spec has a line starting with `@`. Because of that, the plain "option on" run is weak evidence on its own, so the fourth run puts a directive line directly above every example. It shows a Tessera line closes cleanly before every kind of block the spec covers, without changing any of them.
- **Performance**: parsing a 7 MB document (20,000 copies of a page full of directives) takes about 120 ms in a release build with the option on or off; the difference is within noise.

## What was hard

- **A `RefCell` panic, caught by upstream's tests.** comrak's tree nodes are `RefCell`s. The first version of the "is this paragraph a text primary?" check borrowed the paragraph's parent during `finalize`, and multiline blockquotes and block directives finalize their children while holding their own borrow, so several upstream tests (for alerts, block directives, and multiline blockquotes) panicked with "already mutably borrowed". The check now uses `try_borrow` (a borrowed parent is never a Tessera line; see `is_text_primary`). The lesson for later phases: code that inspects other nodes during parsing fails at run time, not compile time, so upstream's test suite must keep running on the fork, as it does now.
- **A node that takes its own line.** A Tessera line with no primary must accept its line (or comrak tries to open more blocks inside it) without being marked blank (which would make a list loose). It accepts the line as a heading does, and keeps an unused copy of it in its content.
- **Doctests.** Renaming the crate breaks every upstream doctest, which import `comrak`. They're turned off; upstream's unit tests still run. Tessera's own documentation example is duplicated as a test.
- **Two genuine spec gaps**, raised as questions rather than guessed: Q1 (may a directive line have one to three spaces of extra indentation, as a heading may?) and Q2 (what a setext underline, table delimiter row, or leading link reference definition does to a text primary). Both are implemented in their most conservative reading and tagged `SPEC-QUESTION`.

## How intrusive the change is

| | Lines | Where |
|---|---|---|
| Upstream source files changed | 71 added (44 code, 27 comments), 3 changed, in 19 hunks | 7 files: `lib.rs`, `nodes.rs`, `parser/options.rs`, `parser/mod.rs`, `html.rs`, `cm.rs`, `xml.rs` |
| Upstream test changed | 13 added, 3 hunks | `tests/sourcepos.rs`, which requires a case for every node type |
| Tessera's own files | 345 and 91 lines | `src/tessera.rs` (option, node, scanner, HTML rendering, unit tests), `src/parser/tessera.rs` (parser hooks) |
| Manifest | rewritten | `Cargo.toml` (see `FORK.md`) |

Of the 3,000-line `parser/mod.rs`, the block parser proper changed in six places: a module declaration, one line in the block-start chain, one match arm in `check_open_blocks`, and three one-line guards. Every changed location carries a `// TESSERA:` marker, and `tests/fork_md.rs` fails if `FORK.md`'s table and the markers disagree.

## How merging upstream releases would go

The procedure is in `FORK.md`: diff the fork against pristine upstream to get the Tessera patch, replace the source with the new release, and reapply the patch.

A dry run measured how that patch survives upstream churn, by applying the v0.55.0 patch to older releases (the same distance a merge forward would cover):

| Release | Released | Lines upstream changed between it and v0.55.0, in the six files Tessera hooks into | Patch |
|---|---|---|---|
| v0.54.0 | 2026-07 | 6 | Applies cleanly |
| v0.52.0 | 2026-04 | 487 | Applies cleanly |
| v0.50.0 | 2026-01 | 1,169 | Applies cleanly, **builds with a one-line fix** (a renderer helper, `Context::lf`, that v0.50.0 lacks), and passes all of v0.50.0's unit tests (397, plus the 5 scanner tests) and all 22 spike tests |
| v0.45.0 | 2025-10 | 3,836, including 1,519 in `parser/mod.rs` | 20 of 22 hunks apply; 2 in `nodes.rs` are rejected |

The first version of the patch, which added each new `match` arm at the end next to upstream's newest variant, conflicted with every older release, because upstream appends its own new node types there. Moving Tessera's arms next to long-standing neighbors (`Document`, `FrontMatter`, `Paragraph`) is what made the patch apply cleanly to every release tried from the last eight months. Expect a merge to take an hour or two of mechanical work, plus reading upstream's changelog for new rules that convert a paragraph into something else, which would need the `is_text_primary` guard.

comrak is actively maintained (a minor release roughly monthly) and makes breaking API changes in minor releases. Vendoring insulates Tessera from those until it chooses to merge, and nothing forces frequent merges: the fork only needs upstream for bug fixes and CommonMark spec updates.

## Risks for later phases

- **Positions are line and column, not byte offsets.** comrak's sourcepos is 1-based line and byte column. Phase 05 must convert to byte offsets with its line index, and compute sub-spans (name, attributes, colon, primary) from `NodeTesseraLine::raw`, the node's start column, and `text_primary`. That's straightforward but has edge cases: tabs, CRLF, and blockquote markers inside a multi-line primary. The primary's inline nodes already carry correct positions.
- **The primary is a paragraph in the tree.** Consumers must not treat a Tessera line's child as a block of content (phase 06's structure pass, phase 18's emitters). It's the directive's primary.
- **Head parsing is deliberately minimal here.** The fork only finds where a text primary starts: past an attribute block with quoted strings, and a `:`. A head it can't read (an unclosed `{`, text after the name) gets no primary, so it doesn't continue onto the next line; phase 05 reports it. If phase 05's head grammar ever disagrees with this scanner about where the primary starts, the two must be reconciled; phase 05 should test them against each other.
- **Inline extensions (phase 07)** go into `parser/inlines.rs`, a 2,700-line hand-written inline parser with a byte-dispatch `match`. `{` is already dispatched there for another extension (Phoenix HEEx, off in Tessera), and comrak's `attributes` feature already parses `{…}` after images and links, though with a different grammar (Pandoc style, not Tessera's `key=value, key=value`). Phase 07 has a clear model to follow but should expect a few more changed locations than this phase.
- **comrak's CommonMark renderer isn't Tessera-aware.** It drops the escape in `\@note`, so its output of that text is a directive. Tessera's formatter (phase 23) must escape a line-initial `@keyword` (and `.` title lines and `{key}` phrases) itself if it reuses any of comrak's rendering.
- **Extensions Tessera doesn't use aren't guarded.** With comrak's description-lists extension on, a text primary could still become a description term. Tessera doesn't enable it; if it ever does, it needs the same guard.
- **Spec questions Q1 and Q2** may change recognition or primary continuation. Either change is a few lines in `src/parser/tessera.rs` or at one guard.

## The fallback

markdown-rs is a state-machine CommonMark parser built for MDX. Adding a block construct there means writing a new construct in its tokenizer (states, events, and the resolver), integrating it with its document and flow content types, and teaching its event-to-tree conversion (mdast) about it; its extension points are designed around MDX's needs. It would also need its own fork, with its own merge burden, and phase 00's baseline would have to be redone. Nothing found in this spike makes that cost worth paying.

## Recommendation

**Go: continue with the comrak fork.**

- Every block-level requirement works, including text primaries that continue across lines, and all 22 spike tests pass.
- The CommonMark 0.31.2 suite passes 652/652 with the option off and on, with no exceptions, plus 652/652 with a directive line placed before every example. Upstream's own 490 unit tests pass unchanged.
- The change to upstream is small and confined: 44 lines of code in 7 files, 6 places in the block parser, each marked and listed. Tessera's logic lives in two files of its own.
- The patch applied cleanly to v0.54.0, v0.52.0, and v0.50.0 (eight months of upstream releases), and built and passed every test on v0.50.0 with a one-line fix, so merging upstream is mechanical.
- The remaining risks (position conversion, inline extensions, formatter escaping) belong to later phases and don't depend on the fork decision.
