# Phase 1: The thread

Part of [Embedded help](README.md). Needs nothing.

## Goal

A build writes `_ascribe/help.json`, and a product's build shows a permalink's summary and text from it, with "Read more".

In `examples/quill`, with `prerequisites` and `connect` marked as permalinks:

```console
$ ascribe build --config examples/quill
$ node examples/quill/product/help.ts examples/quill/.ascribe/build/site/site/_ascribe/help.json prerequisites
Prerequisites
Before you install the agent, make sure you have:
Read more: https://docs.quill.dev/go/prerequisites/
```

The entry's `text` has the fragment's list with the phrase `{product}` substituted, and its image as its alt text, "Checklist of prerequisites". `connect`'s entry has no `summary`, since its section opens with a variant group, and its `text` writes the group's arms as the plain output does. A name the file doesn't have prints "No help for <name>" and exits with `1`.

## Context

- README decisions 1 to 13. The proposal's "The design" and "The thread".
- `crates/ascribe-resolve/src/build/permalinks.rs`: `Permalink`, `ResolvedBuild::permalinks`, `ResolvedPage::headings()`, and how a heading's `via` records the includes a fragment's heading came through.
- `crates/ascribe-resolve/src/build/tree.rs`: `ResolvedPage`, `ResolvedBlock`, and how a heading sits inside a container's children (a variant arm, a note).
- `crates/ascribe-emit/src/plain/mod.rs`: `render_page`, `Renderer`, `Renderer::blocks`, `availability_line`, and how a following directive (`@available`) wraps the block after it. `plain/inline.rs`: `link_inline`, `image_inline`.
- `crates/ascribe-emit/src/emitter.rs`: `EmitContext::absolute_url`, `site_origin`.
- `crates/ascribe-emit/src/site/permalinks.rs`: how aliases and retired permalinks are resolved for `permalinks.json`; `help.json` resolves them the same way, and should share the code rather than repeat it.
- `crates/ascribe-emit/tests/all/permalinks.rs` and `support`: `memory_project`, `emit_build`.
- `docs/content/contracts/output-layout.md` §1.2 for the shape of a section about a generated file, and §3 for the manifest.
- `examples/quill/docs/install-agent.md`: `## Prerequisites` (`@id: prerequisites`, an `@include` of `_fragments/prerequisites.md` with a phrase and an image) and `## Connect to {product}` (`@id: connect`, a variant group of two arms).

## Design

### The section

`ascribe_resolve` gains `Permalink`'s section: for a heading's permalink, the blocks after its heading to the next heading at its level or above, or to the end of the container the heading is in (decision 6), found from the heading block's place in its page; for a page's own permalink, the page's blocks before its first heading (decision 2). It returns borrowed slices or indexes into the resolved page, and copies nothing. A permalink in a fragment is cut from the page `permalinks()` records for it (decision 7).

### The entry

In `crates/ascribe-emit/src/site/help.rs` (decision 12):

- **`text`:** the section's blocks through the plain renderer's block loop, with images written as their alt text. The renderer gains a `pub(crate)` entry that takes a page context and a slice of blocks and returns the chunks, and a switch, set for help only, under which `image_inline` writes the alt text. The page's own plain output is unchanged.
- **`summary`:** decision 10 for a heading, passing over a `@memo`, an `@intended`, or an `@available` before the first block, and taking a paragraph `@available` wraps without its availability line; the page's description, else its opening's first paragraph, for a page (decision 2).
- **`title`:** the heading's text as plain text, or the page's title.
- **`address`:** the permalink's address, from `[consumer] site`, the permalink path, and the name, ending as `trailing-slash` says. Absent without `site`.
- **`source`:** as `permalinks.json`'s.
- **Aliases and retired permalinks:** as `permalinks.json` resolves them (decision 3), with that code shared.

The file is `version`, `build`, `line` when the build is a line of a versioned build, and `help`, sorted by name. The site emitter writes it beside `permalinks.json`, `generated` in its manifest.

### The agreement test

