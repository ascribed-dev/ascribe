# Command reference

The `ascribe` command checks, builds, and formats an Ascribe project, and runs the language server the editor uses.

```text
ascribe check [--build <NAME>]... [--format text|json] [--deny-warnings]
ascribe build [--build <NAME>]... [--emit site,plain,json] [--format text|json]
ascribe fmt   [--check] [PATHS]...
ascribe lsp
ascribe --version
```

Install it in a project with `npm install --save-dev @ascribed/cli` (see [Getting started](getting-started.md#install-the-command)) and run it with `npx ascribe`, or from a script in `package.json`.

## Options every command accepts

- `--config <PATH>`: the content model, `ascribe.toml`. A directory means the `ascribe.toml` in it. By default, the nearest `ascribe.toml` in the current directory or a parent, so the commands work from anywhere inside a project.
- `--color <auto|always|never>`: color for text output. `auto` colors a terminal unless `NO_COLOR` is set.

## `ascribe check`

Checks every source file for problems, without building anything: every file-level diagnostic, then the page-level ones for **every build** in `ascribe.toml`. The editor and `ascribe build` run the same checks, so they report the same diagnostics.

- `--build <NAME>` checks only that build. Repeat it for several. An unknown build name is exit code 2, and the message lists the builds.
- `--format json` writes one JSON document instead of text (see [JSON output](#json-output)).
- `--deny-warnings` makes warnings fail the command too, for CI.

How diagnostics are reported:

- **Each problem is reported once**, however many builds it appears in. One that isn't in every build names the builds it's in, at the end of its message (`only in build cloud`).
- **A problem in included content is reported at the include**, with its place in the fragment as related information.
- **Content that no build publishes is checked too**, so a problem in an arm no build selects isn't missed. Its diagnostics say "in content that no build publishes". Not with `--build`, which is about one build.

Every diagnostic, with its fix, is in the [diagnostics reference](diagnostics.md).

### Exit codes

| Code | Meaning |
|---|---|
| `0` | No errors. Warnings don't fail the command unless you pass `--deny-warnings`. |
| `1` | There are errors, or warnings with `--deny-warnings`. |
| `2` | The project couldn't be checked: a usage error, no `ascribe.toml`, or a content model with errors (they're shown, and nothing else is checked). A source file that can't be read, or isn't UTF-8, is a `source-unreadable` error in the list instead, and the rest of the project is still checked. |

### Text output

Each diagnostic shows its code, message, and source, and the output ends with a summary:

```text
[ASC041] Warning: this looks like the published route of `install-agent.md`; link to the file instead: `/install-agent.md`
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

Diagnostics go to standard output. A failure that stops the command (exit code 2) goes to standard error.

### JSON output

`--format json` writes one JSON document to standard output, whatever the outcome, so a tool can always parse it. The schema is versioned: `schema_version` changes only when a field is removed or changes meaning. New fields can appear without a new version, so **ignore fields you don't know**.

| Field | Type | Meaning |
|---|---|---|
| `schema_version` | number | `1` |
| `ascribe_version` | string | The version of `ascribe` that wrote the report |
| `error` | string or null | Why the project couldn't be checked (exit code 2), or `null`. When it isn't `null`, `diagnostics` holds what was found first: the content model's problems. |
| `files_checked` | number | How many source files were checked |
| `diagnostics` | array | Every diagnostic, in file order, and in source order within a file |
| `summary` | object | `errors` and `warnings`: how many of each |

Each diagnostic:

| Field | Type | Meaning |
|---|---|---|
| `code` | string | The code, such as `ASC036` |
| `slug` | string | The diagnostic's name, such as `link-target-missing` |
| `severity` | string | `"error"` or `"warning"` |
| `message` | string | What's wrong, and what to do about it |
| `file` | string | The file, relative to the project root (the directory of `ascribe.toml`), with `/` separators. `ascribe.toml` for a content-model problem. |
| `range` | object | Where: `start` and `end` positions |
| `related` | array | Other places that explain it: `{file, range, message}` |
| `fixes` | array | Edits that would fix it: `{title, file, edits}`, where each edit is `{range, new_text}` and replaces the text in `range` |
| `builds` | array of strings | The builds a page-level diagnostic appears in, in `ascribe.toml`'s order. Empty for a file-level diagnostic, and for one in content no build publishes. With `--build`, only that build. |
| `unpublished` | boolean | `true` for a problem in content that no build publishes |

A position is `{line, column, offset}`: `line` and `column` start at 1, `column` counts Unicode characters (not bytes or UTF-16 units), and `offset` is the byte offset from the start of the file. A range's `end` is just past its last character; an edit that inserts text has equal positions.

```json
{
  "schema_version": 1,
  "ascribe_version": "0.1.0",
  "error": null,
  "files_checked": 4,
  "diagnostics": [
    {
      "code": "ASC041",
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
      ],
      "builds": [],
      "unpublished": false
    }
  ],
  "summary": { "errors": 0, "warnings": 1 }
}
```

## `ascribe build`

Checks the project, then writes each build's outputs. The checks run first, exactly as `ascribe check` runs them, and print what it prints. **If any check finds an error, nothing is written, for any build.**

- `--build <NAME>` builds only that build. Repeat it for several. By default, every build in `ascribe.toml`.
- `--emit <OUTPUTS>`: any of `site`, `plain`, and `json`, separated by commas. All three by default.
- `--format text|json`: how the checks' results are shown, as for `ascribe check`.

Progress (`built cloud/plain: 3 pages, 2 assets`) and warnings go to standard error.

### Outputs

Each build writes each output to `<output-dir>/<build>/<output>/`, `.ascribe/build/` being the default output directory, with a manifest beside it (`<output>.manifest.json`) listing every file it wrote. Every output is self-contained: images and other files that pages use are copied into it.

| Output | What it is | For |
|---|---|---|
| `site` | Markdown with web components (`<ascribe-note>`, `<ascribe-tabs>`, and the rest), heading ids and image attributes as markers the Astro integration applies, and the generated Zod schema in `_ascribe/schema.ts` | An Astro site, through `@ascribed/astro` ([Astro](astro.md)) |
| `plain` | Fully resolved CommonMark with no HTML: notes as quotes, variants as labeled sections, availability as text, and links as absolute URLs | Search indexes, LLMs, and export |
| `json` | The resolved tree of each page | Your own tools |

Without `[consumer] site` in `ascribe.toml`, plain-Markdown links are root-relative instead of absolute, and the build says so.

A rebuild replaces its previous output. It removes only files its own manifest listed, and never overwrites a file it didn't write: a file in the way is an error. Two builds can't write to one output directory at once.

### Exit codes

| Code | Meaning |
|---|---|
| `0` | Built |
| `1` | The checks found errors; nothing was written |
| `2` | The build couldn't run: a usage error, no `ascribe.toml`, a content model with errors, another build writing to the output directory, a file in the output directory that Ascribe didn't write and would have to overwrite, or two pages of the site output with the same route |

## `ascribe fmt`

Rewrites Ascribe constructs into canonical form: the spacing of directive lines and attribute blocks, attribute order, quoting, and blank lines between directives and their blocks. It changes nothing else, never how a page renders, and leaves alone a construct that has an error. See [Canonical form](directives.md#canonical-form).

- With no paths, it formats every `.md` file under the content root. Given files or directories, it formats the `.md` files among them. Directories whose names start with `.`, and `node_modules`, are skipped.
- It lists each file it changed.
- `--check` changes nothing, and lists each file that would change. Use it in CI.

| Code | Meaning |
|---|---|
| `0` | Every file was formatted, or, with `--check`, none needed it |
| `1` | With `--check`: a file would change |
| `2` | A problem: no `ascribe.toml`, a content model with errors, or a path or file that can't be read |

## `ascribe lsp`

Runs the language server, speaking the Language Server Protocol over standard input and output. An editor starts it; you don't run it yourself. It takes no options of its own, and refuses `--config` (exit code `2`): its project is the nearest `ascribe.toml` at or above the workspace folder the editor gives it, never one below, and `ascribe.toml`'s `[editor] build` says which build's page-level diagnostics to report. Its logs go to standard error, starting with the project it uses.

The VS Code extension runs it for you, one server for each project in the workspace. See [Editing](editor.md). Any editor with an LSP client can run `ascribe lsp` too; for several projects, start one per project ([Other editors](editor.md#other-editors)).
