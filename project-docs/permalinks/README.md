# Permalinks

Something outside the docs links to a page or a section by a name that never changes, and Ascribe keeps that link working while the docs are renamed, moved, and rewritten. The idea, and the research behind it, are in the [proposal](../permalinks.md). This is the plan for building it.

## Who it's for

- **A product's developers,** whose code links to its docs: a help link in an interface, an error message that says where to read more, a command's `--help`.
- **Docs authors,** who can then move and rename pages without breaking those links, and are told when a change would.
- **Ascribe itself.** Its binary is the first product to use it.

## What exists today

- **A page's identity is its file's path,** and its address is computed from it. A heading can have an `@id` (SPEC §4.1), unique within its page, which is also its anchor. `@id` takes no attributes (`Builtin::Id` in `crates/ascribe-core/src/schema.rs`).
- **Three frontmatter keys are reserved** for pages: `available`, `variant`, and `formatted`, with `slug` under the `astro` profile (`crates/ascribe-model/src/sections.rs`).
- **The binary builds full addresses into the docs.** `ascribe_core::docs_site!` holds the site, and three places add a path and an anchor to it:
  - each command's `--help` (`docs_page!` in `crates/ascribe-cli/src/cli.rs`): `…/reference/cli/#ascribe-check`;
  - each diagnostic (`Entry::docs` in `crates/ascribe-check/src/registry.rs`): `…/reference/diagnostics/#asc036-link-target-missing`, in `check`'s JSON, `explain`, and the editor;
  - the agents' rules and the plugin's homepage (`crates/ascribe-query/src/rules.rs`, `crates/ascribe-cli/src/agents/plugin.rs`).
- **The diagnostics reference's headings are generated** from the registry into fragments (`docs/content/_generated/diagnostics-*.md`) that the reference page includes.
- **Each build's output is complete on its own,** with `_ascribe/` reserved in each emitter root for what Ascribe places there (`docs/content/contracts/output-layout.md`).
- **`@ascribed/astro` adds no routes.** It loads the site output as a collection, serves `_ascribe/files/`, and the site writes its own route for pages.
- **One file follows the content today:** `ascribe.lock`, read by `ascribe-model` and written by `ascribe-sources`. `check` says when it's behind.
- **The docs site installs `@ascribed/astro` from npm's `next` tag,** so it gets a change to the integration the night after it merges.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

1. **Declared in the content, on what it names.** A heading: `@id {permalink=true}: rotate-keys`. A page: `permalink: cli-reference` in its frontmatter. Nothing is a permalink unless marked.
2. **The permalink is the id.** A marked heading's `@id` is its anchor and its permalink, one name for one thing.
3. **Addresses are pages, not a server.** Each permalink is published as a small page that sends the reader on, under a path the project sets (`go/` under its base path, unless it says otherwise). It works on any static host.
4. **The build writes a list;** a site draws the addresses from it. `@ascribed/astro` does that. Another generator gets the file.
5. **Not a second way to link inside a project.** Pages link to files. A permalink is for what's outside.
6. **Removal is deliberate and recorded, never blocked.** Removing a permalink without retiring it is a warning. Retiring one takes a reason.
7. **A committed list is how removal is noticed,** and what a product checks its ids against. It's written by Ascribe and follows the content, as `ascribe.lock` does.
8. **Ascribe hosts nothing and the binary stays as it is:** no HTTP, no server.
9. **No work is done for released versions.** Ascribe is before 1.0 and has no users, so the addresses in 0.2.0's binary aren't kept working.

Three more are made in phase 1, because each fixes a shape that ships in phase 2 and can't change afterwards.

## The thread

**The use case:** `ascribe check --help` ends with a link to `/go/cli-check`, and following it on the docs site opens the section about `ascribe check`.

**The layers it crosses:**

| Layer | Where | What the thread needs from it |
|---|---|---|
| The language | `ascribe-core` (the `@id` schema), `SPEC.md` | `@id` takes `permalink=true` |
| Resolve | `ascribe-resolve` | Each build's permalinks, with the page and anchor each leads to |
| Check | `ascribe-check`, the registry | One name used twice in a build is an error |
| Emit | `ascribe-emit` | The list, in the site output |
| The consumer | `@ascribed/astro` | A page at `/go/<id>` for each entry |
| The docs | `docs/content/reference/cli.md` | One heading, marked |
| The product | `ascribe-cli` | One help link built from an id |

