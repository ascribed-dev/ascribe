# Inventory

Phase 1 of [Optimization](README.md): what Ascribe is made of, how long it takes, and where it repeats itself. Nothing was changed to produce it.

Measured on `main` at `d48ed66`, on 5 October 2026, on an Apple M1 Pro (10 cores, macOS). CI numbers come from GitHub's records of the last 100 workflow runs. Each section says what wasn't measured.

## Summary

The code is in better shape than the plan's first sketch assumed, and the trouble is narrower.

**What's fine, and shouldn't take time:**

- **Builds and tests are fast.** A clean debug build takes 18 s, all 1,625 Rust tests 98 s, all 613 JS unit tests 29 s. CI on a pull request takes 5.3 minutes.
- **Speed has budgets already.** `tests/corpora` times the commands on a 3,000-page project every week and fails on a regression. `check` takes 0.48 s there against a 5 s target.
- **The code carries no debt markers.** No `TODO`, `FIXME`, or `HACK` in our code. Three comments name an issue they wait on.
- **The parser fork is small.** It's 49,500 lines because it's a whole vendored library; our changes are 76 added lines, each marked, listed in `FORK.md`, and counted by a test.
- **Crates depend in one direction,** with no cycles, and 26 of 37 direct third-party dependencies belong to one crate each.

**What's worth the work.** The first five are ranked by what they cost today; the sixth is what the agents plan needs:

