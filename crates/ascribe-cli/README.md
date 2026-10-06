# ascribe-cli

The `ascribe` binary: `check`, `build`, `diff`, `drift`, `fmt`, and `lsp`. How to use it (every option, the outputs, the JSON report's schema, and the exit codes) is in the [command reference](https://ascribed-dev.com/reference/cli/). This README is about the code.

| Module | Role |
|---|---|
| `src/cli.rs` | The command line: options every subcommand shares, and the subcommands. Its header says what a new subcommand needs. |
| `src/commands/` | One module per subcommand, each with an `Args` type and a `run`: the arguments, one call into a library (the [command entries](../../ARCHITECTURE.md#each-commands-entry)), and the report |
| `src/context.rs` | Finding and loading the project: `--config`, or the nearest `ascribe.toml` (`ascribe_check::Project::locate`), then `Project::load` |
| `src/commands/diff.rs` | `ascribe diff`: calls `ascribe_diff::diff_project`, and writes the report as text, JSON, or HTML |
| `src/commands/drift.rs` | `ascribe drift`: calls `ascribe_diff::drift_project`, and writes its groups as text, JSON, or Markdown for a CI job's summary |
| `src/report/` | The text and JSON reports |
| `src/exit.rs` | The exit codes: 0 (no errors), 1 (errors, or warnings under `--deny-warnings`), 2 (the command couldn't run) |

`check` and `build` call `ascribe_check::diagnose`, and it and the language server call the same checking functions (`ascribe_check::check_all_builds`, and `check_project` for one build), so they report identical diagnostics. Codes, slugs, severities, and message templates come from the diagnostics registry, `tests/conformance/diagnostics.toml`.

The JSON report's `schema_version` changes only when a field is removed or changes meaning; new fields can appear without it, so consumers ignore fields they don't know.

## Tests

- `tests/check.rs`: every exit code, the text and JSON output, `--config`, and finding the model in a parent directory.
- `tests/build.rs`: the outputs, replacing a previous build, and reporting what `ascribe check` reports.
- `tests/diff.rs`: comparing with a base revision in a temporary git repository: the text, JSON, and HTML output (a snapshot of the HTML report's data, and that it loads nothing from the network), merge bases, changes through a fragment or the content model, and the failures that exit with 2.
- `tests/drift.rs`: the drift report in a temporary git repository: the groups in text, JSON, and the summary, an example that no longer resolves, nothing to report, `--exit-code`, and the failures that exit with 2, a shallow clone among them (where `check` behaves as in the full repository).
- `tests/fmt.rs`: exit statuses, `--check`, and which files it visits.
- `tests/lsp_parity.rs`: for every build of `examples/quill` and of a fixture with problems, the language server, run as a real process, publishes what `ascribe check --build` reports.