Decision 11: for every project under `examples/` and `tests/conformance/cases/`, every build, and every entry, the entry's `text`, with its images read back as `![alt](…)`, is found in its page's plain output, in order. An empty `text` would pass that, so for quill's `prerequisites` and `connect` the test also runs the other way: the page's plain output from after the heading to the section's end, images read the same way, is the entry's `text` exactly. A section that can't be rendered alone and agree is the stop-and-report case.

### The product

`examples/quill/product/help.ts`: a few lines of TypeScript, run with Node's type stripping, which the Node 24 the engines name has, so it's `.ts` with no build step, that read the file, look up a name, and print its title, summary, and "Read more: " and its address, or "No help for <name>" with exit code `1`. It reads the file and nothing else: no package, no Ascribe. `ascribe.examples.toml` lists it and the test that runs it, and its observation is written again.

### The docs' own file

Build `docs/`'s file and record in the pull request, for its permalinks: how many have a summary, the median and the longest, how many are over 300 characters, how many have none, and that each diagnostic's is its metadata line. Phase 3 and phase 4 read it.

## Tasks

1. The section, with tests in `ascribe-resolve`: a heading's section ending at a heading of its level, at a higher one, at the end of a variant arm, and at the end of the page; a page's opening; a fragment's heading cut from the recorded page.
2. The renderer's entry for a slice of blocks, and the alt-text switch, with the page's plain output byte for byte the same (`node scripts/compare/outputs.ts --base main` shows only `help.json` and the manifests).
3. `help.rs`: the entry and the file, with `crates/ascribe-emit/tests/all/help.rs` covering a heading's entry, a page's, a same-page link made absolute, a link left a route without `site`, an include, a phrase, an image's alt text, `@available` before the first paragraph, a section with no paragraph first, an alias, a retired permalink, a fragment's heading, `line` in a versioned build's lines and its absence otherwise, and an empty table.
4. The agreement test.
5. `@id {permalink=true}` on quill's `prerequisites` and `connect`. Quill is the project behind many tested outputs and the docs' snippets, so the change reaches past its own build: run the CLI's output tests, the conformance suite, and the emit tests, and read every changed file. Expect at least `ascribe outline`'s output for the page, `ascribe permalinks list`'s, `site_quill.rs`'s permalinks, and any `search` or `drift` output that quotes the headings' lines (`crates/ascribe-cli/tests/output/`: `outline-memos.txt`, `search.txt`, `search.json`, `search-fragment.txt`, `drift.txt`, `drift-guide.txt` name these sections today). Bless each that changes as its file says, observe the examples again (`cargo run -p ascribe-cli -- observe`), and name the build outputs' changes with `--accept`. If quill keeps an `ascribe.permalinks`, sync it.
6. The product program, its test, and `ascribe.examples.toml`, observed again.
7. The docs' own file built, and the measurements in the pull request.
8. The docs: a page, `docs/content/permalinks/in-your-product.md` ("Show the docs in your product"), `@available: ascribe planned`, `@id {permalink=true}` on it: what the file holds, the program by `@snippet`, bundling from the product's own line's build, setting `[consumer] site`, and checking names with `ascribe permalinks verify` as the permalinks guide does; `ascribe.nav` places it; the output-layout contract's §1.7; a decision file for §1.7; `changes/language/embedded-help.md`.

## Out of scope

Everything in the README's "Left out" table. A summary longer than a tooltip is written as it is.

## Acceptance criteria

- The goal's commands, in a test.
- The agreement test passes for every project in the repository.
- `node scripts/compare/outputs.ts --base main` changes only `help.json`, the manifests, and quill's `permalinks.json`, each named with `--accept`; every tested output task 5 found changed is blessed, its diff read, and `examples.observations/default.toml` observed again.
- The agreement test passes both ways for quill's two sections.
- The measurements of the docs' file are in the pull request.

## Verify

```sh
cargo test -p ascribe-resolve section
cargo test -p ascribe-emit help
cargo run -p ascribe-cli -- build --config examples/quill
node examples/quill/product/help.ts examples/quill/.ascribe/build/site/site/_ascribe/help.json prerequisites
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Find a permalink's section in a resolved page"
2. "Render a slice of a page's blocks as the plain output does"
3. "Write each permalink's section to _ascribe/help.json"
4. "Show the docs in a product: quill's help program and its page"