**The seam it tests** is between the build and the site: a list Ascribe writes becoming pages a site generator serves, at an address the binary can know without knowing the site's layout. The integration has never added a route. If it can't do it well, in `astro build` and `astro dev`, with a base path, on a static host, with the anchor kept, the alternative is the emitter writing the forwarding pages itself as files, which changes what the output contract has to say. That's worth knowing before anything is built on the list.

**Left out, and where it comes back:**

| Left out | Phase |
|---|---|
| Permalinks on pages; the rest of the binary's links | 3 |
| The committed list, noticing a removal, `ascribe permalinks verify` | 4 |
| Retiring, aliases, the editor's fixes | 5 |
| What a link carries; a page for a name nobody declared | 6 |

Until phase 4, nothing notices a permalink being removed. The directive reference says so while that's true.

## Phases

| Phase | Result |
|---|---|
| [1: Three decisions](phase-1-decisions.md) | The maintainer settles what a name may look like, how far it must be unique, and where one may be declared. |
| [2: The thread](phase-2-thread.md) | A heading marked as a permalink has an address on the site, and `ascribe check --help` links to one. |
| [3: Pages, and every link Ascribe makes](phase-3-own-links.md) | A page can have a permalink. Every address the binary builds is an id, so the docs can move. |
| [4: The list](phase-4-list.md) | `ascribe.permalinks` is committed, `check` reports a permalink that went away, and a product's CI can check its ids. |
| [5: Retiring and aliases](phase-5-retire.md) | A permalink can be retired with a reason, alone or many at once, and one place can have several names. |
| [6: What a link carries](phase-6-context.md) | A link can say which variant its reader wants, and a name nobody declared gets a helpful page. |

Phase 1 comes first because phase 2 ships contracts. After phase 2 the order is by risk, then by what each unblocks: phase 3 is what the [website](../website.md) waits for, and phase 4 is the last new mechanism.

Each phase's result is something a person can run or open. None builds a layer for a later phase to use.

## The other plans

- **The website** needs phase 3, and nothing after it. Once the binary links by id, the docs can move to `/docs/`.
- **Versions.** Phase 1's second decision says whether one permalink may be on two pages that no build publishes together, which the [versions proposal](../versions.md) would use in place of `replaces`. Phase 6 leaves versions out of what a link carries until that work exists.
- **Content checks.** This plan's warnings are ordinary warnings. When the [content checks plan](../content-checks/README.md)'s contract lands, they take a kind of next step like every other diagnostic. Retiring's required reason is the same rule as that plan's acknowledgements, and should read the same.

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- **A new diagnostic** is added to `tests/conformance/diagnostics.toml` and `crates/ascribe-core/src/diagnostics.rs`, in the same order, with a conformance case in `tests/conformance/cases/`.
- **The language and the contracts change only by a recorded decision.** Every phase from 2 on changes at least one. Go through the matching list in `project-docs/checklists.md` first, and add the decision to `project-docs/decisions.md` in the same pull request.
- **A contract's shape doesn't change after it ships.** A later phase may add to the list's format, the model's tables, or a command's JSON. It may not change what an earlier phase wrote. If a phase finds it must, stop and report.
- **Two renderers must agree,** and a change to the site output's markup changes the fixtures in `tests/render/`.
- **Project files are read through `ascribe_resolve::FileSystem`.** Libraries don't print, and don't panic on input.
- **User-visible changes** update the page under `docs/content/` and the unreleased section of `CHANGELOG.md` in the same phase.
- **Ascribe's docs are the first user.** A phase that adds something the docs can use, uses it there.
- No phase history in code or docs. Describe what the code does now.
- Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  ```

## Later, not in this plan

- Redirect rules written for a particular host, as a faster path to the same pages.
- Links from one project into another by id, checked.
- The list as constants a language imports.
- Finding permalinks nothing uses.
- Serving a section's text to a product: "Embedded help" in the [brainstorm](../brainstorm.md).
- A version on a link, which waits for [versions](../versions.md).
