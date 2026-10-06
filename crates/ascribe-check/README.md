# ascribe-check

The checks, and loading a project from disk to run them on. Every diagnostic Ascribe reports comes from here: `ascribe check`, `ascribe build`, and the language server all call the same functions, which is what keeps their results identical (SPEC §8).

Which diagnostics exist, with their codes, severities, and messages, is defined in one file, `tests/conformance/diagnostics.toml`. The [diagnostics reference](https://ascribed-dev.com/reference/diagnostics/) is generated from it.

## Pieces

| Module | Main types and functions | What it does |
|---|---|---|
| `src/project.rs` | `Project`, `SourceFile`, `LoadError`, `Project::locate`, `Project::load`, `Project::load_model`, `Project::index` | A project as the checks see it: the content model and every source file's text, read through `ascribe_resolve::DiskFs`. `Project::locate` finds the content model a command works on, and `Project::load` is how the commands load a project (`fmt` takes only `load_model`); `index` is the source index over the same files, for the commands that resolve builds; `from_parts_with_fs` builds one over another `FileSystem`, as the language server does. |
| `src/builds.rs` | `select_builds`, `diagnose`, `UnknownBuild` | What `ascribe check` and `ascribe build` call: the builds named by `--build`, and their diagnostics, each problem once. `diff` and `drift` choose builds with `select_builds` too. |
| `src/lib.rs` | `check_files` | The file-level checks over every file: the content model's warnings, the lock and the copies of sources, then each file's diagnostics, in file order and source order. |
| `src/checks/` | `check_file` | The file-level checks of one file (SPEC §8.1): the parser's issues, then attributes (`attrs.rs`), availability (`avail.rs`), frontmatter (`frontmatter.rs`), references and snippets (`refs.rs`), and sources in other repositories (`sources.rs`). |
| `src/page/` | `check_project`, `check_all_builds`, `check_builds`, `PageChecker` | The page-level checks, on resolved pages: duplicate ids, links to ids, and what a build removes. `check_project` is file-level then page-level for one build; `check_all_builds` reports each distinct problem once, naming the builds it appears in. `src/page/bridge.rs` builds the source index over a checked project. |
| `src/diagnostic.rs` | `Diagnostic`, `Severity`, `RelatedInfo` | The diagnostic every tool reports. |
| `src/registry.rs` | `Registry` | The diagnostics registry, embedded at build time: how each `Issue` becomes a worded, ranked `Diagnostic`. |
| `src/yaml.rs` | | Where a key or value is in a frontmatter block, so a problem is reported at its line. |

```rust,ignore
let config = ascribe_check::Project::locate(None)?;
let project = ascribe_check::Project::load(&config)?;
let found = ascribe_check::diagnose(&project, &[])?;
```

## Rules

- **One entry point per job.** A tool that checks a project calls `check_project` or `check_all_builds`; it doesn't put the checks together itself. `tests/conformance`'s `page-check` adapter and `crates/ascribe-cli/tests/lsp_parity.rs` hold every tool to them.
- **The rules for references and snippets aren't here.** Whether a link, image, include, or snippet resolves is decided by `ascribe_resolve::references` and `ascribe_resolve::snippet`, which the source index uses too. This crate words and reports what they find.
- **Every message comes from the registry.** A check reports an `Issue` with a slug from `ascribe_core::diagnostics`; `Registry` gives it its code, severity, and text.
- **A problem is reported once, where it's caused.** Inside included content, that's the outermost include, with the place in the fragment as related information.
- **It depends on `ascribe-core`, `ascribe-syntax`, `ascribe-model`, and `ascribe-resolve`,** and on no crate that uses it: not `ascribe-emit`, `ascribe-diff`, `ascribe-lsp`, or `ascribe-cli`.

## Tests

- `tests/commands.rs`: what the commands call, each with a typed result: `locate`, `load_model`, `select_builds`, and `diagnose`.
- `tests/checks.rs`: the file-level checks, on small projects in memory or in a temporary folder.
- `tests/page.rs`: the page-level checks.
- `tests/page_index.rs`: checking resolved pages over an index the caller already has reports what `check` reports.
- `tests/snippets.rs`: where `@snippet` problems are reported.
- `tests/file_system.rs`: the checks probe the `FileSystem` they're given, not the disk.
- `tests/parity.rs`: `ascribe check` and the source index report the same reference problems, at the same places, for one tree with every kind of problem.
- `tests/ids.rs`: this crate's `Project` and `ascribe_resolve::IncrementalProject` number files the same way.
- The conformance suite (`tests/conformance`) runs every case's expected diagnostics through these checks.
