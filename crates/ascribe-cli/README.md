# ascribe-cli

The `ascribe` binary: `check`, `build`, `diff`, `drift`, `fmt`, `sources`, `lsp`, and the commands that answer questions (`explain`, `model`, `outline`, `link`, `refs`, `render`). How to use it (every option, the outputs, the JSON report's schema, and the exit codes) is in the [command reference](https://ascribed-dev.com/reference/cli/). This README is about the code.

| Module | Role |
|---|---|
| `src/cli.rs` | The command line: options every subcommand shares, and the subcommands. Its header says what a new subcommand needs. |
| `src/commands/` | One module per subcommand, each with an `Args` type and a `run`: the arguments, one call into a library (the [command entries](../../ARCHITECTURE.md#each-commands-entry)), and the report |
| `src/context.rs` | Finding and loading the project: `--config`, or the nearest `ascribe.toml` (`ascribe_check::Project::locate`), then `Project::load` |
| `src/commands/diff.rs` | `ascribe diff`: calls `ascribe_diff::diff_project`, and writes the report as text, JSON, or HTML |
| `src/commands/drift.rs` | `ascribe drift`: calls `ascribe_diff::drift_project`, and writes its groups as text, JSON, or Markdown for a CI job's summary |
| `src/commands/sources.rs` | `ascribe sources fetch`, `status`, and `update`: calls `ascribe_sources`, and `update` then `ascribe_diff::drift_project` for the pages whose examples changed |
| `src/answer.rs`, and `explain.rs`, `model.rs`, `outline.rs`, `link.rs`, `refs.rs`, and `render.rs` in `src/commands/` | The commands that answer questions: each calls one function in `ascribe_query` and writes its answer as text or JSON. `answer.rs` holds what they share: finding the project from a path on the command line, reading a page argument, and `--format`. |
| `src/report/` | The text and JSON reports |
| `src/docs.rs`, `src/shapes.rs` | Tests that write generated files: the command reference's fragments in `docs/content/_generated/`, and the JSON Schemas in `schemas/` with the TypeScript generated from them |
| `src/exit.rs` | The exit codes: 0 (no errors), 1 (errors, or warnings under `--deny-warnings`; for an answer, something that isn't there), 2 (the command couldn't run) |

`check` and `build` call `ascribe_check::diagnose`, and it and the language server call the same checking functions (`ascribe_check::check_all_builds`, and `check_project` for one build), so they report identical diagnostics. Codes, slugs, severities, and message templates come from the diagnostics registry, `tests/conformance/diagnostics.toml`.

The JSON report's `schema_version` changes only when a field is removed or changes meaning; new fields can appear without it, so consumers ignore fields they don't know.

## Tests

- `tests/check.rs`: every exit code, the text and JSON output, `--config`, and finding the model in a parent directory.
- `tests/build.rs`: the outputs, replacing a previous build, and reporting what `ascribe check` reports.
- `tests/diff.rs`: comparing with a base revision in a temporary git repository: the text, JSON, and HTML output (a snapshot of the HTML report's data, and that it loads nothing from the network), merge bases, changes through a fragment or the content model, and the failures that exit with 2.
- `tests/drift.rs`: the drift report in a temporary git repository: the groups in text, JSON, and the summary, an example that no longer resolves, nothing to report, `--exit-code`, and the failures that exit with 2, a shallow clone among them (where `check` behaves as in the full repository).
- `tests/fmt.rs`: exit statuses, `--check`, and which files it visits.
- `tests/sources.rs`: `ascribe sources`, and every other command over a project with sources in other repositories, in temporary repositories reached by `file://` URLs.
- `tests/answers.rs`: the commands that answer questions, on a project in a subfolder: their JSON, exit codes, `model`'s budget on `examples/quill` and on a long model, and `refs`'s cut list.
- `tests/output.rs`: the command output the docs show, in `tests/output/` (`ASCRIBE_BLESS=1` rewrites it).
- `tests/determinism.rs`: every output is the same from one run to the next, over the example projects and the docs.
- `tests/network.rs`: only `sources fetch` and `sources update` can reach another repository.
- `tests/lsp_project_log.rs`: `ascribe lsp` names the project it loaded.
- `tests/lsp_parity.rs`: for every build of `examples/quill` and of a fixture with problems, the language server, run as a real process, publishes what `ascribe check --build` reports.
