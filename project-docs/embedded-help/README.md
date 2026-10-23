# Embedded help

A product shows a section of its docs inside itself, in a tooltip, a side panel, or beside an error, by the permalink it already links to. The text is the docs' own, for the product's release line, and a product that asks for a name the docs don't have fails its own CI. The idea, the design, and the three questions the maintainer settled are in the [proposal](../proposals/embedded-help.md). This is the plan for building it.

## Who it's for

- **A product team** that links its interface and its error messages to the docs by permalink, and wants the first paragraph of the answer where the user is, without a browser.
- **A docs team** whose words then reach the product as written, checked, and for the right version, with nothing copied into the product's strings to drift.
- **An agent** writing a product's error message, which can read what the docs will show for a permalink before it names one (phase 7).

## What exists today

The proposal's "What exists" holds. The pieces this plan stands on, checked against `main` on 22 October:

- **Permalinks in a build:** `ResolvedBuild::permalinks` in `crates/ascribe-resolve/src/build/permalinks.rs`, each a `Permalink` with its `name`, `page`, `route`, `anchor` (the heading's page id, `None` for a page's own), `at`, and `via` (the includes a fragment's heading came through). Headings come from `ResolvedPage::headings()`, which yields each heading's block, wherever it's nested.
- **The permalinks list:** `crates/ascribe-emit/src/site/permalinks.rs` writes `_ascribe/permalinks.json` in the site output (output-layout contract §1.2), with aliases and retired permalinks resolved, a name the build publishes twice listed with its first page. `crates/ascribe-emit/tests/all/permalinks.rs` holds it.
- **The plain renderer:** `crates/ascribe-emit/src/plain/mod.rs`, `Renderer::blocks` over a slice of `ResolvedBlock`s, which `render_page` calls with a page's blocks after its title and availability line. Links to pages go through `EmitContext::absolute_url` (`crates/ascribe-emit/src/emitter.rs`), which prefixes `[consumer] site`'s origin to a route and leaves it a route without one. An image is `image_inline` in `plain/inline.rs`, written as `![alt](address)`.
- **The headers:** `crates/ascribe-emit/src/site/headers.rs` writes `_ascribe/headers.json` (§1.4), which `[consumer] headers = "netlify"` turns into `_headers`.
- **Astro's permalink addresses:** `packages/astro/src/permalinks.ts` reads `permalinks.json` and `Permalink.astro` serves `go/<name>/`, for the current line only.
- **The diagnostics reference:** `tests/conformance/tests/docs.rs` writes `docs/content/_generated/diagnostics-*.md` from `tests/conformance/diagnostics.toml`. Each diagnostic's section is its heading, `@id {permalink=true}`, a metadata line, then **When**, **Message**, **Fix**, and **Read more** paragraphs.
- **The editor:** This page's Permalink row (`packages/vscode/src/ui/sidebar.ts`) lists each permalink a page declares. The language server's hover (`crates/ascribe-lsp/src/hover.rs`) has nothing for a permalink yet.
- **The docs' permalinks:** `docs/ascribe.permalinks` has 328, most of them diagnostics. A review of the proposal measured the 300 heading permalinks with the first-paragraph rule: a median summary of 94 characters, 34 over 300, 3 sections with no paragraph first, and every diagnostic's summary its metadata line.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

From the proposal's Settled list, answered by the maintainer on 22 October 2026:

1. **The file is `_ascribe/help.json` in the site output's root,** beside `permalinks.json`, written by every build.
2. **A page's permalink gives its description as the summary and its opening as the text.** A page with no description takes its opening's first paragraph as the summary; a page whose opening is empty has an empty `text` and the summary alone.
3. **Every permalink is in the file:** an alias under its own name with its target's entry, a retired permalink left out.

From the proposal's design:

4. **The format, version 1:** `version`, `build`, `line` (absent for a build that isn't versioned), and `help`, keyed by name, each entry with `title`, `summary` (absent when there's none), `text`, `address` (absent without `[consumer] site`), and `source`. A reader rejects a version it doesn't know; a later version adds keys and changes none of these. `title` is plain text on purpose, code spans and all, since a tooltip's title is text; a formatted form, as `pages.json` has, would be a key added beside it.
5. **`text` is the plain output's Markdown** for the section, and an image in it is its alt text, since a product showing a bundled copy offline can't load an image from the docs' host.
6. **A heading's section** runs from the block after the heading to the next heading at its level or above, or to the end of the container the heading is in (a variant arm, a note), whichever comes first. The heading itself isn't in `text`; it's the entry's `title`.
7. **A permalink in a fragment** follows the one record `permalinks.json` keeps: its `source` and `address` are that record's, and its section is cut from the page the record names.
8. **Availability is text, as the build has it.** A build that shows badges writes each mark in the section as an "Available: …" line. A product bundles the file from its own line's build, which `line` lets it check.
9. **No setting.** The file is written for every build, with an empty table when the build publishes no permalink.

From this plan:

10. **The summary, until phase 3:** the section's first block of content when it's a paragraph, as the plain output writes it; absent otherwise. A `@memo`, an `@intended`, or an `@available` between the heading and that block is passed over, and the availability line `@available` adds isn't in the summary. Phase 3 replaces this with the section's own `@summary`, and from then on nothing is guessed from the prose: `summary` is the section's summary, or absent. The key doesn't change.
11. **The section is rendered, not cut.** An entry's `text` comes from the plain renderer's own block loop (`Renderer::blocks`) over the section's blocks, so a section can't read differently in the file and on the page. A test holds every entry's `text` to its page's plain output: found there, in order, once images are read as their alt text. Since an empty `text` passes that, the thread's two sections are also held the other way: the page's plain output between the heading and the section's end is the entry's `text`. If a section can't be rendered alone and agree, stop and report: the fallback is cutting the plain output's text by its headings, which changes how decision 10 finds the summary.
12. **The file is the site emitter's,** in `crates/ascribe-emit/src/site/help.rs` beside `permalinks.rs`, calling the plain renderer through a `pub(crate)` entry. It's listed `generated` in the site output's manifest.
13. **Without `[consumer] site`,** links in `text` stay routes from the host's root and `address` is absent. The docs page says a project whose product shows its help sets `site`; nothing reports it, as nothing reports `permalinks.json`'s routes.
14. **The section summary is a decision of its own, before phase 3** (phase 2). Phase 1's measurements showed no rule over the prose picks well, so a summary is something a writer or an agent decides. The recommendation there: `@summary`, general to every section, with `@note`'s two bindings: `@summary: text` gives its own text, which the page's body doesn't show; `@summary` alone makes the next block the summary, rendered where it is. Embedded help is its first reader, and a check asks for one only on a permalink, as advice unless a project raises it. Nothing before phase 3 depends on its syntax.
15. **The CLI surface is the proposal's,** `ascribe resolve go:<name>` and `ascribe_resolve` with the same, in phase 7. A name after `go:` is a permalink, never a page path, since a page path can't start with `go:` and a colon.

The contracts, and the phase that records each in `project-docs/decisions/`: the file and its format, output-layout contract §1.7 (phase 1); the section summary, SPEC §3, §4, and Appendix A, what each output writes for it (`site-render.md`, with review's source anchors), and the JSON output's shape for it (phases 2 and 3); the checks, `diagnostics.toml` (phase 4); the per-line addresses, `@ascribed/astro`'s reference (phase 5); the arms' key, §1.7 (phase 6); `resolve`'s `go:` and its JSON (phase 7).

## The thread

**The use case:** a product's build copies `help.json` from the docs' build, and shows the `weave-config` section's summary in a tooltip, with "Read more" opening the permalink's address. In this repository the product is a short TypeScript program beside `examples/quill`, run by a test, which the docs page shows.

**The layers it crosses:**

| Layer | Where | What the thread needs from it |
|---|---|---|
| Resolve | `ascribe-resolve` | Each permalink's section: its heading block, and the blocks to the next heading at its level or the end of its container (decision 6) |
| Emit | `ascribe-emit`'s site and plain emitters | The section rendered by the plain renderer, its summary, the entry, and the file in the site output and its manifest |
| The command | `ascribe-cli` | `ascribe build` writes it, with nothing new to say |
| The contract | `docs/content/contracts/output-layout.md` | §1.7, the file and its format |
| The docs | `docs/content/permalinks/` | A page: show the docs in your product, with the program and the product's CI check |

**The seam it tests** is a section rendered outside its page and agreeing with it: a link to another heading on the same page made the page's address and that heading, an include inside the section resolved, a switcher's arms as the plain output writes them, an image as its alt text, availability as its line, and nothing of the page around it. Decision 11's test holds the agreement for every permalink of every project in the repository. If it fails, the design changes to cutting the plain output's text, and the summary rule with it, before a product reads the file.

The thread also builds Ascribe's own docs' file and records what the first-paragraph rule picks for their 328 permalinks: the lengths, the sections with no summary, and the diagnostics' metadata lines. That's phase 3's and phase 4's input.

**Left out, and where it comes back:**

| Left out | Phase |
|---|---|
| A section's own summary, `@summary`, and the diagnostics' `when` as theirs | 2 (the decision), 3 |
| Checks for a missing or long summary, seeing a summary in the editor, and agents told to write them | 4 |
| Fetching an entry for the product's line from the docs site | 5 |
| A product choosing its reader's arm of a switcher | 6 |
| Reading an entry from the command line and the MCP server | 7 |

## Phases

| Phase | Result |
|---|---|
| [1: The thread](phase-1-thread.md) | A build writes `_ascribe/help.json`, and a product's build shows a permalink's summary and text from it, with "Read more". |
| [2: Decide the section summary](phase-2-marker-decision.md) | The maintainer settles how a section has a summary. |
| [3: Section summaries](phase-3-marked-summary.md) | A writer gives any section a summary with `@summary`, its own text or a block on the page, and the docs' diagnostics have theirs. |
| [4: Summaries checked and shown](phase-4-sized.md) | `ascribe check` reports a permalink with no summary and a summary too long, the editor shows a section's summary, and agents are told to write them. |
| [5: Fetched for its line](phase-5-fetched.md) | `@ascribed/astro` serves each entry of each line's `help.json` at its own address, and a product fetches its line's, falling back to its bundled copy. |
| [6: The reader's arm](phase-6-arms.md) | A product shows a section's `pnpm` arm to a reader it knows uses pnpm. |
| [7: For agents](phase-7-agents.md) | `ascribe resolve go:weave-config` and `ascribe_resolve` show an entry. |

Phase 1 is the thread, through the riskiest seam. After it, the order is by risk: the section summary is a language change and the summary's design, so its decision and its build come next, while phase 1's measurements are fresh; the check and the editor follow, once a summary can be marked; phase 5 adds a second contract, addresses on a host, and meets versioned builds; phase 6 adds a key products ask for after using phase 1; phase 7 is a surface over what exists.

Every phase's result is something a person or an agent can use, but phase 2's: a decision, which comes before phase 3 because phase 3 ships a contract.

## Carved out, and waiting

- **A component that shows the text,** for any framework. A product renders Markdown its own way.
- **Text written only for the product.** It's the product's own string; embedded help is the docs' text, or it can't promise the two agree.
- **Translation,** which would be a build per language, each writing its own file.
- **A smaller file,** with summaries only, if bundling the whole text turns out to cost a product; phase 5's entry at a time is the answer for now.

## The other plans

None is running. The [CI proposal](../proposals/ci-release-testing.md)'s Step 4 waits on its own data and touches nothing here.

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- **Outputs change only as each phase says.** Phase 1 adds `help.json` to every build's site output and its manifest, and nothing else; each later phase names what it adds. Run `node scripts/compare/outputs.ts --base main`, and name each changed file with `--accept`, labeled `outputs changed`, with the block CONTRIBUTING.md asks for.
- **Contracts change only by a recorded decision,** in the phase that changes them, as the README's list says.
- **Libraries don't print, and don't panic on input.** The section, the summary, and the entry are returned; only `ascribe-cli` writes. Project files are read through `ascribe_resolve::FileSystem`.
- **Behavior changes come with tests at the layer that proves them** ([writing-tests](../../.claude/skills/writing-tests/SKILL.md)): the file in `crates/ascribe-emit/tests/all/`, as `permalinks.json` is; a directive and a diagnostic in `tests/conformance/cases/`; the editor's rows and hovers in the language server's and the extension's tests; Astro's addresses in `packages/astro`'s.
- **The program the docs show is a tested example:** listed in `ascribe.examples.toml`, run by a test, shown by `@snippet`, and observed again (`cargo run -p ascribe-cli -- observe`).
- **The map of the surface** is regenerated when a phase changes it (`node scripts/surface/inventory.ts`), and its diff read against the conventions.
- **User-visible changes** update their page under `docs/content/`, marked `@available: ascribe planned` where they're new, and add an entry in `changes/` in the same phase. Each phase file names its pages and its area.
- No phase history in code or docs. Describe what the code does now.
- Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  ```

## Later, not in this plan

- Everything in the proposal's "What this leaves out".
- Showing the help in Ascribe's own editor for a diagnostic in a user's page: the binary already carries its text (`ascribe explain`), so the editor needs no file for it. Phase 4's hover is a different use: a writer, on a permalink's `@id` line in the docs' source, sees what a product will get.
