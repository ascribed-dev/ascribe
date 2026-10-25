# Redirects

No address the docs have published stops working without someone deciding it should. A move made with Ascribe's tools records where the page went, a page deleted without a decision is a warning, and the build writes the redirects for the host. The idea, the research, the spike, the host test, and the maintainer's eleven settled answers are in the [proposal](../proposals/redirects.md); what waits for this plan to ship is in [the later steps](../proposals/redirects-later.md). This is the plan for building the first.

## Who it's for

- **A docs team** that reorganizes pages without losing readers who arrive by a bookmark, a search result, or a link from another site.
- **A reviewer**, who sees each move's redirect in the pull request, and CI stopping a page deleted with no decision.
- **An agent** that moves pages, told to move them with `ascribe rename`, and with a command that records moves it made with `git mv` (phase 2).

## What exists today

The proposal's "What exists" holds. The pieces this plan stands on, checked against `main` on 24 October:

- **The permalinks list,** the pattern to follow: `crates/ascribe-model/src/permalinks.rs` reads and writes `ascribe.permalinks` (`PermalinkList::parse`, `to_toml`); `crates/ascribe-check/src/permalinks.rs` compares it with the project (`permalinks-list-invalid`, `permalinks-list-behind`, `permalink-removed`, ASC175 to ASC177 in `tests/conformance/diagnostics.toml`); `crates/ascribe-cli/src/commands/permalinks.rs` has `sync`, `list`, `verify`, and `retire`; `crates/ascribe-lsp/src/retire.rs` is the editor's fix.
- **A build's addresses:** the spike listed them through the library: for each build, `Project::for_build`, `resolve_build` with `AstroRouter::from_consumer`, then `ascribe_resolve::pages::records` for each page's route and Markdown version, and `ResolvedPage::headings()` for each explicit id (`explicit`, `explicit_at`). Its code is a throwaway; the shape is what to keep.
- **Reading a revision:** `crates/ascribe-diff/src/gitfs.rs`, `GitFs`, a project as it is at a revision, read through one `git cat-file --batch`. `ascribe_diff::Repository::discover`, `Repository::base(rev, exact)`, and `Revision::read(...).project()` give the base's `ascribe_resolve::Project`. `ascribe diff`'s `--base` and `--base-exact` (`crates/ascribe-cli/src/commands/diff.rs`) say how a base is chosen, and what a shallow clone needs. `crates/ascribe-diff/tests/all/git.rs` makes repositories for tests.
- **The crates' order:** `ascribe-diff` depends on `ascribe-check`, so the check can't read git itself. Its caller reads the base and passes the addresses in (decision 2).
- **Moves:** `crates/ascribe-lsp/src/refactor.rs`, `will_rename`, is the one edit for a file that moves: the editor's `workspace/willRenameFiles` and `ascribe rename` (`crates/ascribe-lsp/src/rename.rs`, `Renaming::File`, called by `crates/ascribe-cli/src/commands/rename.rs`) both go through it. A heading's id is `Renaming::Id`, the server's rename. So a move recorded there is recorded by the CLI and the editor at once.
- **Headers for the host:** `crates/ascribe-emit/src/site/headers.rs` writes `_ascribe/headers.json` (output-layout §1.4). `[consumer] headers = "netlify"` (`crates/ascribe-model/src/sections.rs`, `HeadersFormat`) tells `@ascribed/astro`'s `packages/astro/src/headers.ts` to write Netlify's `_headers`, merging every project's, and to fail the build when a site's own `public/_headers` lacks the lines.
- **Check Every Build** (`packages/vscode/src/ui/checkEveryBuild.ts`) runs `ascribe check --format json`, so a diagnostic of the command's reaches the editor's run with nothing else. The MCP server's `ascribe_check` is in `crates/ascribe-cli/src/mcp/tools.rs`; the agents' rules are written by `crates/ascribe-cli/src/agents/`.
- **The trailing slash:** under `trailing-slash = "never"`, `@ascribed/astro` writes `x/index.html`, which a host serves at `/x/` behind a 301 ([#903](https://github.com/ascribed-dev/ascribe/issues/903)). Fixed before phase 1.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

The proposal's Settled 1 to 11 hold as written there, and so does its design, except where this list narrows it. From this plan:

1. **#903 comes first.** A page's published address is the one the host serves before any redirect is written to it.
2. **Where the code lives.** A build's addresses are listed in `ascribe-resolve`, beside the page records. `ascribe.addresses` is read and written in `ascribe-model`, beside `permalinks.rs`. The comparison and the file's checks are in `ascribe-check`, taking the base's address sets as an argument, since the check can't read git. The callers read the base: `ascribe check` through `ascribe-diff`, as `ascribe diff` does, and the language server in phase 4. `redirects.json` is the site emitter's (`crates/ascribe-emit/src/site/redirects.rs`), and so is the host's file. `@ascribed/astro` places and merges the host files, from what `headers.ts` does today.
3. **The base is chosen as `ascribe diff` chooses it,** with the same `--base` and `--base-exact` on `ascribe check`, and the same default: the merge base with the repository's default branch.
4. **Nothing applies without `ascribe.addresses`,** the base isn't read, and nothing costs time (Settled 5). With it, a base that can't be read is a warning of its own.
5. **Builds are matched by name.** A build in the base and not the working tree, a retired line, is one piece of advice until the later proposal.
6. **One warning per source,** a page or a file's `#id`, listing its addresses and builds.
7. **A lost explicit `#id` is advice,** until anchors reach a reader in the later proposal. Its entry is still recorded.
8. **`address-lost` runs where the proposal says:** `ascribe check`, the MCP server's `ascribe_check` over a project, and Check Every Build, which runs the command. Never in the language server's diagnostics as you type, nor the agents' hook.
9. **A move is recorded in `refactor::will_rename`** and in the id rename, the edits the CLI and the editor share, so the editor's moves record from the phase that records the CLI's. Phase 4 adds what only the editor needs.
10. **Netlify only.** `[consumer] host = "netlify"` is its one value. Under an Astro base that isn't `/`, the integration writes no redirects and says so, as for headers.
11. **The formats are phase 0's decision,** before phase 1 ships them: `ascribe.addresses`'s grammar, `redirects.json`, `[consumer] host` and where the host's files are written, the diagnostics' names and levels, and the commands.

The contracts, and the phase that records each in `project-docs/decisions/`: the formats (phase 0); `ascribe.addresses` in SPEC beside §7.5, `redirects.json` in the output layout, `[consumer] host` in the content-model contract (§15), the first diagnostics, and `ascribe addresses`' commands and JSON (phase 1); the file's other checks, the `gone` page, and the MCP tools (phase 2); `[former]` (phase 3); This page's and the Project view's shapes (phase 4). Decision #457b is amended, and the search and answers plan's decisions 9 and 35 superseded, in phase 1.

## The thread

**The use case:** in `examples/astro-site`, a writer runs `ascribe addresses init`, moves a page with `ascribe rename`, and the pull request passes `ascribe check`; a page deleted with no decision fails it with one warning; and on Netlify the old address answers 301 with the new one.

**The layers it crosses:**

| Layer | Where | What the thread needs from it |
|---|---|---|
| The model | `ascribe-model` | `ascribe.addresses` read and written, `[consumer] host` |
| Resolve | `ascribe-resolve` | Each build's addresses: pages, Markdown versions, explicit ids |
| Diff | `ascribe-diff` | The base's project, read from git |
| Check | `ascribe-check` | The comparison, `address-lost`, and an unreadable base |
| The edit | `ascribe-lsp`'s `refactor.rs` | A move's entry, in the move's edit |
| Emit | `ascribe-emit`'s site emitter | `redirects.json`, and Netlify's `_redirects` and `_headers` |
| The commands | `ascribe-cli` | `ascribe addresses init`, `move`, and `retire`; `check --base` |
| Astro | `@ascribed/astro` | The host's files at the site's root, every project's merged, a site's own first-wins |
| The contracts | SPEC, output layout, content model | Phase 0's formats, written down |
| The docs | `docs/content/publishing/` | A page: keep the addresses you've published |

**The seam it tests** is the build reading only the sources and `ascribe.addresses`: the thread's test merges the move, moves the base past it, and checks the redirect is still written and the check still passes. If the build or the check needs the old state once the base has moved on, Settled 8 fails, and the design goes back to the proposal. The thread also meets the bases the spike didn't read: a shallow clone, a pull request from a fork, and a push whose `before` commit is gone, each in a repository a test makes.

**Left out, and where it comes back:**

| Left out | Phase |
|---|---|
| `address-lost`'s fixes, `--from-git`, an `@id`'s rename recorded, the file's other checks, the `gone` page, and the agents' rules and tools | 2 |
| Settings changes (`[former]`), `go/` permalinks and `llms.txt` in the comparison, `agents` turned off | 3 |
| The editor's kept base and live count, This page's row, the Project view's count, and `init` offered | 4 |

## Phases

| Phase | Result |
|---|---|
| [0: Decide the formats](phase-0-formats.md) | The maintainer settles `ascribe.addresses`, `redirects.json`, `[consumer] host`, the diagnostics, and the commands. |
| [1: The thread](phase-1-thread.md) | A project with `ascribe.addresses` records its moves, CI stops a page deleted with no decision, and Netlify forwards the old address. |
| [2: Decisions](phase-2-decisions.md) | Every lost address has a fix: forward it, retire it, or record git's renames. A retired address serves a page saying why, and agents are told how. |
| [3: Settings](phase-3-settings.md) | Changing `base-path`, `permalink-path`, or `agents` is one warning whose fix forwards every address it moved, permalinks included. |
| [4: The editor](phase-4-editor.md) | The editor shows lost addresses between checks, and This page lists the old addresses that lead to a page. |

Phase 0 is a decision, because phase 1 ships three contracts. Phase 1 is the thread, through the riskiest seam. After it, by risk: phase 2 makes the warning answerable in every case, which is what makes the feature usable; phase 3 is the settings scenarios that ended in silent 404s in the tabletop; phase 4 is a surface over what exists.

## Carved out, and waiting

[The later steps](../proposals/redirects-later.md): versions, folders as one entry, anchors and stub pages, a build's own host, other hosts, and review's report.

## The other plans

None is running.

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- **Outputs change only as each phase says.** Phase 1 adds `redirects.json` and the host files to every build's site output and its manifest, and moves `_headers` from `@ascribed/astro`'s writing to the compiler's; each later phase names what it adds. Run `node scripts/compare/outputs.ts --base main`, and name each changed file with `--accept`, labeled `outputs changed`, with the block CONTRIBUTING.md asks for.
- **Contracts change only by a recorded decision,** in the phase that changes them, as the list above says.
- **Libraries don't print, and don't panic on input.** The addresses, the comparison, and the entries are returned; only `ascribe-cli` writes. Project files are read through `ascribe_resolve::FileSystem`, `ascribe.addresses` included.
- **Behavior changes come with tests at the layer that proves them** ([writing-tests](../../.claude/skills/writing-tests/SKILL.md)): the comparison in `ascribe-check`'s tests, with the base given as data; reading the base in tests that make repositories, as `crates/ascribe-diff/tests/all/git.rs` does; each diagnostic in `tests/conformance/cases/`; `redirects.json` and the host file in `crates/ascribe-emit/tests/all/`; the integration's placing and merging in `packages/astro`'s tests; the editor's rows in the language server's and the extension's.
- **Diagnostics** are added to `tests/conformance/diagnostics.toml` and `crates/ascribe-core/src/diagnostics.rs` in the same order.
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

- Everything in the proposal's "What this leaves out", and [the later steps](../proposals/redirects-later.md).
