# Architecture

How Ascribe is put together: what each part is for, how the parts reach each other, and where the tests are. It describes the code as it is. What the language means is in [SPEC.md](SPEC.md), how to contribute is in [CONTRIBUTING.md](CONTRIBUTING.md), and each crate and package has a README with its own details.

The Rust crates keep the project's working name, Tessera. The npm packages are `@ascribed/*`.

## The parts

### Rust crates

All in `crates/`. Each depends only on the crates listed for it in [How the crates depend on each other](#how-the-crates-depend-on-each-other).

| Crate | What it's for |
|---|---|
| [`tessera-core`](crates/tessera-core) | The types every crate shares: spans, paths (`RelPath`), attributes, availability specs, issues and the diagnostic slugs, directive schemas, and the consumer traits (`Slugger`, `Router`). |
| [`tessera-syntax`](crates/tessera-syntax) | Parsing one source file into Ascribe's syntax tree, with exact byte spans, through the fork of comrak. No downstream crate sees comrak's types. |
| [`tessera-model`](crates/tessera-model) | Loading and validating the content model, `ascribe.toml`, into a typed `ContentModel`; also `ascribe.lock`. |
| [`tessera-resolve`](crates/tessera-resolve) | The project graph: the file system (`FileSystem`), the source index, includes and snippets, build resolution (availability, variants, phrases, ids, links, glossary), routes, slugs, and incremental updates. |
| [`tessera-check`](crates/tessera-check) | The checks, file-level and page-level, and loading a project from disk (`tessera_check::Project`). Diagnostics are worded from the registry. |
| [`tessera-emit`](crates/tessera-emit) | The outputs: site, plain, and JSON; the output directory's ownership rules; `render_site_html`; the generated Zod schema. |
| [`tessera-diff`](crates/tessera-diff) | What changed between a git revision and the working tree (`diff`), which pages' examples changed while their words didn't (`drift`), and the static HTML report. |
| [`tessera-sources`](crates/tessera-sources) | Sources in other repositories: `fetch`, `update`, and `status` of the copies and their pins. The only code that reaches another repository. |
| [`tessera-fmt`](crates/tessera-fmt) | The formatter: minimal edits that put Ascribe constructs in canonical form, leaving prose alone. |
| [`tessera-lsp`](crates/tessera-lsp) | The language server, run as `ascribe lsp`. |
| [`tessera-cli`](crates/tessera-cli) | The `ascribe` binary: `check`, `build`, `diff`, `drift`, `fmt`, `sources`, and `lsp`. |
| [`comrak-tessera`](crates/comrak-tessera) | A fork of the comrak CommonMark parser, with Ascribe's block-level changes. Each change is marked `// TESSERA:` and listed in [FORK.md](crates/comrak-tessera/FORK.md). It doesn't use the workspace lints. |

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
| `tests/conformance` | The conformance suite (`tessera-conformance`): cases written from the spec, and `diagnostics.toml`, the diagnostics registry. |
| `tests/commonmark` | The CommonMark spec's examples, run against the fork and the parser. |
| `tests/corpora` | Real documentation sets, the 3,000-page synthetic project (`tests/corpora/synthetic`), and the performance benchmarks with their baselines. |
| `tests/render` | The site-render fixtures both HTML renderers must pass. |
| [`tests/zod`](tests/zod) | Type-checks the generated Zod schemas and validates pages with them. |
| `examples/` | Example projects. `examples/quill` is the complete one most tests use; `examples/astro-site` publishes one with Astro. |
| `docs/`, `site/` | The user docs, an Ascribe project, and the Astro site that publishes them. `site/` installs Ascribe from npm, outside the workspace. |
| `scripts/` | Release scripts (`scripts/release`, with `consumers.ts`, which checks the npm packages as packed), the review and sources fixtures, the comparison of two builds' outputs (`scripts/compare/outputs.ts`), the checks that the READMEs link to real docs pages (`scripts/docs-site`) and that this map's paths and commands exist (`scripts/repo-docs`), and `build-all.ts`. |
| `project-docs/` | Plans. They describe what was intended, not necessarily what is. |

## How the crates depend on each other

Each crate uses the crates listed after it, and no others of ours.

```text
tessera-core       (none)
tessera-syntax     core, comrak-tessera
tessera-model      core
tessera-fmt        core, syntax, model
tessera-resolve    core, syntax, model
tessera-check      core, syntax, model, resolve
tessera-sources    core, model, resolve
tessera-emit       core, syntax, model, resolve
tessera-diff       core, syntax, model, resolve, emit
tessera-lsp        core, syntax, model, resolve, check, emit, diff, fmt
tessera-cli        core, model, resolve, check, emit, diff, sources, fmt, lsp
```

There are no cycles. Every crate's `Cargo.toml` takes our crates from `[workspace.dependencies]` in the root [Cargo.toml](Cargo.toml).

The npm packages depend on each other like this: `review` uses `elements`; `astro` uses `cli`, `elements`, and `review`; the VS Code extension uses `elements` and `review`, and runs the binary.

## Surfaces

Every surface reaches the same core.

| Surface | Where | How it reaches the core |
|---|---|---|
| Command line | `tessera-cli` | Calls the crates. Each subcommand is a module in `crates/tessera-cli/src/commands/`. |
| Language server | `tessera-lsp` | Calls the crates, over an `IncrementalProject` it keeps current as the editor types. |
| VS Code | `packages/vscode` | Starts `ascribe lsp`, one server per project (`packages/vscode/src/registry.ts`). The preview and review use custom requests: `ascribe/preview`, `ascribe/review/setBase`, and `ascribe/review/changes`. |
| Astro | `packages/astro` | Runs the binary: `ascribe build` (`packages/astro/src/run.ts`) and, for review, `ascribe diff` (`packages/astro/src/review/diff.ts`). Then reads the files it wrote. |
| Review | `packages/review` | Reads `ascribe diff --format json`'s data, and GitHub through `gh` (`packages/review/src/github/`). |
| Elements | `packages/elements` | The site output's markup. |
| HTML report | `tessera-diff`, `crates/tessera-diff/src/html/` | In the binary. Its script and stylesheet are built from `review` and `elements` by `pnpm --filter @ascribed/review embed`. |

## From a source file to an output

The steps every surface shares, in order:

1. **Parse.** `tessera_syntax::parse` reads one file into a `ParsedDocument`: blocks, directives with their heads, containers and groups, and the issues the parser finds.
2. **Index.** `tessera_resolve::index_file` records what's true of one file alone: headings and their source ids, includes, links, images, phrases, snippets. It doesn't touch the file system.
3. **Load the graph.** `tessera_resolve::Project::load` indexes every source the `FileSystem` lists and resolves what each reference names.
4. **Expand.** `Project::expand` replaces each `@include` with its target and each `@snippet` with its code. Every block keeps the file and span it was written in.
5. **Resolve for a build.** `BuildResolver` applies availability, the build's modes, phrases, page ids, links to routes, and the glossary, giving a `ResolvedPage` (SPEC §9.2).
6. **Check.** `tessera_check::check_files` runs the file-level checks; `PageChecker` runs the page-level ones on resolved pages. `check_project` and `check_all_builds` do both.
7. **Emit.** A `tessera_emit::Emitter` turns resolved pages into files, and `OutputDir` writes them.

## Loading a project, and reading files

- **Finding `ascribe.toml`.** The commands look in `--config` or the nearest parent (`tessera_check::Project::find_config`, called from `crates/tessera-cli/src/context.rs`). The language server looks from its workspace folders (`find_config` in `crates/tessera-lsp/src/core.rs`). `ascribe fmt` has its own (`crates/tessera-cli/src/commands/fmt.rs`).
- **Loading.** `tessera_check::Project::load` reads the content model (`tessera_model::load`) and the sources, and is what `check` reports on. The page-level checks index it again as a `tessera_resolve::Project` (`crates/tessera-check/src/page/bridge.rs`), unless the caller passes its own index (`PageChecker::with_index`). The commands that resolve builds (`build`, `diff`, `drift`, `sources`) build their own `tessera_resolve::Project` over the same files as well, so they index the project twice. The language server loads with `tessera_model::load_str_in` and `tessera_resolve::IncrementalProject::load`, since it holds unsaved text and updates in place.
- **Reading.** `tessera_resolve::FileSystem` (`crates/tessera-resolve/src/fs.rs`) is where a project's files are read: it knows the content root, which files are sources, the boundary a file must stay inside, exact-case names, and symbolic links. `DiskFs` reads a real project, `MemoryFs` holds one for tests, and `tessera_diff::GitFs` reads one at a commit. Many reads don't go through it; [the inventory](project-docs/optimization/inventory.md#2-file-reading-has-a-home-that-isnt-used) counts them. Examples: the content model's (`crates/tessera-model/src/lib.rs`), `ascribe.lock`'s (`crates/tessera-check/src/project.rs`, `crates/tessera-sources/src/lib.rs`), the output directory's (`crates/tessera-emit/src/store.rs`), the copies of sources in other repositories (`crates/tessera-sources/src/copies.rs`), `fmt`'s (`crates/tessera-cli/src/commands/fmt.rs`), the language server's walk when files change (`crates/tessera-lsp/src/core.rs`), and the images the HTML report inlines (`DiskAssets` in `crates/tessera-diff/src/html/mod.rs`), which skips the boundary, case, and link rules.
- **Git.** `tessera-diff` (`crates/tessera-diff/src/git.rs`) and `tessera-sources` (`crates/tessera-sources/src/remote.rs`) run the `git` executable with a fixed argument list. No git or HTTP library is linked.

## Two HTML renderers

The site output is Markdown with web components, and two things turn it into HTML:

- **Ours,** `render_site_html` in `crates/tessera-emit/src/render/`, used by the editor's preview and the HTML report.
- **The site's,** Astro's Markdown pipeline with our plugin: `packages/astro/src/satteri.ts` for Astro's default processor, `packages/astro/src/rehype.ts` for `unified()`.

They're meant to agree, and these tests hold them to it:

- `tests/render/`: fixtures of site Markdown and the HTML it must render to. `crates/tessera-emit/tests/render_fixtures.rs` runs ours, and `packages/astro/test/render-fixtures.test.ts` runs the plugin with both processors. See [tests/render/README.md](tests/render/README.md).
- `crates/tessera-emit/tests/site_anchors.rs` writes the fixtures' inputs and the corpus from the site emitter, so they stay what the emitter writes.
- `pnpm --filter ascribe-vscode test:parity` compares the editor's preview with the Astro site built from the same project (`packages/vscode/test/parity/`). CI runs it after the Astro end-to-end test.

## Outputs and their contracts

| Output | Written by | Its contract |
|---|---|---|
| Site: Markdown with web components | `tessera_emit::SiteEmitter` | [Site render](docs/content/contracts/site-render.md), and [the elements' contract](packages/elements/CONTRACT.md) |
| Plain Markdown | `tessera_emit::PlainEmitter` | SPEC §9.4, and [crates/tessera-emit/README.md](crates/tessera-emit/README.md) |
| JSON, one document per page | `tessera_emit::JsonEmitter` | [crates/tessera-emit/README.md](crates/tessera-emit/README.md) |
| Where each output's files go, and what Ascribe may replace | `tessera_emit::OutputDir` | [Output layout](docs/content/contracts/output-layout.md) |
| Images and linked files | `crates/tessera-emit/src/assets.rs` | [Assets](docs/content/contracts/assets.md) |
| The Zod schema, `_ascribe/schema.ts` | `crates/tessera-emit/src/zod/` | [crates/tessera-emit/README.md](crates/tessera-emit/README.md#zod), with each field type's schema in the [content model contract](docs/content/contracts/content-model.md); `tests/zod` type-checks it |
| The commands' JSON reports | `crates/tessera-cli/src/report/`, `tessera-diff`, `tessera-sources` | The [command reference](docs/content/reference/cli.md) |
| Diagnostics: codes, severities, messages | `tests/conformance/diagnostics.toml`, through `tessera_check::Registry` | The [diagnostics reference](docs/content/reference/diagnostics.md), generated from the registry |

The names the site output, the HTML report, and review put on a page (elements, attributes, classes, and ids) have one home, `tessera_core::names` (`crates/tessera-core/src/names.rs`). The TypeScript imports them from a module generated from it in each package (`packages/astro/src/names.ts`, `packages/elements/src/names.ts`, `packages/review/src/names.ts`, and `packages/vscode/src/names.ts`), and `crates/tessera-core/tests/names.rs` fails on a literal of one anywhere else in source, and on a stylesheet, Astro template, or the docs site's code using a name that isn't declared.

`SPEC.md`, the contracts in `docs/content/contracts/`, the commands' JSON, and the published packages' APIs are fixed lines: code behind them can change, and they don't change without a decision to change them.

## Tests

`cargo test --workspace --locked` runs every Rust test; `pnpm test` runs every JS unit test.

| Kind | Where |
|---|---|
| Unit tests | In each crate's source, and each package's `test/` |
| Integration tests | Each crate's `tests/` |
| Conformance cases, written from the spec | `tests/conformance/cases/`, run by `cargo test -p tessera-conformance` |
| Snapshots (`insta`) | `crates/tessera-emit/tests/snapshots/` and `crates/tessera-cli/tests/snapshots/`. Review a change with `cargo insta review` |
| Property tests (`proptest`) | `tessera-core`, `tessera-syntax`, `tessera-resolve`, `tessera-fmt`, `tessera-check` |
| Two implementations compared | `crates/tessera-syntax/tests/agreement.rs` (two parsers), `crates/tessera-resolve/tests/incremental_differential.rs` and `crates/tessera-lsp/tests/differential.rs` (incremental against from scratch), `crates/tessera-check/tests/parity.rs` (checks against the index), `crates/tessera-cli/tests/lsp_parity.rs` (the command against the server), and the renderers above |
| The command's output, as files the docs show | `crates/tessera-cli/tests/output.rs`, writing `crates/tessera-cli/tests/output/` |
| Browser tests | `packages/elements/test/`, in Chromium, Firefox, and WebKit |
| The editor | `packages/vscode/test/integration/`, in a real VS Code (CI only) |
| The site end to end | `examples/astro-site/test/e2e/`, with the packed npm packages |
| Speed | `tests/corpora/benches/perf.rs` and `crates/tessera-resolve/benches/incremental.rs`; the weekly Corpora workflow fails a regression against `tests/corpora/baselines/perf.json` |

### `ASCRIBE_BLESS=1`

Some files are generated from a source, or copied from what the code writes, and a test fails when they're stale. Run that test with `ASCRIBE_BLESS=1` and it rewrites them instead; then read the diff. The tests that do this:

| Test | Rewrites |
|---|---|
| `tests/conformance/tests/docs.rs` | The diagnostics reference's fragments in `docs/content/_generated/` |
| `crates/tessera-cli/src/docs.rs` | Each command's options in `docs/content/_generated/` |
| `crates/tessera-cli/tests/output.rs` | The command output in `crates/tessera-cli/tests/output/` |
| `crates/tessera-core/tests/names.rs` | The names Ascribe puts on a page, `packages/astro/src/names.ts`, `packages/elements/src/names.ts`, `packages/review/src/names.ts`, and `packages/vscode/src/names.ts`, from `crates/tessera-core/src/names.rs` |
| `crates/tessera-emit/tests/site_anchors.rs` | The site-render fixtures' inputs and corpus in `tests/render/` |
| `crates/tessera-emit/tests/zod.rs` | The generated schemas in `tests/zod/generated/` |
| `packages/vscode/test/unit/docs.test.ts` | The extension's settings and commands in `docs/content/_generated/` |

The corpora's recorded counts in `tests/corpora/baselines/` have their own variable, `ASCRIBE_CORPORA_BLESS=1` ([tests/corpora/README.md](tests/corpora/README.md)).

## CI

`.github/workflows/ci.yml` decides what a change needs and runs `rust.yml`, `js.yml`, and `site.yml`; the one required check, `all checks`, waits for them. `drift.yml` and `review.yml` report on pull requests that change the docs. `site-npm.yml` builds the docs with the published canary, after each canary and after a merge to `main` that touches the docs or the site. `canary.yml` publishes the nightly `next` packages, `release.yml` and `release-build.yml` make a release ([RELEASING.md](RELEASING.md)), and `corpora.yml` runs the benchmarks weekly.