1. **The site check on `main` fails after merges that need a newer canary:** 4 of the last 10 pushes. See [finding 1](#1-main-goes-red-while-it-waits-for-the-canary).
2. **Reading files and handling paths has a home that most crates walk past:** 34 direct reads in 11 files outside the shared file system. See [finding 2](#2-file-reading-has-a-home-that-isnt-used).
3. **Names shared between Rust and TypeScript are typed out by hand:** one attribute name appears in 14 source files across six crates and packages. See [finding 3](#3-rust-and-typescript-share-names-by-hand).
4. **`diff` does all its work whatever changed, and nothing watches its time.** See [finding 4](#4-diff-has-no-short-path-and-no-baseline).
5. **A handful of facts live in many files:** the Node version in 14. See [finding 5](#5-facts-with-many-homes).

6. **Two commands keep their core where nothing else can call it, and failures are strings:** see [Command paths](#command-paths) and [finding 6](#6-commands-arent-all-thin-wrappers).

The rest are smaller and are listed under [Findings](#findings).

## Size

Lines include comments and blanks. Rust "source" includes the unit tests written inside source files, so the split understates tests.

### Rust

| Crate | Source | Tests | What it is |
|---|--:|--:|---|
| `tessera-resolve` | 11,028 | 6,058 | Includes, variants, links, snippets, builds; the file system; incremental updates |
| `tessera-lsp` | 7,451 | 5,396 | The language server |
| `tessera-emit` | 5,895 | 2,625 | Site, plain, and JSON outputs; the HTML render |
| `tessera-model` | 5,176 | 1,125 | Loading and checking `ascribe.toml` |
| `tessera-core` | 4,453 | 125 | Shared types: paths, attributes, availability, diagnostics |
| `tessera-syntax` | 3,691 | 3,098 | Parsing a file into Ascribe's tree |
| `tessera-diff` | 3,690 | 1,793 | Comparing revisions; the drift report; the HTML report |
| `tessera-check` | 3,273 | 1,667 | The checks, and loading a project |
| `tessera-cli` | 2,989 | 3,976 | The `ascribe` command |
| `tessera-sources` | 1,487 | 693 | Sources in other repositories |
| `tessera-fmt` | 792 | 1,249 | The formatter |
| **Ours, total** | **49,925** | **27,805** | |
| `comrak-tessera` | 35,299 | 14,205 | The vendored Markdown parser. One file, `scanners.rs`, is 19,175 lines |
| `tests/conformance` | 4,981 | | Plus 403 case files |
| `tests/corpora` | 4,391 | | Corpora, the synthetic project, benchmarks |
| `tests/commonmark` | 613 | | The CommonMark suite's runner |

`tessera-diff` also embeds a generated 1,152-line script and a 1,141-line stylesheet for the HTML report, built from the JS packages.

### TypeScript and the rest

| Package | Source | Tests | Styles |
|---|--:|--:|--:|
| `@ascribed/review` | 7,600 | 3,664 | 768, plus 570 in a `.ts` file |
| `ascribe-vscode` | 5,879 | 5,344 | 463 |
| `@ascribed/astro` | 3,745 | 1,690 | 249 in a `.ts` file |
| `@ascribed/elements` | 201 | 671 | 370 |
| `@ascribed/cli` | 89 | 73 | |
| **Total** | **17,514** | **11,442** | |

Also: `scripts/` 2,983 lines in 14 files; `site/` about 1,330; 11 workflows, 1,302 lines; `SPEC.md` 1,327; `docs/` 44 pages, 5,250 lines; `project-docs/` 47 files, 4,986 lines.

### The largest files we wrote

| Lines | File |
|--:|---|
| 1,954 | `packages/review/src/overlay/overlay.ts` |
| 1,257 | `crates/tessera-model/src/loader.rs` |
| 1,169 | `packages/vscode/src/preview/controller.ts` |
| 1,154 | `crates/tessera-resolve/src/incremental/mod.rs` |
| 1,079 | `crates/tessera-model/src/sections.rs` |
| 1,057 | `packages/astro/src/toolbar/app.ts` |
| 1,051 | `crates/tessera-lsp/src/complete.rs` |

Seven files over 1,000 lines, out of about 370. Function sizes weren't measured.

### History

513 commits since 9 September 2026. 90 pull requests merged since 28 September: about eleven a day. 29 issues filed, 3 open.

## Shape

### How the crates depend on each other

```text
core ─► syntax ─► (comrak-tessera)
  │        │
  ├─► model
  │        │
  └────────┴─► resolve ─► check
                  │   └─► emit ─► diff
                  │   └─► sources
                  └─► fmt (core, model, syntax only)

lsp  = check + diff + emit + fmt + resolve + model + syntax + core
cli  = lsp + sources + everything lsp uses
```

No cycles. `tessera-check` owns loading a project (`Project::load`), and every command and the language server go through it. `tessera-resolve` owns the file system (`FileSystem`, with a disk, a memory, and a git implementation).

### Surfaces

| Surface | Where | Reaches the core by |
|---|---|---|
| Command line | `tessera-cli` | Calling the crates |
| Language server | `tessera-lsp` | Calling the crates |
| VS Code | `packages/vscode` | The language server, plus custom requests for the preview and review |
| Astro | `packages/astro` | Running the binary (`build`, `diff`) and reading its output files |
| Review | `packages/review` | Data from `ascribe diff`; GitHub through `gh` |
| Elements | `packages/elements` | The site output's markup |
| HTML report | `tessera-diff`, with `review` and `elements` embedded | In the binary |

### Who turns a page into HTML

Two renderers, on purpose:

- **Ours,** `render_site_html` in `tessera-emit`, used by the editor's preview and the HTML report.
- **The site's own,** Astro's Markdown pipeline with our plugin, for the published site.

They can disagree. Tests compare them (`tests/render`, `site_anchors.rs`); PR #117's lost anchor was a case those tests didn't have.

### Third-party code

- **Rust:** 158 packages in the lock file, 37 used directly. The binary links 112.
- **In the binary twice:**
  - **Markdown.** The fork parses sources, and upstream `comrak` 0.55 renders HTML in `tessera-emit`. They're the same library at the same version.
  - **YAML.** `serde_yaml_ng` reads values, and `yaml-rust2` gives positions in `tessera-check`.
- **JS:** not counted.

## Time

### On a laptop

| What | Time |
|---|--:|
| Clean debug build of the command | 18 s |
| Clean release build | 28 s |
| Rebuild after a change in `tessera-core` | 3 s |
| Rebuild after a change in `tessera-cli` | 1 s |
| Build all tests | 23 s |
| Run all Rust tests (1,625 in 108 test programs) | 98 s |
| Clippy, after the tests are built | 9 s |
| JS type check, lint, format check | 1 s, 1 s, 2 s |
| JS unit tests (613) | 29 s |

Where the test time goes: four test programs take 31 of the 56 seconds spent inside tests (`agreement`, `fuzz`, `incremental_differential`, `structure`); each compares two implementations or generates inputs. In JS, the elements' browser tests take 19 of 29 seconds.

The slowest thing to compile is `lsp-types` (6.5 s), then `tessera-resolve` (3.8 s) and the fork (3.5 s).

The release binary is 10.5 MB.

Not run here: the VS Code integration suites and the Astro end-to-end tests (see CI).

### CI

| Workflow | When | Median | Notes |
|---|---|--:|---|
| CI | Pull request | 5.3 min | 18.4 minutes of machine time across 10 jobs |
| CI | Push to `main` | 3.4 min | |
| Review, Drift | Pull request | under 30 s | Report only |
| Site from npm | Push to `main` | 0.8 min | See [finding 1](#1-main-goes-red-while-it-waits-for-the-canary) |
| Canary | Nightly, by hand | 8.7 min | |
| Corpora | Weekly | not timed | Last run passed, 5 October |

The longest CI job is Rust on Windows (4.7 min), then the Astro end-to-end test (2.9 min). Queueing is under 15 seconds. Of the last 34 CI runs, 10 were cancelled by a newer push, which is the setting working.

## Speed

Release build, median of 3 to 5 runs.

| Command | On | Time |
|---|---|--:|
| `check` | `docs/` (44 files) | 54 ms |
| `check` | an example (5 to 11 files) | 5 to 7 ms |
| `diff`, `drift` | `docs/` against 20 commits back | 175 ms |
| `check` | 3,000 pages | 482 ms |
| `check` | 3,000 pages with a snippet on each | 775 ms |
| `build` | 3,000 pages, first time | 2.1 s |
| `build` | 3,000 pages, nothing changed | 1.3 s |
| `diff` | 3,000 pages, whatever changed | 1.3 s |
| `diff` | 3,000 pages with snippets | 3.2 s |
| `drift` | the same, nothing changed | 0.6 s |
| `drift` | the same, one region changed | 2.5 s |
| `check`, text or JSON | 1,000 pages with a warning each | 170 ms |

Recorded earlier on the CI machine and not rerun: the language server answers a keystroke in 3 to 9 ms and gives first diagnostics in 755 ms, on 3,000 pages.

Not measured: the Elastic corpus (it needs a fetch), `ascribe build` on `docs/` (my command was wrong), memory, and anything on Windows.

## Repeats

### Reading files and paths

| | Count |
|---|--:|
| File-system calls in our Rust source | 119 |
| of which inside `tessera-resolve` | 62 |
| Direct reads outside the shared `FileSystem` | 34, in 11 files across 8 crates |
| Files that handle symbolic links or real paths | 5: `resolve/fs.rs`, `emit/store.rs`, `sources/copies.rs`, `model/loader.rs`, `cli/commands/fmt.rs` |
| Functions named `relative_to` | 3: `diff/gitfs.rs`, `lsp/uri.rs`, `resolve/snippet/mod.rs` |
| Functions that normalize a path | 2: `lsp/uri.rs`, `model/loader.rs`, beside `RelPath` in `core` |

The direct reads are mostly in `emit/store.rs` (9), `cli/commands/fmt.rs` (6), and `sources/copies.rs` (5). The three link fixes this month (#73, #99, #118) each landed in a different file from this list, and the two gaps #118's review found (`fmt` writing through a link; a snippet leaving a `..` source) are in two more.

On the JS side, three files find the binary (`cli`, `astro`, `vscode`), and eight source files start a process.

### Names shared across languages

| Name | Source files | Crates and packages |
|---|--:|---|
| `data-ascribe-source` | 14 | `emit`, `diff`, `cli`, `astro`, `review`, `vscode` |
| `ascribe-attributes` | 9 | `core`, `emit`, `astro` |
| `ascribe-anchor` | 4 | `emit`, `cli`, `astro` |
| `data-ascribe-term` | 4 | `emit`, `diff`, `elements` |
| `data-ascribe-scheme` | 4 | `diff`, `astro`, `review`, `elements` |
| `data-ascribe-change` | 4 | `diff`, `review` |

Each is a string literal wherever it appears. Eight TypeScript files declare types for JSON that Rust writes. One of those shapes is checked: the site output's schema, by `tests/zod`.

### Facts written in several files

Counted outside lock files, plans, snapshots, and the changelog.

| Fact | Files | Kept in step by |
|---|--:|---|
| The Node version (24) | 14 | Nothing |
| The release version (0.1.1) | 11 | The release script |
| The npm platform packages | 10 | A manifest test, for some |
| The docs' source folders | 9 | Nothing |
| The glibc floor (2.28) | 5 | The release workflow tests the floor, not the text |
| The Rust version (1.98) | 2 | Nothing |

Ten test files already compare a generated or copied file with its source and fail when it's stale (`ASCRIBE_BLESS`). The habit exists; these facts are the ones outside it.

### Styles

Ten style sources hold 393 custom-property definitions between them. The largest is the HTML report's (1,140 lines, 127 properties), which is generated. Whether the hand-written ones repeat each other's values wasn't worked out.

## Command paths

How each command gets from its arguments to its result, and whether the language server reaches the same code. Read from the source at `5f5dd4c`.

| Command | Loads the project with | Core function | The language server |
|---|---|---|---|
| `check` | `context::load_project` | `diagnose`, in the CLI crate, over `check_all_builds` or `check_builds` | Loads its own way; calls `check_file` and `PageChecker::check_resolved` for the editor's build |
| `build` | `load_project`, then a second load in `tessera-resolve` | `diagnose`, then `write_outputs`, both in the CLI crate | Renders one page for the preview through `tessera-emit` |
| `diff` | `load_project`, then a second load | `tessera_diff::compare_builds`; about 100 lines around it in the CLI crate | Calls `compare_builds` and `compare_page_in`: the same code |
| `drift` | `load_project`, then a second load | `tessera_diff::drift` | Doesn't do this |
| `fmt` | Its own `find_config`, `load_model`, and `collect` | `tessera_fmt::format` | Calls `format`: the same code |
| `sources` (`fetch`, `status`, `update`) | `load_project`, then a second load in two places | `tessera_sources::fetch`, `status`, `update` | Doesn't do this |
| `lsp` | From the editor | `tessera_lsp::run_stdio` | |

What it shows:

- **Three ways to load a project.** `context::load_project` (five commands); `fmt`'s own three functions, which walk the disk themselves; and the language server's (`tessera_model::load_str_in` with `IncrementalProject::load`). The server's is separate for a reason, since it holds unsaved text and updates in place. `fmt`'s isn't.
- **The second load is written five times.** `tessera_resolve::Project::load`, with the same three arguments taken from the first project, appears in `build`, `diff`, `drift`, and twice in `sources`.
- **Two commands keep their core in the CLI crate.** `check` and `build` share `diagnose` and `select_builds`, and `build` has `write_outputs` (about 130 lines). Nothing outside the binary can call them. `diff`, `drift`, `fmt`, and `sources` have their core in a library, with argument handling and reporting around it.
- **Where both do the same job, two of four share one function.** Formatting and comparing do. Checking shares the checks but not the entry point, and a test (`lsp_parity`) compares the results. Rendering differs by design.

## Rules in force

What's already enforced, so no phase needs to add it.

- **Rust:** `unsafe` is forbidden. `unwrap`, `expect`, `panic`, `dbg!`, and `todo!` are linted, and CI treats warnings as errors. Public items must have documentation.
- **TypeScript:** strict mode with unchecked index access and exact optional properties; a type-aware lint.
- **Actions** are pinned to commits, and Dependabot runs weekly.

What isn't:

| Gap | Count |
|---|--:|
| Functions that return an error as a plain string | 10: 8 in the CLI crate (5 of them in `fmt.rs`), 2 in `tessera-lsp/src/review.rs` |
| Lines in library crates that print to standard error | 7, in `lsp/core.rs`, `lsp/server.rs`, and `resolve/snippet/tags.rs`. No logging library is used |
| Files under `diff`, `emit`, `resolve`, and the CLI that use a hash map or set | 13. Whether any iteration order reaches an output wasn't traced |
| A release profile in `Cargo.toml` | None: no link-time optimization, no stripping |
| An audit of dependencies (advisories, licenses, duplicates, unused) | None, for Rust or JS |
| A check of the npm packages' `exports` and types as published | None |
| `pub fn` in `tessera-resolve` | 134. How many are used outside the crate wasn't counted |

## Waiting on something

Comments in code or configuration that name an issue as the reason for what follows:

| Where | Issue | State |
|---|---|---|
| `packages/astro/src/code-titles.ts` | #106 | Closed. The file is the fix, not a workaround |
| `packages/astro/src/schema.ts` | #93 | Closed. The same |
| `.github/workflows/review.yml` | #107 | Closed. A blank line kept out of a rendered example |
| `packages/vscode/test/integration/run.ts` | #56, #74 | Tracing, and a note |

So: no open workaround in the code. The site's overrides that reviews flagged went with #117. This pattern was real last week and is small today.

## Tests

- **Rust:** 1,625 tests, 5 ignored. Kinds: unit, snapshot (`insta`), 403 conformance cases, property tests, and four that compare two implementations.
- **JS:** 613 unit tests, 2 skipped; browser tests for the elements in three engines; VS Code integration suites; Astro end-to-end.
- **Agreement tests exist** between surfaces: `lsp_parity` (command against server), `parity` (checks), `agreement` and `differential` (incremental against from-scratch), `tests/render` (our HTML against the site's).
- **Platform:** six Rust test files have tests that run only on Unix, all about links. CI runs Rust and the VS Code suite on Windows for pull requests.
- **Overlap** between suites wasn't analysed.

## Guides for implementors

| File | Lines |
|---|--:|
| `SPEC.md` | 1,327 |
| `tests/conformance/README.md` | 518 |
| `crates/tessera-lsp/README.md` | 414 |
| `RELEASING.md` | 304 |
| `CONTRIBUTING.md` | 89 |
| `README.md` | 71 |

- No `ARCHITECTURE.md`, `AGENTS.md`, or `CLAUDE.md`.
- Four of our eleven crates have a README (`cli`, `diff`, `emit`, `lsp`). The seven without include the largest, `tessera-resolve`. Most source files do open with a module comment.
- **No single command builds the JS packages from a fresh checkout.** `pnpm -r build` fails in `examples/astro-site`, which needs a native binary staged first. The right order exists only as steps in `js.yml`.

## Outside the repository

| Thing | State |
|---|---|
| Repositories in the organization | `ascribe` (public); `review-fixture`, `sources-fixture-docs`, `sources-fixture-code` (private) |
| Elsewhere | `KyleBlankRollins/ascribe-review-scratch` (private, last pushed 4 October): superseded by `review-fixture` |
| Branches | `main` only |
| Protection on `main` | One required check, `all checks`. No required review. Admins can bypass |
| Repository secrets | `NETLIFY_AUTH_TOKEN`, `NETLIFY_SITE_ID`, `SITE_BUILD_HOOK` |
| Environments | `release` (reviewers required; holds `NPM_TOKEN`), `canary`, `copilot` |
| Variables in workflows | `AZURE_CLIENT_ID`, `AZURE_TENANT_ID`; none listed at repository level, so they're in an environment or unset |
| npm, public | `@ascribed/cli`, `astro`, `elements`, `review`, and four `cli-<platform>` packages |
| npm tags | `latest` 0.1.1, `next` and `next-pending` 0.1.2-next.6. **`@ascribed/review`'s `latest` is 0.1.2-next.2,** a canary |
| Dependabot | Weekly, for cargo, npm, and Actions |
| Third-party actions | 9, each pinned to a commit |

Not checked, because it needs your accounts: Netlify's settings, the npm trusted-publisher entries, the GitHub App the update workflow uses, the VS Code Marketplace publisher, and what the `copilot` environment is for.

Leftovers to remove or decide: `NPM_TOKEN` (trusted publishing replaced it), the scratch repository, and the `latest` tag on `@ascribed/review`.

## Findings

Ranked by what they cost today. Each names the phase that would take it.

### 1. `main` goes red while it waits for the canary

"Site from npm" builds the docs with the published canary. A merge that uses anything newer fails it until the next nightly: 4 failures in the last 10 pushes, each needing a manual canary run to clear. Drift and Review fail on pull requests for the same reason (5 of the last 30 Drift runs). A red check that means "wait" teaches people to ignore red. **Phase 6.**

### 2. File reading has a home that isn't used

`FileSystem` in `tessera-resolve` is the one place that knows the content root and what to do with a link. 34 reads in 8 crates go around it, and every link bug this month was in one of them. **Phase 5,** and the first thing I'd do there.

### 3. Rust and TypeScript share names by hand

The attribute and element names that tie the site output to the elements, review, the editor, and Astro are literals in up to 14 files, and most JSON shapes are declared twice. PR #117's lost anchor was this kind of break. A generated constants file and schema, as `tests/zod` does for one shape, would close it. **Phase 4.**

### 4. `diff` has no short path and no baseline

`diff` takes 1.3 s on 3,000 pages whether one page changed or none, and 3.2 s once pages have snippets. The benchmark times `diff` and `drift`, but `baselines/perf.json` records no number for them, so the weekly check can't fail on them. That's how #79 doubled the time unnoticed. Recording the baselines is an hour's work. **Phase 2,** then **phase 7.**

### 5. Facts with many homes

The Node version, the platform list, the glibc floor, and the docs' source folders, as counted above. **Phase 4.**

### 6. Commands aren't all thin wrappers

From [Command paths](#command-paths): `check` and `build` have their core in the CLI crate, `fmt` loads a project its own way, and the second load is written five times. Ten functions report failure as a string, and library crates print to standard error in seven places. A tool that wraps a command, as the agents plan's will, needs a function it can call and a failure it can tell apart from another. **Phase 5,** and the part of it that has to finish before the agents plan's phase 2.

### 7. Two copies of one parser

The binary holds the fork and upstream `comrak` at the same version. If the fork can render HTML for `tessera-emit` with its Ascribe option off, the second copy goes, with about 2 s of build and some of the 10.5 MB. Whether it can wasn't tested. There are two YAML libraries for a different reason, and nothing would have flagged either pair: no check looks for duplicates. **Phase 5,** with the check in **phase 6.**

### 8. No map

No first-read file, seven crates without a README, and a JS build order that lives in a workflow. At eleven pull requests a day, mostly from implementors starting cold, this is the cheapest thing on the list to fix. **Phase 3.**

### 9. A few very large files

`overlay.ts` (1,954 lines) is nearly twice the next one. Large files aren't wrong, but these seven are where parallel pull requests will collide. **Phase 5,** only where a file is also being changed for another reason. Function sizes weren't measured; a lint with a threshold would measure them and keep them measured.

### 10. Leftovers

The three under [Outside the repository](#outside-the-repository). **Phase 6.**

### What the first sketch got wrong

- **Workarounds outliving their cause:** nearly none left.
- **Slow CI:** already fixed by the single-gate change.
- **No performance budgets:** they exist; two commands are missing from them.
- **The fork as a burden:** it's large on disk and small to maintain.
- **Workflow size:** 1,302 lines, not the 2,600 the plan first said.

## What this means for the plan

- **Phase 2 shrinks.** The budgets exist. What's left is proving the outputs are the same from one run to the next, the comparison, the two missing baselines, and the two things never measured: memory and Windows.
- **Phases 4 and 5 have clear first targets:** shared names, then file reading and the command paths.
- **Phase 6 has one job that matters,** the canary wait, plus an audit of dependencies and a short list of leftovers.
- **Phase 7 is mostly `diff`,** and a release profile.
- **Phase 3 matters more than its place in the list suggests,** and changes no code, so it can run beside phase 2.

## After

Taken again on 6 October 2026, when phases 2 to 8 had merged, for [phase 8](phase-8-close.md#part-b-the-measures). The plan's [Measures](README.md#measures) table has the counts.

**The laptop numbers couldn't be taken the same way:** the Apple M1 Pro isn't reachable from where phase 8 ran. So both sides were taken again on one machine, a 4-core Linux x64 cloud container, building `d48ed66` (this inventory's commit) and #151's head (`main` at `b27d9f8` with the rename) each into an empty target folder. Read the columns against each other, not against the M1's figures above.

| What | Before (`d48ed66`) | After |
|---|--:|--:|
| Clean debug build of the command | 34 s | 31 s |
| Clean release build | 66 s | 143 s |
| Rebuild after a change in `ascribe-core` | 3.1 s | 2.9 s |
| Rebuild after a change in `ascribe-cli` | 2.6 s | 2.6 s |
| Build all tests | 60 s | 72 s |
| Run all Rust tests | 55 s, 1,625 tests | 80 s, 1,675 tests |
| Clippy, after the tests are built | 21 s | 20 s |
| JS lint, format check | 1.4 s, 5.0 s | 1.5 s, 4.7 s |
| Release binary, Linux x64 | 12.5 MB | 8.2 MB |

- **The release build takes twice as long,** for link-time optimization (phase 7B, which measured about a minute more on every platform and accepted it for a third off the binary).
- **The Rust tests take 25 s longer, and one test is most of it:** `ascribe-cli`'s `determinism` test (phase 2A), which runs the binary over every example project, takes 21.5 s alone; the other test programs together went from 58 s to 61 s. Filed with CI's growth as [#154](https://github.com/ascribed-dev/ascribe/issues/154).
- JS unit test time wasn't taken: this machine can't run the elements' Firefox and WebKit tests, which were 19 of the 29 seconds.

**CI**, from GitHub's records of the last 100 CI runs (from 6 October, 14:26 UTC on), as the inventory took it:

| | Before | After |
|---|--:|--:|
| CI on a pull request, median | 5.3 min | 6.3 min |
| CI on a push to `main`, median | 3.4 min | 4.9 min |
| Machine time per pull request (8 green runs) | 18.4 min | 24.2 min |
| Longest job: Rust on Windows | 4.7 min | 6.9 min |

The plan added `outputs unchanged` (about 2 minutes, beside the others) and the dependency audits (12 s); the Windows job, which sets the wall time, grew most. Filed as [#154](https://github.com/ascribed-dev/ascribe/issues/154).

**Speed and memory on 3,000 pages,** on the Corpora workflow's runner, against phase 2B's first baselines (the inventory's own figures are the M1's):

| Command | Phase 2B (runner) | After (runner) |
|---|--:|--:|
| `check` | 704 ms | 659 ms |
| `build`, first time | 1.93 s | 1.80 s |
| `diff`, nothing changed | 1.45 s | 0.82 s |
| `diff`, with snippets, nothing changed | 1.87 s | 1.05 s |
| `drift`, one region changed | 923 ms | 545 ms |
| Peak memory: `check`, `build`, `diff` with nothing changed | 221, 265, 547 MB | 221, 265, 371 MB |

The language server still answers a keystroke in 2 to 9 ms on 3,000 pages.

**Review findings,** counted in the reviews on GitHub, by the kinds in the plan's [What reviews have shown](README.md#what-reviews-have-shown) table: the ten pull requests merged before phase 2 began (#116 to #126, without #124, an issue; five had no review) against the ten most recent (#141 to #150, all reviewed). Every review was by one reviewer, so the count is rough.

| Kind | Before (27 findings) | After (33 findings) |
|---|--:|--:|
| The same fact is written in several places | 1 | 3 |
| The same bug is fixed once per surface | 3 | 1 |
| A language change doesn't reach every surface | 3 | 0 |
| A workaround outlives its cause | 0 | 0 |
| One change breaks another surface | 2 | 2 |
| A cost appears unmeasured | 0 | 2 |
| A decision is made and not recorded | 1 | 5 |
| Things are left behind | 0 | 1 |
| Some platforms are checked less | 1 | 0 |
| Something else | 16 | 19 |

How each was classed: duplicated logic, or a gap in a one-home check, is the first kind; two parallel pull requests where one breaks the other's check is "one change breaks another" (a plain note about merge order is "something else"); the same fix made twice in different code is "the same bug"; a reviewer asking for a decision or a deferral to be written down is "not recorded"; a cost counts only where the reviewer raised one. Before, "something else" was mostly correctness and security bugs in new feature code. After, it's gaps in the checks themselves (#145, #146), merge order between parallel pull requests, and nits. The rise in "not recorded" is reviewers holding the new rule that decisions go in `decisions.md`, so it reads as the rule working, not as a new problem.
