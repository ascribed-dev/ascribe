# Architecture

How Ascribe is put together: what each part is for, how the parts reach each other, and where the tests are. It describes the code as it is. What the language means is in [SPEC.md](SPEC.md), how to contribute is in [CONTRIBUTING.md](CONTRIBUTING.md), and each crate and package has a README with its own details.

The Rust crates are `ascribe-*` and the npm packages `@ascribed/*`. Until October 2026 the crates were `tessera-*`, the project's working name, so older plans and the changelog use it.

## The parts

### Rust crates

All in `crates/`. Each depends only on the crates listed for it in [How the crates depend on each other](#how-the-crates-depend-on-each-other).

| Crate | What it's for |
|---|---|
| [`ascribe-core`](crates/ascribe-core) | The types every crate shares: spans, paths (`RelPath`), attributes, availability specs, issues and the diagnostic slugs, directive schemas, and the consumer traits (`Slugger`, `Router`). |
| [`ascribe-syntax`](crates/ascribe-syntax) | Parsing one source file into Ascribe's syntax tree, with exact byte spans, through the fork of comrak. No downstream crate sees comrak's types. |
| [`ascribe-model`](crates/ascribe-model) | Loading and validating the content model, `ascribe.toml`, into a typed `ContentModel`; also `ascribe.lock`. |
| [`ascribe-resolve`](crates/ascribe-resolve) | The project graph: the file system (`FileSystem`), the source index, includes and snippets, build resolution (availability, variants, phrases, ids, links, glossary), routes, slugs, and incremental updates. |
| [`ascribe-check`](crates/ascribe-check) | The checks, file-level and page-level, and loading a project from disk (`ascribe_check::Project`). Diagnostics are worded from the registry. |
| [`ascribe-emit`](crates/ascribe-emit) | The outputs: site, plain, and JSON; the output directory's ownership rules; `render_site_html`; the generated Zod schema. |
| [`ascribe-diff`](crates/ascribe-diff) | What changed between a git revision and the working tree (`diff`), which pages' examples changed while their words didn't (`drift`), and the static HTML report. |
| [`ascribe-sources`](crates/ascribe-sources) | Sources in other repositories: `fetch`, `update`, and `status` of the copies and their pins. The only code that reaches another repository. |
| [`ascribe-fmt`](crates/ascribe-fmt) | The formatter: minimal edits that put Ascribe constructs in canonical form, leaving prose alone. |
| [`ascribe-lsp`](crates/ascribe-lsp) | The language server, run as `ascribe lsp`. |
| [`ascribe-cli`](crates/ascribe-cli) | The `ascribe` binary: `check`, `build`, `diff`, `drift`, `fmt`, `sources`, and `lsp`. |
| [`comrak-ascribe`](crates/comrak-ascribe) | A fork of the comrak CommonMark parser, with Ascribe's block-level changes. Each change is marked `// ASCRIBE:` and listed in [FORK.md](crates/comrak-ascribe/FORK.md). It doesn't use the workspace lints. |

### npm packages

All in `packages/`, a pnpm workspace with `examples/astro-site` and `tests/zod`.

| Package | What it's for |
|---|---|
| [`@ascribed/cli`](packages/cli) | The `ascribe` command for npm. It finds and runs the native binary from one of the `@ascribed/cli-<platform>` packages (`packages/cli/platforms/`). |
| [`@ascribed/astro`](packages/astro) | The Astro integration: runs `ascribe build`, loads the site output as a content collection, renders its markers, and serves review in `astro dev`. |
| [`@ascribed/elements`](packages/elements) | The custom elements the site output uses, to the letter of [CONTRACT.md](packages/elements/CONTRACT.md). |
| [`@ascribed/review`](packages/review) | Reviewing pages as readers see them: marking changed blocks, and placing a pull request's review threads beside them. |
| [Ascribe for VS Code](packages/vscode) (`ascribe-vscode`) | The editor extension: a client of the language server, the page preview, and review in the editor. |

### Everything else

| Directory | What it holds |
|---|---|
| `tests/conformance` | The conformance suite (`ascribe-conformance`): cases written from the spec, and `diagnostics.toml`, the diagnostics registry. |
| `tests/commonmark` | The CommonMark spec's examples, run against the fork and the parser. |
| `tests/corpora` | Real documentation sets, the 3,000-page synthetic project (`tests/corpora/synthetic`), and the performance benchmarks with their baselines. |
| `tests/render` | The site-render fixtures both HTML renderers must pass. |
| [`tests/zod`](tests/zod) | Type-checks the generated Zod schemas and validates pages with them. |
| `examples/` | Example projects. `examples/quill` is the complete one most tests use; `examples/astro-site` publishes one with Astro. |
| `docs/`, `site/` | The user docs, an Ascribe project, and the Astro site that publishes them. `site/` installs Ascribe from npm, outside the workspace. |
| `schemas/` | The JSON Schemas of the JSON the commands and the language server write, generated from the Rust types (`crates/ascribe-cli/src/shapes.rs`). |
| `scripts/` | Release scripts (`scripts/release`, with `consumers.ts`, which checks the npm packages as packed), the review and sources fixtures, the comparison of two builds' outputs (`scripts/compare/outputs.ts`), the checks that the READMEs link to real docs pages and that the facts several files repeat (the Node, Rust, and glibc versions, the docs' source folders) agree (`scripts/docs-site`) and that this map's paths and commands exist (`scripts/repo-docs`), and `build-all.ts`. |
| `project-docs/` | Plans. They describe what was intended, not necessarily what is. |

