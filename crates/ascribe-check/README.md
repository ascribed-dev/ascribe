# ascribe-check

The checks, and the project they run on. Every diagnostic Ascribe reports comes from here: `ascribe check`, `ascribe build`, and the language server all call the same functions, which is what keeps their results identical (SPEC §8).

Which diagnostics exist, with their codes, severities, and messages, is defined in one file, `tests/conformance/diagnostics.toml`. The [diagnostics reference](https://ascribed-dev.com/go/diagnostics/) is generated from it.

## Pieces

| Module | Main types and functions | What it does |
|---|---|---|
| `src/project.rs` | `Project`, `LOADER`, `load`, `in_memory`, `Project::index` | A project as the checks see it: a view of an `ascribe_workspace::WorkspaceSnapshot`, with the day the checks run on and whether personal memos count, which loads, indexes, and holds nothing of its own. `ascribe-workspace` finds and loads a project, once, as a `Workspace`; `LOADER` is what it loads one with, this crate's content model loader (`type_files::load_model`) and side files (`SIDE_FILES`), and `load` loads one with it, on the disk. `Project::new` is the view of a snapshot; `index` is the snapshot's source index, which the page-level checks and the commands that resolve builds share. `in_memory` and `in_memory_with` build a `Workspace` over texts given, as a test does. |
| `src/type_files.rs` | `type_files::load_model`, `type_files::check` | The files a content type names, `template` and `guide`: each is there, inside the project's folder and outside its content root and output directory, through `FileSystem`, and a template reads as a page. `load_model` loads `ascribe.toml` and checks them, for `LOADER` and the language server, so a problem with one is the content model's. |
| `src/builds.rs` | `select_builds`, `diagnose`, `UnknownBuild` | What `ascribe check` and `ascribe build` call: the builds named by `--build`, and their diagnostics, each problem once. `diff` and `drift` choose builds with `select_builds` too. |
| `src/lib.rs` | `check_files` | The file-level checks over every file: the content model's warnings, the lock and the copies of sources, then each file's diagnostics, in file order and source order. |
| `src/checks/` | `check_file` | The file-level checks of one file (SPEC §8.1): the parser's issues, then attributes (`attrs.rs`), availability (`avail.rs`), frontmatter (`frontmatter.rs`), references and snippets (`refs.rs`), sources in other repositories (`sources.rs`), and the advice about a page's content: its description, its recheck date, and code blocks' languages (`content.rs`). |
| `src/page/` | `check_project`, `check_all_builds`, `check_builds`, `PageChecker` | The page-level checks, on resolved pages: duplicate ids, links to ids, what a build removes, heading levels, and each page's size in the plain output (`size.rs`). `check_project` is file-level then page-level for one build; `check_all_builds` reports each distinct problem once, naming the builds it appears in. They read the project's source index, or one the caller has with the same files and ids (`PageChecker::with_index`). |
| `src/diagnostic.rs` | `Diagnostic`, `Severity`, `RelatedInfo` | The diagnostic every tool reports. |
| `src/registry.rs` | `Registry` | The diagnostics registry, embedded at build time: how each `Issue` becomes a worded, ranked `Diagnostic`. |
| `src/yaml.rs` | | Where a key or value is in a frontmatter block, so a problem is reported at its line. |

```rust,ignore
let config = ascribe_workspace::locate(None)?;
let workspace = ascribe_check::load(&config)?;
let project = ascribe_check::Project::new(workspace.snapshot());
let found = ascribe_check::diagnose(&project, &[])?;
```

## Rules

- **One entry point per job.** A tool that checks a project calls `check_project` or `check_all_builds`; it doesn't put the checks together itself. `tests/conformance`'s `page-check` adapter and `crates/ascribe-cli/tests/all/lsp_parity.rs` hold every tool to them.
- **The rules for references and snippets aren't here.** Whether a link, image, include, or snippet resolves is decided by `ascribe_resolve::references` and `ascribe_resolve::snippet`, which the source index uses too. This crate words and reports what they find.
- **Every message comes from the registry.** A check reports an `Issue` with a slug from `ascribe_core::diagnostics`; `Registry` gives it its code, severity, and text.
- **A problem is reported once, where it's caused.** Inside included content, that's the outermost include, with the place in the fragment as related information.
- **It depends on `ascribe-core`, `ascribe-syntax`, `ascribe-model`, `ascribe-resolve`, `ascribe-workspace`, and `ascribe-emit`,** and on no crate that uses it: not `ascribe-diff`, `ascribe-lsp`, or `ascribe-cli`. `page-size` measures a page's plain Markdown as `ascribe-emit` writes it, so `ascribe-emit` must never use this crate.
- **It doesn't read the clock.** `recheck-overdue` compares a date with the day the caller gives `Project::with_today`; a project without one doesn't report it.

## Tests

- `tests/all/commands.rs`: what the commands call, each with a typed result: `LOADER`, `select_builds`, and `diagnose`.
- `tests/all/checks.rs`: the file-level checks, on small projects in memory or in a temporary folder.
- `tests/all/page.rs`: the page-level checks.
- `tests/all/page_index.rs`: checking resolved pages over an index the caller already has reports what `check` reports.
- `tests/all/snippets.rs`: where `@snippet` problems are reported.
- `tests/all/file_system.rs`: the checks probe the `FileSystem` they're given, not the disk.
- `tests/all/parity.rs`: `ascribe check` and the source index report the same reference problems, at the same places, for one tree with every kind of problem.
- `tests/all/ids.rs`: this crate's `Project` and `ascribe_resolve::IncrementalProject` number files the same way.
- The conformance suite (`tests/conformance`) runs every case's expected diagnostics through these checks.
