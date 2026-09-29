# tessera-cli

The `tessera` command-line tool.

```sh
tessera --version
tessera check [--config <PATH>] [--format text|json] [--deny-warnings] [--color auto|always|never]
tessera build [--build <NAME>]... [--emit plain,json] [--format text|json]
```

| Command | What it does | Phase |
|---|---|---|
| `check` | Checks every source file for problems, without building | 10 |
| `lsp` | The language server | 15 |
| `build` | Builds outputs | 18 |
| `fmt` | Rewrites source into canonical form | 23 |

`check`, `build`, and `fmt` exist so far. Each subcommand is one module under `src/commands/`; see the note at the top of `src/cli.rs` for what a new one needs (its own module, and one variant and one match arm in `cli.rs`).

## `tessera build`

Checks the project, then writes each build's outputs to `<output-dir>/<build>/<emitter>/` (`.tessera/build/` by default), each with a manifest beside it. See `crates/tessera-emit/README.md` for the outputs and `project-docs/contracts/output-layout.md` for how the previous output is replaced.

- `--build <NAME>`: build only this build (repeat for several). By default, every build in `tessera.toml`.
- `--emit <OUTPUTS>`: `plain`, `json`, or both (`plain,json`, the default). `site` comes with phase 20; asking for it now is an error.
- `--format text|json`: how the checks' report is shown, as for `tessera check`.

The checks run first, for every build asked for, and print exactly what `tessera check` prints, once each, on standard output. If any has an error, nothing is written for any build. Progress (`built cloud/plain: 3 pages, 2 assets`) and warnings go to standard error. Without `[consumer] site` in `tessera.toml`, plain-markdown links are root-relative, and the build says so.

| Code | Meaning |
|---|---|
| `0` | Built |
| `1` | The checks found errors; nothing was written |
| `2` | The build couldn't run: a usage error, no `tessera.toml`, a content model with errors, another build writing to the output directory, or a file in the output directory that Tessera didn't write and would have to overwrite |

## Options every command accepts

- `--config <PATH>`: the content model, `tessera.toml`. A directory means the `tessera.toml` in it. By default, the nearest `tessera.toml` in the current directory or a parent, so `tessera check` works from anywhere inside a project.
- `--color <auto|always|never>`: color for text output. `auto` colors a terminal unless `NO_COLOR` is set.

## `tessera check`

Loads the content model, reads every `.md` file under its content root (skipping names that begin with `.`), and reports file-level diagnostics (SPEC §8.1): the same list the language server and the build report, from `tessera_check::check_files`. Page-level checks (ids and link targets that need an assembled page) are reported by the build.

### Exit codes

| Code | Meaning |
|---|---|
| `0` | No errors. Warnings don't fail the command unless `--deny-warnings` is given. |
| `1` | There are errors, or warnings under `--deny-warnings`. |
| `2` | The command couldn't check the project: a usage error, no `tessera.toml`, or a content model with errors (they're shown, and nothing else is checked). A source file that can't be read, or isn't UTF-8, is a `source-unreadable` error in the list, and the rest of the project is still checked. |

### Text output

Each diagnostic shows its code, message, and source, and ends with a summary line:

```text
[TSR041] Warning: this looks like the published route of `install-agent.md`; link to the file instead: `/install-agent.md`
   ╭─[ docs/keys.md:7:15 ]
   │
 7 │ See [Install](/install-agent/).
   │               ───────┬───────
   │                      ╰───────── link-route
   │
   │ Help: Link to the page's file instead of its route
───╯
checked 4 files: 0 errors, 1 warning
```

Diagnostics are written to standard output; failures that stop the command (exit code 2) are written to standard error.

### JSON output

`--format json` writes one JSON document to standard output, whatever the outcome, so a tool can always parse it. The schema is versioned: `schema_version` changes only when a field is removed or changes meaning. New fields can appear without a new version, so ignore fields you don't know.

| Field | Type | Meaning |
|---|---|---|
| `schema_version` | number | `1` |
| `tessera_version` | string | The version of `tessera` that wrote the report |
| `error` | string or null | Why the command couldn't check the project (exit code 2), or `null`. When it isn't `null`, `diagnostics` holds what was found before that: a content model's problems. |
| `files_checked` | number | How many source files were checked |
| `diagnostics` | array | Every diagnostic, in file order and, within a file, in source order |
| `summary` | object | `errors` and `warnings`: how many of each |

Each diagnostic:

| Field | Type | Meaning |
|---|---|---|
| `code` | string | The registry code, such as `TSR036` |
| `slug` | string | The registry slug, such as `link-target-missing` |
| `severity` | string | `"error"` or `"warning"` |
| `message` | string | What's wrong and what to do about it |
| `file` | string | The file, relative to the project root (the directory of `tessera.toml`), with `/` separators. `tessera.toml` for a content-model problem. |
| `range` | object | Where: `start` and `end` positions |
| `related` | array | Other places that explain it: `{file, range, message}` |
| `fixes` | array | Edits that would fix it: `{title, file, edits}`, where each edit is `{range, new_text}` and replaces the text in `range` |

A position is `{line, column, offset}`: `line` and `column` start at 1, `column` counts Unicode characters (not bytes or UTF-16 units), and `offset` is the byte offset from the start of the file. A range is `{start, end}`; `end` is just past the last character, and an edit that inserts text has equal positions. Codes, slugs, severities, and message templates come from the diagnostics registry, `tests/conformance/diagnostics.toml`.

An example, for a page that links to a route:

```json
{
  "schema_version": 1,
  "tessera_version": "0.0.0",
  "error": null,
  "files_checked": 4,
  "diagnostics": [
    {
      "code": "TSR041",
      "slug": "link-route",
      "severity": "warning",
      "message": "this looks like the published route of `install-agent.md`; link to the file instead: `/install-agent.md`",
      "file": "docs/keys.md",
      "range": {
        "start": { "line": 7, "column": 15, "offset": 55 },
        "end": { "line": 7, "column": 30, "offset": 70 }
      },
      "related": [],
      "fixes": [
        {
          "title": "Link to the page's file instead of its route",
          "file": "docs/keys.md",
          "edits": [
            {
              "range": {
                "start": { "line": 7, "column": 15, "offset": 55 },
                "end": { "line": 7, "column": 30, "offset": 70 }
              },
              "new_text": "/install-agent.md"
            }
          ]
        }
      ]
    }
  ],
  "summary": { "errors": 0, "warnings": 1 }
}
```

When the command can't run, `error` says why (the example is a missing content model; the report otherwise has the same shape):

```json
{
  "schema_version": 1,
  "tessera_version": "0.0.0",
  "error": "no tessera.toml found in /work or any parent directory; run tessera from a project, or pass --config",
  "files_checked": 0,
  "diagnostics": [],
  "summary": { "errors": 0, "warnings": 0 }
}
```

## Tests

`tests/check.rs` runs the binary and covers each exit code, the text and JSON output, `--config`, and finding the model in a parent directory.
