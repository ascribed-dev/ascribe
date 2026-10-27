# Architecture

How Ascribe is put together: what each part is for, how the parts reach each other, and where the tests are. It describes the code as it is. What the language means is in [SPEC.md](SPEC.md), how to contribute is in [CONTRIBUTING.md](CONTRIBUTING.md), and each crate and package has a README with its own details.

The Rust crates are `ascribe-*` and the npm packages `@ascribed/*`. Until October 2026 the crates were `tessera-*`, the project's working name, so older plans and the changelog use it.

## The parts

### Rust crates

All in `crates/`. Each depends only on the crates listed for it in [How the crates depend on each other](#how-the-crates-depend-on-each-other).

| Crate | What it's for |
|---|---|
| [`ascribe-core`](crates/ascribe-core) | The types every crate shares: spans, paths (`RelPath`), attributes and code blocks' info strings, availability specs, issues and the diagnostic slugs, directive schemas, the consumer traits (`Slugger`, `Router`), `SourceBoundary`, the link rule for a crate that walks the disk itself, and `Visited`, which keeps a walk that follows links to going into each folder once. |
| [`ascribe-syntax`](crates/ascribe-syntax) | Parsing one source file into Ascribe's syntax tree, with exact byte spans, through the fork of comrak. No downstream crate sees comrak's types. |
| [`ascribe-model`](crates/ascribe-model) | Loading and validating the content model, `ascribe.toml`, into a typed `ContentModel`; also `ascribe.lock`, the permalinks list, `ascribe.permalinks` (`PermalinkList`), and a source's tested examples: `ascribe.examples.toml` and the observations `ascribe observe` writes (`crates/ascribe-model/src/examples.rs`), which it reads and writes without running anything. |
| [`ascribe-resolve`](crates/ascribe-resolve) | The project graph: the file system (`FileSystem`), the source index, includes and snippets, build resolution (availability, variants, phrases, ids, links, glossary), routes, slugs, and incremental updates; what a memo file's `about` and a target of `ascribe refs` name, by SPEC's rule `about-target` (`Named`, `crates/ascribe-resolve/src/named.rs`), and memo files, read from `memos/` and placed against the index (`crates/ascribe-resolve/src/memos.rs`). |
| [`ascribe-check`](crates/ascribe-check) | The checks, file-level and page-level, the content checks across the project (`crates/ascribe-check/src/across.rs`, with the evidence their prompts carry in `crates/ascribe-check/src/evidence.rs`), the programs it runs and reads the answers of (`crates/ascribe-check/src/tool.rs`: Vale in `prose/`, lychee in `links.rs`, with the answers it reuses in `crates/ascribe-check/src/links/cache.rs` and what it remembers of each link in `crates/ascribe-check/src/links/history.rs`, `afdocs` in `site.rs`), a finding's identity across runs (`crates/ascribe-check/src/identity.rs`) and what the report remembers of its problems and the site's checks (`crates/ascribe-check/src/report_history.rs`), and loading a project from disk (`ascribe_check::Project`). Diagnostics are worded from the registry. |
| [`ascribe-emit`](crates/ascribe-emit) | The outputs: site, plain, and JSON; the output directory's ownership rules; `render_site_html`; the generated Zod schema. |
| [`ascribe-diff`](crates/ascribe-diff) | What changed between a git revision and the working tree (`diff`), which pages' examples changed while their words didn't (`drift`), and the static HTML preview. |
| [`ascribe-sources`](crates/ascribe-sources) | Sources in other repositories: `fetch`, `update`, and `status` of the copies and their pins. The only code that reaches another repository. |
| [`ascribe-fmt`](crates/ascribe-fmt) | The formatter: minimal edits that put Ascribe constructs in canonical form, leaving prose alone. |
| [`ascribe-query`](crates/ascribe-query) | Answers about a project that change nothing: a diagnostic explained, the content model, a page's outline, a link, a page rendered for one build, where something is used, the project's rules for an agent, the report on what state a project is in (`report`), and a project's release lines and what differs between two (`versions`). |
| [`ascribe-edit`](crates/ascribe-edit) | The edits the commands share with the language server, which need no client: `rename`, a release (`release`), and retiring a line (`retire_line`), with the editor's rename and file moves they're made of (`refactor`), and the edits to `ascribe.toml` (`model_file`), `ascribe.nav` (`navigation`), and the memos. Each works from a `nav::Ctx`, which the server makes for each request and a command loads from the disk (`project::load`), and returns each file's text rather than writing. |
| [`ascribe-lsp`](crates/ascribe-lsp) | The language server, run as `ascribe lsp`, and `Watched`, which the agents' hook shares with it. |
| [`ascribe-mcp`](crates/ascribe-mcp) | The Model Context Protocol over standard input and output, for `ascribe mcp`: messages, revisions, and errors. What the server offers is the binary's (`crates/ascribe-cli/src/mcp/`), since each tool returns what its command writes. |
| [`ascribe-templates`](crates/ascribe-templates) | The page templates Ascribe ships: The Good Docs Project's, converted into Ascribe pages by `scripts/templates/convert.ts` and committed in `set/`, with `set.toml` listing them; writing them into a project, with their types, for `ascribe templates eject` (`crates/ascribe-templates/src/eject.rs`); saying what's changed in the set since a project's copy, for `ascribe templates status` (`crates/ascribe-templates/src/status.rs`); and starting a page from its type's template, for the `new-page` prompt and the editor's New Page (`crates/ascribe-templates/src/start.rs`). |
| [`ascribe-cli`](crates/ascribe-cli) | The `ascribe` binary: `check`, `build`, `diff`, `drift`, `fmt`, `sources`, `lsp`, `mcp`, `agents`, `report`, `rename`, and the commands that answer questions (`explain`, `model`, `outline`, `link`, `refs`, `resolve`). |
| [`comrak-ascribe`](crates/comrak-ascribe) | A fork of the comrak CommonMark parser, with Ascribe's block-level changes. Each change is marked `// ASCRIBE:` and listed in [FORK.md](crates/comrak-ascribe/FORK.md). It doesn't use the workspace lints. |

### npm packages

All in `packages/`, a pnpm workspace with `examples/astro-site` and `tests/zod`.

| Package | What it's for |
|---|---|
| [`@ascribed/cli`](packages/cli) | The `ascribe` command for npm. It finds and runs the native binary from one of the `@ascribed/cli-<platform>` packages (`packages/cli/platforms/`). |
| [`@ascribed/astro`](packages/astro) | The Astro integration: runs `ascribe build`, loads the site output as a content collection, renders its markers, gives each permalink an address that forwards to it, and serves review in `astro dev`. |
| [`@ascribed/elements`](packages/elements) | The custom elements the site output uses, to the letter of [CONTRACT.md](packages/elements/CONTRACT.md). |
| [`@ascribed/review`](packages/review) | Reviewing pages as readers see them: marking changed blocks, and placing a pull request's review threads beside them. |
| [Ascribe for VS Code](packages/vscode) (`ascribe-vscode`) | The editor extension: a client of the language server, the page preview, and review in the editor. |
| [`@ascribed/archive`](packages/archive) | An archive's search script, which `ascribe versions export` and `ascribe preview` write from the binary's copy. Private: it isn't published. |

### Everything else

| Directory | What it holds |
|---|---|
| `tests/conformance` | The conformance suite (`ascribe-conformance`): cases written from the spec, and `diagnostics.toml`, the diagnostics registry. |
| `tests/commonmark` | The CommonMark spec's examples, run against the fork and the parser. |
| `tests/corpora` | Real documentation sets, the 3,000-page synthetic project (`tests/corpora/synthetic`), and the performance benchmarks with their baselines. |
| `tests/render` | The site-render fixtures both HTML renderers must pass. |
| `fuzz` | The fuzz targets (`cargo-fuzz`): the parser, the content model's loader, and the formatter on arbitrary text. A crate of its own, outside the workspace, on a dated nightly ([CONTRIBUTING.md](CONTRIBUTING.md#fuzzing)). |
| [`tests/zod`](tests/zod) | Type-checks the generated Zod schemas and validates pages with them. |
| `examples/` | Example projects. `examples/quill` is the complete one most tests use; `examples/astro-site` publishes one with Astro. |
| `docs/`, `site/` | The user docs, an Ascribe project, and the Astro site that publishes them. `site/` installs Ascribe from npm, outside the workspace. |
| [`design/`](design) | The design tokens, `design/tokens.toml`: every color, font stack, space, and radius of Ascribe's stylesheets. The mark and the wordmark as pixel grids (`mark.toml`), in the colors under `color.brand` in the tokens; what Ascribe says it is, the headline, the sentence, its short form, and the keywords (`words.toml`), which every surface describing Ascribe takes or is compared with; the images generated from them, `mark.svg` and `wordmark.svg` among them, are listed in `assets.json`, and the README headers and avatar are in `design/out/`. `design/candidates/` holds the chosen palette and type scale (`chosen.toml`) and the color pairings it's checked against; `design/specimen.html`, generated from them by `scripts/design/specimen.ts`, shows them in light and dark. `design/README.md` says how to change them. |
| `schemas/` | The JSON Schemas of the JSON the commands and the language server write, generated from the Rust types (`crates/ascribe-cli/src/shapes.rs`). |
| `plugins/ascribe/` | The plugin for agents (Claude Code, Copilot's CLI, VS Code, Cursor): the language server, the stop hook, the MCP server, the skill, and commands for the named prompts, generated from the binary (`crates/ascribe-cli/src/agents/plugin.rs`). |
| `scripts/` | Release scripts (`scripts/release`, with `consumers.ts`, which checks the npm packages as packed, and `changes.ts`, which checks the changelog's entries in `changes/` and gathers them into `CHANGELOG.md`), the review and sources fixtures, the comparison of two builds' outputs (`scripts/compare/outputs.ts`), the checks that the READMEs link to real docs pages and that the facts several files repeat (the Node, Rust, and glibc versions, the docs' source folders) agree (`scripts/docs-site`), and that this map's paths and commands exist, and that each decision in `project-docs/decisions/` is well formed (`scripts/repo-docs`), the checks on the workflows (`scripts/ci`: every job has a timeout, and Playwright's container image is the `playwright-core` version installed), the design tokens' generator, the design specimen, the mark's assets, and the check that every surface says what Ascribe is in `design/words.toml`'s words (`scripts/design`), the page templates' conversion from The Good Docs Project (`scripts/templates`), `bless.ts`, which rewrites every generated file, and `build-all.ts`. |
| `project-docs/` | Plans. They describe what was intended, not necessarily what is. |

## How the crates depend on each other

Each crate uses the crates listed after it, and no others of ours.

```text
ascribe-core       (none)
ascribe-syntax     core, comrak-ascribe
ascribe-model      core
ascribe-fmt        core, syntax, model
ascribe-resolve    core, syntax, model
ascribe-emit       core, syntax, model, resolve, comrak-ascribe
ascribe-check      core, syntax, model, resolve, emit
ascribe-sources    core, model, resolve
ascribe-diff       core, syntax, model, resolve, check, emit
ascribe-query      core, model, resolve, check, emit
ascribe-edit       core, syntax, model, resolve, check, emit
ascribe-lsp        core, syntax, model, resolve, check, emit, diff, fmt, templates, edit
ascribe-mcp        (none)
ascribe-templates  core, syntax, model, resolve, check
ascribe-cli        core, model, resolve, check, emit, diff, sources, fmt, query, edit, lsp, mcp, templates
```

There are no cycles. Every crate's `Cargo.toml` takes our crates from `[workspace.dependencies]` in the root [Cargo.toml](Cargo.toml).

The npm packages depend on each other like this: `review` uses `elements`; `astro` uses `cli`, `elements`, and `review`; the VS Code extension uses `elements` and `review`, and runs the binary.

## Surfaces

Every surface reaches the same core.

| Surface | Where | How it reaches the core |
|---|---|---|
| Command line | `ascribe-cli` | Each subcommand is a module in `crates/ascribe-cli/src/commands/`: its arguments, one call into a library ([Each command's entry](#each-commands-entry)), and the report. |
| Language server | `ascribe-lsp` | Calls the crates, over an `IncrementalProject` it keeps current as the editor types. |
| MCP server | `ascribe-mcp`, `crates/ascribe-cli/src/mcp/` | Each tool calls the `answer` function its command calls, and returns the JSON the command writes. Projects are kept loaded between calls (`crates/ascribe-cli/src/mcp/cache.rs`), through the `answer::Projects` the commands take, and loaded again when a listing of their files differs. |
| VS Code | `packages/vscode` | Starts `ascribe lsp`, one server per project (`packages/vscode/src/registry.ts`). The preview and review use custom requests: `ascribe/preview`, `ascribe/review/setBase`, `ascribe/review/changes`, and `ascribe/review/baseFile`. The editor's actions (`packages/vscode/src/actions/`) use `ascribe/context`, `ascribe/targets`, and `ascribe/edit`, and so do New Page and Go to Next Placeholder (`packages/vscode/src/actions/newPage.ts`), Prompt agent (`packages/vscode/src/actions/promptAgent.ts`) uses `ascribe/agentPrompt`, the build lens (`packages/vscode/src/ui/buildLens.ts`) uses `ascribe/buildView`, the Project view's Apply Every Safe Fix and Format Every Page (`packages/vscode/src/ui/projectEdits.ts`) use `ascribe/projectEdit`, and agents' tools (`packages/vscode/src/agents/`) read the published diagnostics, review, and `ascribe/targets`; the extension also offers `ascribe mcp` to VS Code's agents. |
| Astro | `packages/astro` | Runs the binary: `ascribe build` (`packages/astro/src/run.ts`) and, for review, `ascribe diff` (`packages/astro/src/review/diff.ts`). Then reads the files it wrote. |
| Review | `packages/review` | Reads `ascribe diff --format json`'s data, and GitHub through `gh` (`packages/review/src/github/`). |
| Elements | `packages/elements` | The site output's markup. |
| HTML preview | `ascribe-diff`, `crates/ascribe-diff/src/html/` | In the binary. Its script and stylesheet are built from `review` and `elements` by `pnpm --filter @ascribed/review embed`. |
| Archive | `ascribe-emit`, `crates/ascribe-emit/src/archive/` | In the binary, for `ascribe versions export` and `ascribe preview`. Its search script is built from `archive` by `pnpm --filter @ascribed/archive embed`. |

## From a source file to an output

The steps every surface shares, in order:

1. **Parse.** `ascribe_syntax::parse` reads one file into a `ParsedDocument`: blocks, directives with their heads, containers and groups, and the issues the parser finds.
2. **Index.** `ascribe_resolve::index_file` records what's true of one file alone: headings and their source ids, includes, links, images, phrases, snippets. It doesn't touch the file system.
3. **Load the graph.** `ascribe_resolve::Project::load` indexes every source the `FileSystem` lists and resolves what each reference names.
4. **Expand.** `Project::expand` replaces each `@include` with its target and each `@snippet` with its code. Every block keeps the file and span it was written in.
5. **Resolve for a build.** `BuildResolver` applies availability, the build's modes, phrases, page ids, links to routes, and the glossary, giving a `ResolvedPage` (SPEC §9.2).
6. **Check.** `ascribe_check::check_files` runs the file-level checks; `PageChecker` runs the page-level ones on resolved pages. `check_project` and `check_all_builds` do both.
7. **Emit.** An `ascribe_emit::Emitter` turns resolved pages into files, and `OutputDir` writes them.

## Loading a project, and reading files

- **Finding `ascribe.toml`.** The commands look in `--config` or the nearest parent (`ascribe_check::Project::locate`, called from `crates/ascribe-cli/src/context.rs`). The language server looks from its workspace folders (`find_config` in `crates/ascribe-lsp/src/core.rs`).
- **Loading.** `ascribe_check::Project::load` reads the content model (`Project::load_model`) and the sources, and is what `check` reports on; every command loads its project with it, through `load_project` in `crates/ascribe-cli/src/context.rs` (or `load` in `crates/ascribe-cli/src/answer.rs`, which finds `ascribe.toml` from a path on the command line), except `fmt`, which takes only the content model (`Project::load_model`) so that a project whose pages have errors can still be formatted, `model`, which needs nothing else, and `explain`, which needs no project. Loading reads each source once, through any `FileSystem` (`Project::load_from`; `Project::load` passes `DiskFs`), and keeps the listing it read them from. `Project::index` is the one source index, an `ascribe_resolve::Project` built once from that listing and the texts held (`Held` in `crates/ascribe-check/src/page/bridge.rs`), unless the caller gives its own (`Project::with_link_index`, as the language server does): the page-level checks read it, and so do the commands that resolve builds or the source index (`build`, `diff`, `drift`, `sources`, `outline`, `link`, `refs`, `resolve`, `search`, `permalinks`, `versions`), so a command reads each source once and indexes once. `crates/ascribe-cli/tests/all/read_once.rs` counts the reads. The language server loads with `ascribe_model::load_str_in` and `ascribe_resolve::IncrementalProject::load`, since it holds unsaved text and updates in place.
- **Reading.** `ascribe_resolve::FileSystem` (`crates/ascribe-resolve/src/fs.rs`) is where a project's files are read: it knows the content root, which files are sources, the boundary a file must stay inside, exact-case names, and symbolic links. `DiskFs` reads a real project, `MemoryFs` holds one for tests, and `ascribe_diff::GitFs` reads one at a commit. A read anywhere else says why it isn't a project read, in a comment starting `Outside FileSystem:`, and `crates/ascribe-resolve/tests/all/file_reads.rs` fails on one that doesn't. The ones left: the content model's (`crates/ascribe-model/src/lib.rs`, and its checks that the folders it names exist), finding and reading `ascribe.toml` (`Project::locate` and `Project::load_model` in `crates/ascribe-check/src/project.rs`, and `load` in `crates/ascribe-edit/src/project.rs`, for a command's edit), the output directory's (`crates/ascribe-emit/src/store.rs`), writing the copies of sources in other repositories (`crates/ascribe-sources/src/copies.rs`), `fmt`'s (`crates/ascribe-fmt/src/files.rs`, which takes the link rule from `DiskFs` through `ascribe_core::SourceBoundary`, and walks folders as `DiskFs` does, with `ascribe_core::Visited`), the language server's walk when files change (`crates/ascribe-lsp/src/core.rs`, with `Visited` too), the instruction files `ascribe agents sync` reads and writes (`crates/ascribe-cli/src/agents/sync.rs`), the MCP server's listing of a project's files to see whether it changed, and its search for projects (`crates/ascribe-cli/src/mcp/`), which `ascribe preview` lists them with too, the files `ascribe preview`'s server reads from the folder it wrote (`crates/ascribe-cli/src/commands/preview/serve.rs`), the hook's check server and its files in the cache folder (`crates/ascribe-cli/src/agents/checker.rs`), the link answers `ascribe report links` keeps under `.ascribe/cache/links/` and its histories in `.ascribe/history/links.json` and `report.json` (`crates/ascribe-check/src/links/cache.rs` and `history.rs`, and `crates/ascribe-check/src/report_history.rs`, which only the report's code opens, as `crates/ascribe-check/tests/all/link_cache.rs` holds), the list and the names `ascribe permalinks verify` reads, which need no project, and the list `sync` and `retire` write, and `ascribe.toml`, which `retire` writes (`crates/ascribe-cli/src/commands/permalinks.rs`), `ascribe.baseline`, which `ascribe baseline` writes (`crates/ascribe-cli/src/commands/baseline.rs`), the templates, `templates.lock`, and `ascribe.toml` that `ascribe templates eject` writes (`crates/ascribe-templates/src/eject.rs`), reading what's there through `DiskFs`, and the first `ascribe.toml` and content root `ascribe init` writes (`crates/ascribe-templates/src/init.rs`), and assets: the build copies them from `EmitContext::asset_source`, and the HTML preview inlines them from the same place (`DiskAssets` in `crates/ascribe-diff/src/html/mod.rs`), both skipping the boundary, case, and link rules.
- **Side files.** The files beside `ascribe.toml` that aren't sources (`ascribe.lock`, `ascribe.permalinks`, `ascribe.nav`, `ascribe.addresses`, and `ascribe.baseline`) are each a `SideFile` (`crates/ascribe-resolve/src/side_file.rs`): its path, the id its locations carry, how it reads, and what a change to it makes stale (`Stale`). `SIDE_FILES` in `crates/ascribe-check/src/side_files.rs` lists them, and `ascribe_check::Project` reads each from it, the language server follows each by its `Stale` (`crates/ascribe-lsp/src/core.rs`) and publishes what the checks across the project find at each, and a report's file table names each. The source index reads the ones whose `Stale::index` is set, through `side_file::Indexed`, and takes a change to them as `Change::SideFiles`. Adding a side file is its module's `SideFile`, an entry in `SIDE_FILES` (and a field of `Indexed` when the index reads it), and its checks.
- **Paths.** `ascribe_core::path` holds the path helpers every crate shares: `RelPath::relative_to` (a path inside a folder), `relative_path` (from one directory on disk to another, with a leading drive letter matched in either case), and `normalize`.
- **Git.** `ascribe-diff` (`crates/ascribe-diff/src/git.rs`) and `ascribe-sources` (`crates/ascribe-sources/src/remote.rs`) run the `git` executable with a fixed argument list. No git or HTTP library is linked.

## Each command's entry

Each command is one call into a library, after the project is loaded. The call takes plain arguments (the project, and options) and returns a typed result; the CLI parses the arguments, prints the result, and chooses the exit code. A tool that wraps a command calls the same function.

| Command | Entry | Notes |
|---|---|---|
| `check` | `ascribe_check::Request`: `find`, then `settle` | A `Request` names the builds (or the editor's build), the scope, whether the baseline applies, and whether Vale ran. `find` checks them (`diagnose`, or `diagnose_editor_build_in`); the CLI adds Vale's findings and the addresses compared with the base; `settle` sets every finding at the levels `[checks]` sets, holds back the advice `ascribe.baseline` lists (`Project::baseline`) unless `--no-baseline`, and keeps what counts for the paths (`Scope::of_paths`, `Scope::report`). With `--stdin`, `Project::with_source` lays the text over the project first. `build`, `report`, the agents' hook and MCP tool, and the language server go through the same `Request`, so no surface applies the baseline or the scope on its own. With `--format prompt`, `ascribe_check::prompt` writes what's reported as a prompt for an agent, as the language server's `ascribe/agentPrompt` does. |
| `build` | `ascribe_check::diagnose_with`, then `ascribe_emit::write_outputs` | The report is printed between the two, and a build with errors stops there. `write_outputs` reports each output as it's written through a callback. |
| `preview` | `check::run_check`, then `ascribe_emit::preview` | Prints `check`'s report for the build, then renders it with the archive's renderer, errors or not, and writes it into `.ascribe/preview/` with `OutputDir::replace_archive`. Serves that folder at `127.0.0.1` from `crates/ascribe-cli/src/commands/preview/serve.rs`, the binary's one socket (decision #981), and renders again when the MCP server's listing of the project's files (`mcp::cache::watched`) changes. |
| `baseline` | `ascribe_check::baseline::write`, `prune` | Run `check`'s own call first, without the baseline, for every build, and take its diagnostics, with `Project::baseline`. Each returns the file's text, from `baseline::text`, and the CLI writes it. |
| `diff` | `ascribe_diff::diff_project` | Returns the report, with the working tree's error count; `ProjectDiff::html` renders it for `--format html`, and `ProjectDiff::pages_prompt` and `prompt_about` write `--format prompt` (`ascribe_diff::prompt`), as the language server's `ascribe/agentPrompt` does for `pageChanges` and `fragmentReach`. |
| `drift` | `ascribe_diff::drift_project` | |
| `fmt` | `ascribe_fmt::format_files` | Takes the content model from `Project::load_model`, and reports each file changed, with its edits, through a callback; `--format json` writes them. |
| `sources` | `ascribe_sources::fetch`, `status`, `update` | Over an `ascribe_sources::Workspace`. `update` then calls `drift_project` against `HEAD` for the pages whose examples changed. |
| `permalinks` | `ascribe_query::permalinks::sync`, `list`, `verify`, `retiring` | Over `Project::index`, with the list `ascribe_check::Project` reads and the model's `[permalinks]`. `sync` returns the list's text, and the CLI writes it. `verify` needs only the list's text. `retire` finds the names with `retiring`, records each in `ascribe.addresses` (`crates/ascribe-cli/src/commands/addresses.rs`), and in the list with `record_retired`. The checks of the list are `ascribe_check`'s (`crates/ascribe-check/src/permalinks.rs`), run once for every build and by `check_files`, and the language server's quick fix writes what `sync` does. |
| `versions` | `ascribe_query::versions::list`, `changes`; `ascribe_edit::release`, `retire_line` | `list` and `changes` read the list and the marks, over `Project::index`, and build nothing; `changes` asks `ascribe_resolve::in_release_line` whether a mark's content is in a line. `release` plans the edits to `ascribe.toml` with `ascribe_edit::model_file::ModelFile`, as the editor's edits to it do, rewrites the marks in the sources, and builds the project before and after in memory (`ascribe_emit::outputs_in_memory`) to compare. `retire` does the same: it archives the line with `ModelFile`, rewrites each mark for the target to the simplest spec that means the same in the lines left (`ascribe_core::availability::without_versions_through`, which a line's output shows a spec through too), and compares the lines' outputs before and after, and lists the content no line has and where each of the line's addresses should go. Each returns each file's text; the CLI asks `ascribe_diff::Repository::not_restorable` whether git can put the files back, and writes them. |
| `templates` | `ascribe_templates::set`, `eject`, `status` | `list` reads only the set the binary carries. `eject` takes the content model from `Project::load_model`, looks at what's there through `DiskFs`, asks `ascribe_check::type_files::folder_problem` whether the folder can hold a type's files, and writes the files, Ascribe's notes on each type from `NOTES` with them, `templates.lock`, and `ascribe.toml`, whose types it adds at the end of the text, and names the pages that name no type, read through `DiskFs`. `status` takes the model as written, so a template that's gone is reported rather than stopping it, reads the files and `templates.lock` through `DiskFs`, and compares them with `Upstream::carried()`, the set the binary carries; `diff` gives `--diff`'s unified diff. |
| `observe` | `crates/ascribe-cli/src/commands/observe.rs` | Reads the current folder's `ascribe.examples.toml` with `ascribe_model::ExamplesConfig`, runs its commands through the platform's shell, and writes `Observation::to_toml`. It's the one command that runs a line of shell a project wrote, so that code is the binary's, and `crates/ascribe-cli/tests/all/program_starts.rs` lists every place the workspace starts a program and fails on a new one. The docs side reads the files in `crates/ascribe-resolve/src/snippet/examples.rs`, where a snippet's status and tested line come from. |
| `init` | `ascribe_templates::init` | Refuses where `ascribe_check::Project::find_config` finds an `ascribe.toml` at or above the folder. Writes `MODEL_TEXT`, the first `ascribe.toml` (`crates/ascribe-templates/first.ascribe.toml`), and makes its content root; with `--templates`, it calls `eject` with that text instead, which writes the file with the six types added. |
| `lsp` | `ascribe_lsp::run_stdio` | |
| `mcp` | `ascribe_mcp::serve` | With `crate::mcp::Server`, whose tools call each command's `answer` function (`crates/ascribe-cli/src/mcp/tools.rs`). |
| `explain` | `ascribe_query::explain`, `list` | Needs no project. |
| `model` | `ascribe_query::model`, `summary` | Takes the content model from `Project::load_model`. |
| `outline` | `ascribe_query::outline` | Over `Project::index`, as the rest below. |
| `link` | `ascribe_query::link` | |
| `refs` | `ascribe_query::refs` | The CLI adds `next_command` when the list is cut. |
| `rename` | `ascribe_edit::rename` | Loads the project as the language server does (`crates/ascribe-edit/src/project.rs`) and makes the edits of its rename and of a file's move (`crates/ascribe-edit/src/refactor.rs`), asked by name instead of at a position. Returns each file's text after, and the CLI writes them and moves the page. |
| `resolve` | `ascribe_query::resolve` | |
| `search` | `ascribe_query::search` | Chooses the builds (`select_builds`), resolves every page each publishes, reads each with `ascribe_resolve::reading::Reading`, and gathers the places with `reading::find`, merged by line across pages and builds. `ascribe diff` takes each block's text from the same reading (`reading::own_text`), with HTML blocks and directive lines that search never matches. The CLI adds the command that lists them all when the list is cut. |
| `report` | `ascribe_query::report` | Takes the programs it runs as `report::Tools`, `Tools::installed()` in the binary and stand-ins in tests. The CLI writes the JSON (`crates/ascribe-cli/src/commands/report/json.rs`), whose findings are `check`'s diagnostics, and builds `--format prompt` with `ascribe_check::prompt::batch`. |
| `agents sync` | `ascribe_query::rules` | The files each agent reads, and the skill, are the binary's: `crates/ascribe-cli/src/agents/`, the one place that knows about particular agents. It finds the repository's root with `ascribe_diff::Repository`. |
| `agents rules` | `ascribe_query::rules` | The block `agents sync` writes beside `ascribe.toml` (`agents::sync::rules_beside`). |
| `agents index` | `ascribe_query::agents_index` | The index is `ascribe_resolve::llms::compact_index`, from the build's page records, as `llms.txt` is; the pages are the `plain` output's. The CLI writes them (`crates/ascribe-cli/src/agents/index.rs`), the index between markers as `sync` writes its block. |
| `agents prompt` | `crates/ascribe-cli/src/agents/prompts.rs` | Each prompt is built from a command's: `new-page` from `ascribe_templates::start`, `ascribe_query::model`, and `ascribe_check::prompt::compose`, `fix` from `check --format prompt`, and `review` from `diff --format prompt`. |
| `agents hook` | `ascribe_check::diagnose_editor_build_in`, `check::run_check`, `ascribe_lsp::Watched` | Reads each harness's input and writes its output in `crates/ascribe-cli/src/agents/hook.rs`; checks in `crates/ascribe-cli/src/agents/checker.rs`, through the check server (`agents hook-server`, hidden) that keeps the project loaded and talks to hooks through files in the user's cache folder. |

The language server calls the same code where it does the same job: `ascribe_fmt::format` for formatting, and `ascribe_diff::compare_builds`, which `diff_project` calls, for review. It checks incrementally, so it calls `check_file` and `PageChecker` instead of `diagnose`, and `crates/ascribe-cli/tests/all/lsp_parity.rs` holds the two to the same diagnostics.

When an entry can't do its work, it returns an error type, never a string. Each error implements `ascribe_core::Coded`: its `code()` is a short lowercase identifier (`unknown_build`, `git_not_found`) that names the failure whatever the message says, and an error that wraps another has the inner one's code. `crates/ascribe-cli/src/exit.rs` lists every code once, in a test, and turns an error into the exit code the command gives (`2`, for every one). Libraries return errors and don't print: `clippy::print_stdout` and `clippy::print_stderr` are denied workspace-wide, and allowed only in the CLI, the benchmarks, the test harnesses, and the language server's log (`crates/ascribe-lsp/src/log.rs`).

## Two HTML renderers

The site output is Markdown with web components, and two things turn it into HTML:

- **Ours,** `render_site_html` in `crates/ascribe-emit/src/render/`, used by the editor's preview and the HTML preview. It renders with the comrak fork, the parser `ascribe-syntax` uses, with Ascribe's option off.
- **The site's,** Astro's Markdown pipeline with our plugin: `packages/astro/src/satteri.ts` for Astro's default processor, `packages/astro/src/rehype.ts` for `unified()`.

They're meant to agree, and these tests hold them to it:

- `tests/render/`: fixtures of site Markdown and the HTML it must render to. `crates/ascribe-emit/tests/all/render_fixtures.rs` runs ours, and `packages/astro/test/render-fixtures.test.ts` runs the plugin with both processors. See [tests/render/README.md](tests/render/README.md).
- `crates/ascribe-emit/tests/all/site_anchors.rs` writes the fixtures' inputs and the corpus from the site emitter, so they stay what the emitter writes.
- `pnpm --filter ascribe-vscode test:parity` compares the editor's preview with the Astro site built from the same project (`packages/vscode/test/parity/`). CI runs it after the Astro end-to-end test.

## Outputs and their contracts

| Output | Written by | Its contract |
|---|---|---|
| Site: Markdown with web components | `ascribe_emit::SiteEmitter` | [Site render](docs/content/contracts/site-render.md), and [the elements' contract](packages/elements/CONTRACT.md) |
| Plain Markdown | `ascribe_emit::PlainEmitter` | SPEC §9.4, and [crates/ascribe-emit/README.md](crates/ascribe-emit/README.md) |
| A build's page records, `_ascribe/pages.json` | `ascribe_resolve::pages`, written by `SiteEmitter` | [Output layout](docs/content/contracts/output-layout.md) §1.3 |
| The headers a build's Markdown versions need, `_ascribe/headers.json` | `ascribe_resolve::pages::header_rules`, from the page records, written by `SiteEmitter`; `@ascribed/astro` writes them into `_headers` | [Output layout](docs/content/contracts/output-layout.md) §1.4 |
| `llms.txt`, with `[consumer] agents = true` | `ascribe_resolve::llms`, from the page records, written by `PlainEmitter` | SPEC §9.4, and the [content model contract](docs/content/contracts/content-model.md) §15; `examples/astro-site/test/e2e/agent-docs.test.ts` runs the delivery spec's checker on the built example |
| JSON, one document per page | `ascribe_emit::JsonEmitter` | [crates/ascribe-emit/README.md](crates/ascribe-emit/README.md) |
| Where each output's files go, and what Ascribe may replace | `ascribe_emit::OutputDir` | [Output layout](docs/content/contracts/output-layout.md) |
| Images and linked files | `crates/ascribe-emit/src/assets.rs` | [Assets](docs/content/contracts/assets.md) |
| The Zod schema, `_ascribe/schema.ts` | `crates/ascribe-emit/src/zod/` | [crates/ascribe-emit/README.md](crates/ascribe-emit/README.md#zod), with each field type's schema in the [content model contract](docs/content/contracts/content-model.md); `tests/zod` type-checks it |
| The commands' JSON reports | `crates/ascribe-cli/src/report/`, `ascribe-diff`, `ascribe-sources` | The [JSON report contract](docs/content/contracts/json-reports.md), with each one's schema, and the [command reference](docs/content/command/index.md) |
| Diagnostics: codes, severities, messages | `tests/conformance/diagnostics.toml`, through `ascribe_check::Registry` | The [diagnostics reference](docs/content/checks/diagnostics.md), generated from the registry |

The names the site output, the HTML preview, and review put on a page (elements, attributes, classes, and ids) have one home, `ascribe_core::names` (`crates/ascribe-core/src/names.rs`). The TypeScript imports them from a module generated from it in each package (`packages/astro/src/names.ts`, `packages/elements/src/names.ts`, `packages/review/src/names.ts`, and `packages/vscode/src/names.ts`), and `crates/ascribe-core/tests/names.rs` fails on a literal of one anywhere else in source, and on a stylesheet, Astro template, or the docs site's code using a name that isn't declared.

The JSON that Rust writes and TypeScript reads has one home too: the Rust types that write it. `crates/ascribe-cli/src/shapes.rs` derives a JSON Schema from each (`schemars`, through the `json-schema` features of `ascribe-diff`, `ascribe-sources`, and `ascribe-lsp`, which only that test turns on), writes them to `schemas/`, and generates from them the TypeScript types each package reads them as: `packages/astro/src/shapes.ts` and `packages/review/src/shapes.ts` (`ascribe diff`'s JSON, and the HTML preview's data) and `packages/vscode/src/shapes.ts` (the language server's answers to the custom requests). The commands' schemas are published in the [JSON report contract](docs/content/contracts/json-reports.md). Every field needs a doc comment, which becomes its description.

The colors, font stacks, spaces, and radii of Ascribe's own stylesheets (the element library's, review's marks, overlay, and report, the Astro toolbar's, the docs site's, and the VS Code preview's) have one home too, `design/tokens.toml`. `scripts/design/tokens.ts` generates each stylesheet's variables from it, between marker comments, and `scripts/design/tokens.test.ts` fails on a stale block and on a color written anywhere else in those stylesheets ([design/README.md](design/README.md)).

What Ascribe says it is has one home too, `design/words.toml`: the headline, the sentence, its short form, and the keywords, from [the story](project-docs/brand/story.md). A surface that can take them is generated from it, and `scripts/design/words.test.ts` compares every other surface with it ([design/README.md](design/README.md#the-words)).

`SPEC.md`, the contracts in `docs/content/contracts/`, the commands' JSON, and the published packages' APIs are fixed lines: code behind them can change, and they don't change without a decision to change them.

## Tests

`cargo test --workspace --locked` runs every Rust test; `pnpm test` runs every JS unit test.

| Kind | Where |
|---|---|
| Unit tests | In each crate's source, and each package's `test/` |
| Integration tests | Each crate's `tests/`. Most crates compile theirs as one program, `all.rs`, which declares a module per file in the `all/` folder beside it (`crates/ascribe-lsp/tests/all.rs`, say): one link and one process instead of one per file. Run one file's tests with its module as the filter, `cargo test -p ascribe-lsp --test all navigation::`; the `::` keeps a name that's also a word in other tests' names from matching them. `crates/ascribe-cli/tests/determinism.rs` stays a program of its own, as CI runs it in its own job |
| Conformance cases, written from the spec | `tests/conformance/cases/`, run by `cargo test -p ascribe-conformance` |
| Snapshots (`insta`) | A `snapshots/` folder beside the test file: `crates/ascribe-emit/tests/all/snapshots/`, `crates/ascribe-cli/tests/all/snapshots/`, and the others. Review a change with `cargo insta review` |
| Property tests (`proptest`) | `ascribe-core`, `ascribe-syntax`, `ascribe-resolve`, `ascribe-fmt`, `ascribe-check`, `ascribe-diff` |
| Fuzzing (`cargo-fuzz`) | `fuzz/fuzz_targets/`: `parse`, `model`, and `format`, run a few minutes each by the nightly Fuzz workflow, which opens an issue for each crash |
| Two implementations compared | `crates/ascribe-syntax/tests/all/agreement.rs` (two parsers), `crates/ascribe-resolve/tests/all/incremental_differential.rs` and `crates/ascribe-lsp/tests/all/differential.rs` (incremental against from scratch), `crates/ascribe-diff/tests/all/reach.rs` (`diff`'s short path against comparing every page), `crates/ascribe-check/tests/all/parity.rs` (checks against the index), `crates/ascribe-cli/tests/all/lsp_parity.rs` (the command against the server), and the renderers above |
| The command's output, as files the docs show | `crates/ascribe-cli/tests/all/output.rs`, writing `crates/ascribe-cli/tests/output/` |
| Browser tests | `packages/elements/test/`, in Chromium, Firefox, and WebKit |
| The editor | `packages/vscode/test/integration/`, in a real VS Code (CI only) |
| The site end to end | `examples/astro-site/test/e2e/`, with the packed npm packages |
| Speed | `tests/corpora/benches/perf.rs` and `crates/ascribe-resolve/benches/incremental.rs`; the weekly Corpora workflow fails a regression against `tests/corpora/baselines/perf.json` |

### `ASCRIBE_BLESS=1`

Some files are generated from a source, or copied from what the code writes, and a test fails when they're stale. Run that test with `ASCRIBE_BLESS=1` and it rewrites them instead; then read the diff. `pnpm bless` (`scripts/bless.ts`) runs every rewrite in both tables but `cargo insta review` and the corpora's, in an order where each sees what the earlier ones wrote, after building `@ascribed/cli` and the debug binary they run, then prints `git diff --stat`; `scripts/bless.test.ts` fails on a row it doesn't run or say why it skips, and on a test's command that isn't the one the test names. The tests that do this:

| Test | Rewrites | Paths |
|---|---|---|
| `tests/conformance/tests/docs.rs` | The diagnostics reference's fragments in `docs/content/_generated/`, the `ascribe.toml` reference's table of crawlers by purpose, `crawlers.md` there, from `crates/ascribe-model/data/crawlers.toml`, and its tables of keys, `content-model-*.md` there, from `crates/ascribe-model/settings.toml` | `docs/content/_generated/diagnostics-*.md`, `docs/content/_generated/crawlers.md`, `docs/content/_generated/content-model-*.md` |
| `crates/ascribe-cli/src/docs.rs` | Each command's options in `docs/content/_generated/` | `docs/content/_generated/cli-*.md`, `docs/content/_generated/directives-*.md` |
| `crates/ascribe-cli/src/shapes.rs` | The JSON Schemas in `schemas/`, the TypeScript types in `packages/astro/src/shapes.ts`, `packages/review/src/shapes.ts`, and `packages/vscode/src/shapes.ts`, and the schemas' fragments in `docs/content/_generated/`, from the Rust types that write the JSON | `schemas/`, `packages/*/src/shapes.ts`, `docs/content/_generated/json-*.md` |
| `crates/ascribe-cli/tests/all/output.rs` | The command output in `crates/ascribe-cli/tests/output/` | `crates/ascribe-cli/tests/output/*` |
| `crates/ascribe-cli/tests/all/agents.rs` | What `ascribe agents sync` writes for the examples, in `crates/ascribe-cli/tests/output/agents/` | `crates/ascribe-cli/tests/output/agents/` |
| `crates/ascribe-cli/src/agents/skill.rs` | The skill `@ascribed/cli` ships, `packages/cli/skills/ascribe/` | `packages/cli/skills/ascribe/` |
| `crates/ascribe-cli/src/agents/plugin.rs` | The Ascribe plugin, `plugins/ascribe/`, all but its README | `plugins/ascribe/`, `!plugins/ascribe/README.md` |
| `crates/ascribe-core/tests/names.rs` | The names Ascribe puts on a page, `packages/astro/src/names.ts`, `packages/elements/src/names.ts`, `packages/review/src/names.ts`, and `packages/vscode/src/names.ts`, from `crates/ascribe-core/src/names.rs` | `packages/*/src/names.ts` |
| `crates/ascribe-emit/tests/all/site_anchors.rs` | The site-render fixtures' inputs and corpus in `tests/render/` | `tests/render/*/input.md`, `tests/render/*/unanchored.md`, `tests/render/corpus/` |
| `crates/ascribe-emit/tests/all/site_sidebar.rs` | The docs site's sidebar, `site/test/fixtures/sidebar.json`, from `docs/`'s `site` build's navigation, grouped as `@ascribed/astro`'s `sidebar()` groups it | `site/test/fixtures/sidebar.json` |
| `crates/ascribe-emit/tests/all/zod.rs` | The generated schemas in `tests/zod/generated/` | `tests/zod/generated/` |
| `packages/astro/test/init.test.ts` | `examples/astro-site`'s collection and route: what `npx @ascribed/astro init` writes for it, from `packages/astro/starter/` | `examples/astro-site/src/content.config.ts`, `examples/astro-site/src/layouts/Docs.astro`, `examples/astro-site/src/pages/[...slug].astro`, `examples/astro-site/src/styles/docs.css` |
| `packages/vscode/test/unit/docs.test.ts` | The extension's settings, commands, and actions in `docs/content/_generated/` | `docs/content/_generated/editor-*.md` |
| `scripts/docs-site/elements.test.ts` | The element library reference's table of custom properties in `docs/content/_generated/`, from `packages/elements/css/style.css` | `docs/content/_generated/elements-*.md` |
| `scripts/surface/inventory.test.ts` | The map of Ascribe's surface, `project-docs/surface.md`: every command, directive, `ascribe.toml` section, diagnostic, editor command and setting, request, tool, and package entry, read from where each is declared. Its diff in a pull request is what the change added to the surface | `project-docs/surface.md` |
| `scripts/templates/convert.test.ts` | The page templates in `crates/ascribe-templates/set/`, from The Good Docs Project's release that `scripts/templates/upstream.toml` pins, by the rules in `scripts/templates/rules.toml`. It needs their repository and an `ascribe` binary, and is skipped without them; `rust.yml` runs `node scripts/templates/convert.ts --check` after its build. When they release, `templates.yml` moves the pin and writes the copies and `examples/templates` again with `scripts/templates/update.ts` | `crates/ascribe-templates/set/`, `examples/templates/` |
| `scripts/design/specimen.test.ts` | The design specimen, `design/specimen.html`, from `design/candidates/` and the stylesheets it renders | `design/specimen.html` |
| `scripts/design/tokens.test.ts` | The generated blocks of each stylesheet `design/tokens.toml` lists, and the shared colors in `design/candidates/chosen.toml`, from it. Then `pnpm --filter @ascribed/review embed` copies them into the report's stylesheet | `design/candidates/chosen.toml`, `packages/elements/css/style.css`, `packages/review/src/marks/marks.css`, `packages/review/src/overlay/style.ts`, `packages/review/src/report/report.css`, `crates/ascribe-emit/src/archive/archive.css`, `packages/astro/src/toolbar/style.ts`, `site/src/styles/site.css`, `packages/vscode/src/webview/preview.css` |

Other files are written by a command rather than by a test run with `ASCRIBE_BLESS=1`:

| Command | Writes | Paths |
|---|---|---|
| `cargo insta review` | The `insta` snapshots, in a `snapshots/` folder beside the test that takes them | `**/snapshots/*.snap` |
| `node scripts/design/assets.ts` | The mark's images (the mark and wordmark SVGs in `design/`, the favicons, the touch icon, and the social cards in `site/public/`, the extension's icons in `packages/vscode/media/`, and the README headers and avatar in `design/out/`), and `design/assets.json`, which lists them. `scripts/design/assets.test.ts` fails, naming the command, when a source has changed since ([design/README.md](design/README.md#the-assets)) | `design/assets.json`, `design/mark.svg`, `design/mark-color.svg`, `design/wordmark.svg`, `design/out/`, `site/public/favicon.*`, `site/public/apple-touch-icon.png`, `site/public/social-card.png`, `site/public/cards/`, `site/src/assets/logo.svg`, `site/home/content/images/hero-mark-*.svg`, `packages/vscode/media/icon.png`, `packages/vscode/media/activity.svg` |
| `pnpm --filter @ascribed/review embed` | The HTML report's script, its hash, and its stylesheet, which `crates/ascribe-diff` embeds, from `packages/review` and `packages/elements`. `packages/review/test/embedded.test.ts` fails while they differ | `crates/ascribe-diff/src/html/report.*` |
| `ascribe observe` | The repository's tested examples: run in the root, it runs the tests `ascribe.examples.toml` names and rewrites `examples.observations/default.toml`. `rust.yml` runs `ascribe observe --check`, which fails when one of the files it lists changed since | `examples.observations/` |
| `ASCRIBE_CORPORA_BLESS=1` | The corpora's recorded counts ([tests/corpora/README.md](tests/corpora/README.md)) | `tests/corpora/baselines/` |

In both tables, a path ending in `/` is everything under it, `*` is any part of one name and `**` any number of folders, and a path after `!` is left out. `node scripts/repo-docs/generated.ts` reads them and prints which files a branch changes are generated, and what writes each; `scripts/repo-docs/generated.test.ts` fails on a path that matches no file.

## CI

`.github/workflows/ci.yml` decides what a change needs and runs `rust.yml`, `js.yml`, and `site.yml`; the one required check, `all checks`, waits for them. `drift.yml` and `review.yml` report on pull requests that change the docs. `site-npm.yml` builds the docs with the published canary, after each canary and after a merge to `main` that touches the docs or the site. `canary.yml` publishes the nightly `next` packages, `release.yml` and `release-build.yml` make a release ([RELEASING.md](RELEASING.md)), and `corpora.yml` runs the benchmarks weekly, and `report.yml` runs `ascribe report links delivery` on the docs daily, keeping the link answers it reuses and its link history with `actions/cache`, and opening an issue when it finds something; it also asks each source of the crawlers Ascribe knows whether it answers and was read within a year (`scripts/crawlers/sources.ts`). `templates.yml` reads The Good Docs Project's tags weekly and, when they release, opens a pull request with the page templates written again (`scripts/templates/update.ts`).