## How the crates depend on each other

Each crate uses the crates listed after it, and no others of ours.

```text
ascribe-core       (none)
ascribe-syntax     core, comrak-ascribe
ascribe-model      core
ascribe-fmt        core, syntax, model
ascribe-resolve    core, syntax, model
ascribe-check      core, syntax, model, resolve
ascribe-sources    core, model, resolve
ascribe-emit       core, syntax, model, resolve, comrak-ascribe
ascribe-diff       core, syntax, model, resolve, check, emit
ascribe-lsp        core, syntax, model, resolve, check, emit, diff, fmt
ascribe-cli        core, model, check, emit, diff, sources, fmt, lsp
```

There are no cycles. Every crate's `Cargo.toml` takes our crates from `[workspace.dependencies]` in the root [Cargo.toml](Cargo.toml).

The npm packages depend on each other like this: `review` uses `elements`; `astro` uses `cli`, `elements`, and `review`; the VS Code extension uses `elements` and `review`, and runs the binary.

## Surfaces

Every surface reaches the same core.

| Surface | Where | How it reaches the core |
|---|---|---|
| Command line | `ascribe-cli` | Each subcommand is a module in `crates/ascribe-cli/src/commands/`: its arguments, one call into a library ([Each command's entry](#each-commands-entry)), and the report. |
| Language server | `ascribe-lsp` | Calls the crates, over an `IncrementalProject` it keeps current as the editor types. |
| VS Code | `packages/vscode` | Starts `ascribe lsp`, one server per project (`packages/vscode/src/registry.ts`). The preview and review use custom requests: `ascribe/preview`, `ascribe/review/setBase`, and `ascribe/review/changes`. |
| Astro | `packages/astro` | Runs the binary: `ascribe build` (`packages/astro/src/run.ts`) and, for review, `ascribe diff` (`packages/astro/src/review/diff.ts`). Then reads the files it wrote. |
| Review | `packages/review` | Reads `ascribe diff --format json`'s data, and GitHub through `gh` (`packages/review/src/github/`). |
| Elements | `packages/elements` | The site output's markup. |
| HTML report | `ascribe-diff`, `crates/ascribe-diff/src/html/` | In the binary. Its script and stylesheet are built from `review` and `elements` by `pnpm --filter @ascribed/review embed`. |

## From a source file to an output

The steps every surface shares, in order:

1. **Parse.** `ascribe_syntax::parse` reads one file into a `ParsedDocument`: blocks, directives with their heads, containers and groups, and the issues the parser finds.
2. **Index.** `ascribe_resolve::index_file` records what's true of one file alone: headings and their source ids, includes, links, images, phrases, snippets. It doesn't touch the file system.
3. **Load the graph.** `ascribe_resolve::Project::load` indexes every source the `FileSystem` lists and resolves what each reference names.
4. **Expand.** `Project::expand` replaces each `@include` with its target and each `@snippet` with its code. Every block keeps the file and span it was written in.
5. **Resolve for a build.** `BuildResolver` applies availability, the build's modes, phrases, page ids, links to routes, and the glossary, giving a `ResolvedPage` (SPEC §9.2).
6. **Check.** `ascribe_check::check_files` runs the file-level checks; `PageChecker` runs the page-level ones on resolved pages. `check_project` and `check_all_builds` do both.
7. **Emit.** A `ascribe_emit::Emitter` turns resolved pages into files, and `OutputDir` writes them.

## Loading a project, and reading files

- **Finding `ascribe.toml`.** The commands look in `--config` or the nearest parent (`ascribe_check::Project::locate`, called from `crates/ascribe-cli/src/context.rs`). The language server looks from its workspace folders (`find_config` in `crates/ascribe-lsp/src/core.rs`).
- **Loading.** `ascribe_check::Project::load` reads the content model (`Project::load_model`) and the sources, and is what `check` reports on; every command loads its project with it, through `load_project` in `crates/ascribe-cli/src/context.rs`, except `fmt`, which takes only the content model (`Project::load_model`) so that a project whose pages have errors can still be formatted. The page-level checks index it again as a `ascribe_resolve::Project` (`crates/ascribe-check/src/page/bridge.rs`), unless the caller passes its own index (`PageChecker::with_index`). The commands that resolve builds (`build`, `diff`, `drift`, `sources`) take their own `ascribe_resolve::Project` over the same files from `Project::index`, so they index the project twice. The language server loads with `ascribe_model::load_str_in` and `ascribe_resolve::IncrementalProject::load`, since it holds unsaved text and updates in place.
- **Reading.** `ascribe_resolve::FileSystem` (`crates/ascribe-resolve/src/fs.rs`) is where a project's files are read: it knows the content root, which files are sources, the boundary a file must stay inside, exact-case names, and symbolic links. `DiskFs` reads a real project, `MemoryFs` holds one for tests, and `ascribe_diff::GitFs` reads one at a commit. A read anywhere else says why it isn't a project read, in a comment starting `Outside FileSystem:`, and `crates/ascribe-resolve/tests/file_reads.rs` fails on one that doesn't. The ones left: the content model's (`crates/ascribe-model/src/lib.rs`, and its checks that the folders it names exist), finding and reading `ascribe.toml` (`Project::locate` and `Project::load_model` in `crates/ascribe-check/src/project.rs`), the output directory's (`crates/ascribe-emit/src/store.rs`), writing the copies of sources in other repositories (`crates/ascribe-sources/src/copies.rs`), `fmt`'s (`crates/ascribe-fmt/src/files.rs`), the language server's walk when files change (`crates/ascribe-lsp/src/core.rs`), and assets: the build copies them from `EmitContext::asset_source`, and the HTML report inlines them from the same place (`DiskAssets` in `crates/ascribe-diff/src/html/mod.rs`), both skipping the boundary, case, and link rules.
- **Paths.** `ascribe_core::path` holds the path helpers every crate shares: `RelPath::relative_to` (a path inside a folder), `relative_path` (from one directory on disk to another, with a leading drive letter matched in either case), and `normalize`.
- **Git.** `ascribe-diff` (`crates/ascribe-diff/src/git.rs`) and `ascribe-sources` (`crates/ascribe-sources/src/remote.rs`) run the `git` executable with a fixed argument list. No git or HTTP library is linked.

## Each command's entry

Each command is one call into a library, after the project is loaded. The call takes plain arguments (the project, and options) and returns a typed result; the CLI parses the arguments, prints the result, and chooses the exit code. A tool that wraps a command calls the same function.

| Command | Entry | Notes |
|---|---|---|
| `check` | `ascribe_check::diagnose` | Chooses the builds (`select_builds`) and returns their diagnostics. |
| `build` | `ascribe_check::diagnose`, then `ascribe_emit::write_outputs` | The report is printed between the two, and a build with errors stops there. `write_outputs` reports each output as it's written through a callback. |
| `diff` | `ascribe_diff::diff_project` | Returns the report, with the working tree's error count; `ProjectDiff::html` renders it for `--format html`. |
| `drift` | `ascribe_diff::drift_project` | |
| `fmt` | `ascribe_fmt::format_files` | Takes the content model from `Project::load_model`, and reports each file changed through a callback. |
| `sources` | `ascribe_sources::fetch`, `status`, `update` | Over a `ascribe_sources::Workspace`. `update` then calls `drift_project` against `HEAD` for the pages whose examples changed. |
| `lsp` | `ascribe_lsp::run_stdio` | |

The language server calls the same code where it does the same job: `ascribe_fmt::format` for formatting, and `ascribe_diff::compare_builds`, which `diff_project` calls, for review. It checks incrementally, so it calls `check_file` and `PageChecker` instead of `diagnose`, and `crates/ascribe-cli/tests/lsp_parity.rs` holds the two to the same diagnostics.

When an entry can't do its work, it returns an error type, never a string. Each error implements `ascribe_core::Coded`: its `code()` is a short lowercase identifier (`unknown_build`, `git_not_found`) that names the failure whatever the message says, and an error that wraps another has the inner one's code. `crates/ascribe-cli/src/exit.rs` lists every code once, in a test, and turns an error into the exit code the command gives (`2`, for every one). Libraries return errors and don't print: `clippy::print_stdout` and `clippy::print_stderr` are denied workspace-wide, and allowed only in the CLI, the benchmarks, the test harnesses, and the language server's log (`crates/ascribe-lsp/src/log.rs`).

## Two HTML renderers

The site output is Markdown with web components, and two things turn it into HTML:

- **Ours,** `render_site_html` in `crates/ascribe-emit/src/render/`, used by the editor's preview and the HTML report. It renders with the comrak fork, the parser `ascribe-syntax` uses, with Ascribe's option off.
- **The site's,** Astro's Markdown pipeline with our plugin: `packages/astro/src/satteri.ts` for Astro's default processor, `packages/astro/src/rehype.ts` for `unified()`.

They're meant to agree, and these tests hold them to it:

- `tests/render/`: fixtures of site Markdown and the HTML it must render to. `crates/ascribe-emit/tests/render_fixtures.rs` runs ours, and `packages/astro/test/render-fixtures.test.ts` runs the plugin with both processors. See [tests/render/README.md](tests/render/README.md).
- `crates/ascribe-emit/tests/site_anchors.rs` writes the fixtures' inputs and the corpus from the site emitter, so they stay what the emitter writes.
- `pnpm --filter ascribe-vscode test:parity` compares the editor's preview with the Astro site built from the same project (`packages/vscode/test/parity/`). CI runs it after the Astro end-to-end test.

## Outputs and their contracts

| Output | Written by | Its contract |
|---|---|---|
| Site: Markdown with web components | `ascribe_emit::SiteEmitter` | [Site render](docs/content/contracts/site-render.md), and [the elements' contract](packages/elements/CONTRACT.md) |
| Plain Markdown | `ascribe_emit::PlainEmitter` | SPEC §9.4, and [crates/ascribe-emit/README.md](crates/ascribe-emit/README.md) |
| JSON, one document per page | `ascribe_emit::JsonEmitter` | [crates/ascribe-emit/README.md](crates/ascribe-emit/README.md) |
| Where each output's files go, and what Ascribe may replace | `ascribe_emit::OutputDir` | [Output layout](docs/content/contracts/output-layout.md) |
| Images and linked files | `crates/ascribe-emit/src/assets.rs` | [Assets](docs/content/contracts/assets.md) |
| The Zod schema, `_ascribe/schema.ts` | `crates/ascribe-emit/src/zod/` | [crates/ascribe-emit/README.md](crates/ascribe-emit/README.md#zod), with each field type's schema in the [content model contract](docs/content/contracts/content-model.md); `tests/zod` type-checks it |
| The commands' JSON reports | `crates/ascribe-cli/src/report/`, `ascribe-diff`, `ascribe-sources` | The [JSON report contract](docs/content/contracts/json-reports.md), with each one's schema, and the [command reference](docs/content/reference/cli.md) |
| Diagnostics: codes, severities, messages | `tests/conformance/diagnostics.toml`, through `ascribe_check::Registry` | The [diagnostics reference](docs/content/reference/diagnostics.md), generated from the registry |

The names the site output, the HTML report, and review put on a page (elements, attributes, classes, and ids) have one home, `ascribe_core::names` (`crates/ascribe-core/src/names.rs`). The TypeScript imports them from a module generated from it in each package (`packages/astro/src/names.ts`, `packages/elements/src/names.ts`, `packages/review/src/names.ts`, and `packages/vscode/src/names.ts`), and `crates/ascribe-core/tests/names.rs` fails on a literal of one anywhere else in source, and on a stylesheet, Astro template, or the docs site's code using a name that isn't declared.

The JSON that Rust writes and TypeScript reads has one home too: the Rust types that write it. `crates/ascribe-cli/src/shapes.rs` derives a JSON Schema from each (`schemars`, through the `json-schema` features of `ascribe-diff`, `ascribe-sources`, and `ascribe-lsp`, which only that test turns on), writes them to `schemas/`, and generates from them the TypeScript types each package reads them as: `packages/astro/src/shapes.ts` and `packages/review/src/shapes.ts` (`ascribe diff`'s JSON, and the HTML report's data) and `packages/vscode/src/shapes.ts` (the language server's answers to the custom requests). The commands' schemas are published in the [JSON report contract](docs/content/contracts/json-reports.md). Every field needs a doc comment, which becomes its description.

`SPEC.md`, the contracts in `docs/content/contracts/`, the commands' JSON, and the published packages' APIs are fixed lines: code behind them can change, and they don't change without a decision to change them.

## Tests

`cargo test --workspace --locked` runs every Rust test; `pnpm test` runs every JS unit test.

| Kind | Where |
|---|---|
| Unit tests | In each crate's source, and each package's `test/` |
| Integration tests | Each crate's `tests/` |
| Conformance cases, written from the spec | `tests/conformance/cases/`, run by `cargo test -p ascribe-conformance` |
| Snapshots (`insta`) | `crates/ascribe-emit/tests/snapshots/` and `crates/ascribe-cli/tests/snapshots/`. Review a change with `cargo insta review` |
| Property tests (`proptest`) | `ascribe-core`, `ascribe-syntax`, `ascribe-resolve`, `ascribe-fmt`, `ascribe-check` |
| Two implementations compared | `crates/ascribe-syntax/tests/agreement.rs` (two parsers), `crates/ascribe-resolve/tests/incremental_differential.rs` and `crates/ascribe-lsp/tests/differential.rs` (incremental against from scratch), `crates/ascribe-diff/tests/reach.rs` (`diff`'s short path against comparing every page), `crates/ascribe-check/tests/parity.rs` (checks against the index), `crates/ascribe-cli/tests/lsp_parity.rs` (the command against the server), and the renderers above |
| The command's output, as files the docs show | `crates/ascribe-cli/tests/output.rs`, writing `crates/ascribe-cli/tests/output/` |
| Browser tests | `packages/elements/test/`, in Chromium, Firefox, and WebKit |
| The editor | `packages/vscode/test/integration/`, in a real VS Code (CI only) |
| The site end to end | `examples/astro-site/test/e2e/`, with the packed npm packages |
| Speed | `tests/corpora/benches/perf.rs` and `crates/ascribe-resolve/benches/incremental.rs`; the weekly Corpora workflow fails a regression against `tests/corpora/baselines/perf.json` |

### `ASCRIBE_BLESS=1`

Some files are generated from a source, or copied from what the code writes, and a test fails when they're stale. Run that test with `ASCRIBE_BLESS=1` and it rewrites them instead; then read the diff. The tests that do this:

| Test | Rewrites |
|---|---|
| `tests/conformance/tests/docs.rs` | The diagnostics reference's fragments in `docs/content/_generated/` |
| `crates/ascribe-cli/src/docs.rs` | Each command's options in `docs/content/_generated/` |
| `crates/ascribe-cli/src/shapes.rs` | The JSON Schemas in `schemas/`, the TypeScript types in `packages/astro/src/shapes.ts`, `packages/review/src/shapes.ts`, and `packages/vscode/src/shapes.ts`, and the schemas' fragments in `docs/content/_generated/`, from the Rust types that write the JSON |
| `crates/ascribe-cli/tests/output.rs` | The command output in `crates/ascribe-cli/tests/output/` |
| `crates/ascribe-core/tests/names.rs` | The names Ascribe puts on a page, `packages/astro/src/names.ts`, `packages/elements/src/names.ts`, `packages/review/src/names.ts`, and `packages/vscode/src/names.ts`, from `crates/ascribe-core/src/names.rs` |
| `crates/ascribe-emit/tests/site_anchors.rs` | The site-render fixtures' inputs and corpus in `tests/render/` |
| `crates/ascribe-emit/tests/zod.rs` | The generated schemas in `tests/zod/generated/` |
| `packages/vscode/test/unit/docs.test.ts` | The extension's settings and commands in `docs/content/_generated/` |

The corpora's recorded counts in `tests/corpora/baselines/` have their own variable, `ASCRIBE_CORPORA_BLESS=1` ([tests/corpora/README.md](tests/corpora/README.md)).

## CI

`.github/workflows/ci.yml` decides what a change needs and runs `rust.yml`, `js.yml`, and `site.yml`; the one required check, `all checks`, waits for them. `drift.yml` and `review.yml` report on pull requests that change the docs. `site-npm.yml` builds the docs with the published canary, after each canary and after a merge to `main` that touches the docs or the site. `canary.yml` publishes the nightly `next` packages, `release.yml` and `release-build.yml` make a release ([RELEASING.md](RELEASING.md)), and `corpora.yml` runs the benchmarks weekly.
