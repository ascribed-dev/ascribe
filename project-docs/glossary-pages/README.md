# Glossary pages

A glossary's terms are written where the rest of the docs are written: on a page, in Markdown, by whoever writes the pages, and each definition once. A term is a heading marked `@term`, its definition is the section's first paragraph, and `[glossary] pages` names the pages that hold terms. `[glossary.terms]` goes. The idea is in the [proposal](../proposals/glossary-pages.md). This is the plan for building it.

## Who it's for

- **A writer who keeps a glossary,** who today writes each definition twice, once in `ascribe.toml` and once on the page a term links to, and keeps the two in step by hand.
- **A team whose glossary is long,** whose `ascribe.toml` becomes mostly definitions, owned by people who shouldn't need to touch the rest of it.
- **A product with versions or variants,** whose definitions can't be marked with `@available` or use a phrase, because they're strings.
- **A reader,** who follows a term in Ascribe's own docs and lands partway down a reference page instead of on a definition.

## What exists today

- **`[glossary]` in `ascribe.toml`** has `match`, `case-sensitive`, and `terms`. A term has `term`, `aliases`, `definition`, `link`, and its own `match` and `case-sensitive`. They're `Glossary` and `GlossaryTerm` in `crates/ascribe-model/src/model.rs`, loaded in `sections.rs`.
- **Every reader takes the terms from the content model:** linking in `crates/ascribe-resolve/src/build/glossary.rs` (SPEC §5.4, §9.2 step 7); the uses in `uses.rs`; the editor's incremental signature in `incremental/signature.rs`; `glossary-term-unused` in `crates/ascribe-check/src/glossary.rs` and `across.rs`; Vale's vocabulary in `prose/setup.rs`; acknowledgements in `crates/ascribe-model/src/intended.rs`; `ascribe refs`'s `term:` targets and `ascribe model`'s glossary section, in `crates/ascribe-query/src/refs.rs` and `model.rs`; and in the language server, `definition.rs`, `inventory.rs`, `edit/model.rs`, and `model_file.rs`.
- **What a term puts in the outputs:** the site output's link carries `data-ascribe-term` with the definition as its title (`docs/content/contracts/site-render.md`, and `packages/elements/CONTRACT.md` §7); each block in the JSON output lists its terms, each with its id and definition (`crates/ascribe-emit/src/json.rs`); `ascribe model` lists each term with its `definition` (`crates/ascribe-query/src/model.rs`).
- **Four diagnostics are about terms:** `model-glossary-duplicate-term` (ASC098) and `model-glossary-link` (ASC099), errors in `ascribe.toml`; `glossary-term-unused` (ASC158), advice, acknowledged by `[[intended]] term`; and `glossary-link-target-missing` (ASC203), an error added by [decision #468](../decisions/468-glossary-link-heading.md), when a term's `link` names a heading its page doesn't have.
- **Renaming a heading's id** rewrites a term's `link` that names it, from `ascribe rename` and from the editor (decision #468).
- **Five projects in the repository have a glossary.** `docs/` has 8 terms and no glossary page: each links to the heading in the reference or the guide that explains it. `examples/monorepo/docs` has 3 terms and a glossary page, `content/reference/glossary.md`, with a heading and an `@id` for each and a paragraph that repeats the TOML's definition almost word for word. `examples/monorepo/handbook/pages/security` and `examples/content-models` (`full.toml`, with `docs/reference/glossary.md`) have terms too, and so do five conformance cases in `tests/conformance/cases/glossary/` and one in `checks/model-entries-unused`.
- **Every page names its type** (decision #450), so a glossary page does too.
- **Ascribe's docs are grouped by feature** (decision #452). The proposal put their glossary at `reference/glossary.md`; there's no `reference/` folder any more.
- **Nothing teaches the glossary.** The reference describes the table, and no guide sets one up.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

From the proposal:

1. **A term is a heading marked with `@term`,** a line directive at the top of the section, as `@id` and `@available` are there. A heading without it is structure.
2. **The term is the heading's text, its link is the heading, its definition is the section's first paragraph, and its id is the heading's id.** The id is what `data-ascribe-term` carries and what names the term everywhere else.
3. **`@term` takes no primary, and three attributes, each only when needed:** `aliases`, a string split on `|` with spaces around each trimmed, quoted when it holds a space; `match`, `first`, `every`, or `marked`; and `case-sensitive`.
4. **`[glossary] pages` is a list of paths from the content root,** each a page. `match` and `case-sensitive` stay, as what every term starts from. A project with no `[glossary]`, or no `pages`, has no glossary.
5. **`terms` goes,** in the release that adds `@term`, with no period where both work. A project that still has it gets an error that names `pages` and `@term`. The changelog says **Behavior change**.
6. **`@term` on a page that isn't in `pages` is an error,** with a fix that adds the page. A restriction chosen for now, not a limit of the design.
7. **A glossary page is an ordinary page otherwise:** a type, frontmatter, a route, fragments, `@available`, and variants.
8. **The short definition is derived:** the first paragraph as plain text, phrases substituted, formatting dropped, links reduced to their words, for each build. The link's title, the editor's hover, and the JSON output take it.
9. **Errors move to the page.** Two terms or aliases with the same text is an error at the second, naming the first. A term with no paragraph is an error. A definition longer than `description-too-long`'s limit is advice, counted on the derived text: one number for both, since both are a sentence shown where there's no room for more. That limit is a constant today, 300 characters (`DESCRIPTION_LIMIT` in `crates/ascribe-resolve/src/llms.rs`), so whether either gets a setting is phase 1's.
10. **`model-glossary-link` goes,** and a term with nowhere to link goes with it: every term is on its page.
11. **A term isn't linked on the page that defines it,** and matching never applies to headings.
12. **The site output's contract doesn't change:** `data-ascribe-term`, with the definition as the link's title.
13. **The repository's projects are moved by hand** in the pull request that makes the change. No command does it.
14. **Ascribe's docs get a glossary page:** a section for each of their terms, a sentence or two and a link on to the page that documents it.

[Phase 1](phase-1-decisions.md) settles six more, each a contract the thread ships: what exactly a definition is, which page a term links to when two glossary pages show it, how `glossary-term-unused` is acknowledged, what `ascribe model` says about a term, the diagnostics' names and levels, and where Ascribe's docs keep their glossary.

## The thread

**The use case:** `examples/monorepo/docs` lists `content/reference/glossary.md` in `[glossary] pages`, marks its three headings with `@term`, and drops its table. `ascribe build` links "rollouts" on the getting started page to the glossary's section, with the section's first paragraph as the link's title. In VS Code, rewording that paragraph changes the hover on "rollouts" in another page, without saving or restarting.

**Why it can't be narrower.** Decision 5 leaves no release with both forms, and a release can be made from `main` on any day. So the pull request that adds `@term` removes `terms`, and every reader of the model's terms (the list under "What exists today") takes them from the pages in the same change, or loses them. The thread keeps what each reader does as it is; making them better is the later phases'.

**The layers it crosses:**

| Layer | Where | What the thread needs from it |
|---|---|---|
| The language | `SPEC.md`, `ascribe-core`'s schema | `@term` and its attributes |
| The content model | `ascribe-model` | `pages`, and `terms` an error |
| Resolve | `ascribe-resolve` | The terms read from the glossary's pages before any page is linked, and each definition for each build; the editor's incremental signature |
| Check | `ascribe-check` | `glossary-term-unused`, Vale's vocabulary, and acknowledgements, from the pages' terms |
| Emit | `ascribe-emit` | The derived definition as the title and in the JSON output |
| Query | `ascribe-query`, `ascribe-mcp` | `ascribe model`'s terms |
| The editor | `ascribe-lsp` | Hover and the inventory from the pages' terms |
| The docs | `docs/`, `examples/`, `tests/conformance/` | Every project moved, and the reference |

**The seam it tests** is terms read from content before content is resolved. Matching needs every term before any page is linked, so the glossary's pages are read first, and a definition is resolved for each build, since a phrase or a section's availability can change it. The source index holds every page's headings and directives already. If it can't give the terms without a second parse of the glossary's pages, or if the editor has to resolve every page again whenever a glossary page changes, terms become a separate pass with a cache of its own, and the editor's latency budget for a keystroke is at risk.

**Left out, and where it comes back:**

| Left out | Phase |
|---|---|
| A term with no definition, a long definition, `@term` off a glossary page, and a term in a build that doesn't publish its section | 3 |
| Go to definition, completion, the quick fixes, rename, and the outline | 4 |
| A guide to the glossary | 5 |

After phase 2, a term with no paragraph has an empty definition, a long one is used whole, and `@term` on a page that isn't in `pages` is ignored. Two terms with the same text are an error from the start, since matching can't choose between them. The reference says what's true at each step.

## Phases

| Phase | Result |
|---|---|
| [1: Decisions](phase-1-decisions.md) | The maintainer settles the six contracts the phases ship. |
| [2: The thread](phase-2-thread.md) | A glossary is a page: its terms are headings, its definitions are paragraphs, and every project in the repository has moved. |
| [3: A glossary page's rules](phase-3-rules.md) | An empty definition, a long one, and a term off a glossary page are each reported where they're written, and a term follows its section's availability. |
| [4: The editor](phase-4-editor.md) | A term goes to its section, `@term` completes, a misplaced one has a fix, and renaming a heading keeps everything that names its term. |
| [5: A guide](phase-5-guide.md) | A writer can set up a glossary from one page. |

Phase 1 comes first because the thread ships a directive, the model's shape, and two commands' JSON. Phase 2 is the thread, and is wider than a thread usually is, for the reason above. After it the order is by value: phase 3 makes the page's mistakes visible where they're made, phase 4 the editor, and phase 5 teaches it.

Each phase's result is something a person can run or open.

## Carved out, and waiting

- **`@term` on any page** (decision 6). A project that wants a term on the reference page that documents it lists that page in `pages`.
- **A generated glossary page,** or a directive that lists every term.
- **A term defined by a definition list or a table row.**
- **Splitting `ascribe.toml` into several files.**
- **A popover in the site.** A site gets the definition as the link's title, as now.

## The other plans

- **Versions.** A term's section marked `@available: self-managed 3.5` is a term the 3.4 line doesn't have, which this plan makes possible and phase 3 tests in a filtered build. The versions plan needs nothing more from it.
- **Memos.** A memo about `term:<id>` names the heading's id, which is the term's id under decision 2, so it's unaffected.
- **Navigation.** A glossary page is a page in its folder's section, as any page is.

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- **Outputs change only where a recorded decision says.** Every phase runs `node scripts/compare/outputs.ts --base main`, and each difference is named with `--accept`, file by file. Phase 2's are the projects whose terms moved.
- **The language and the contracts change only by a recorded decision.** Go through the matching list in `project-docs/checklists.md` first, and record the decision in `project-docs/decisions/` in the same pull request.
- **A new diagnostic** is added to `tests/conformance/diagnostics.toml` and `crates/ascribe-core/src/diagnostics.rs`, in the same order, with a conformance case. A diagnostic that's removed leaves a gap in the codes (decision #457a).
- **Behavior changes come with a conformance case** in `tests/conformance/cases/`.
- **Project files are read through `ascribe_resolve::FileSystem`.** Libraries don't print, and don't panic on input.
- **The map of the surface** is regenerated when a phase changes it (`node scripts/surface/inventory.ts`), and its diff is read against the conventions.
- **User-visible changes** update the page under `docs/content/` and add an entry in `changes/` in the same phase.
- No phase history in code or docs. Describe what the code does now.
- Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  ```

## Later, not in this plan

- Glossaries in more than one language.
- A command that moves a project's terms.
