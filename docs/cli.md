# Command reference

The `ascribe` command checks, builds, and formats an Ascribe project, and runs the language server the editor uses.

```text
ascribe check [--build <NAME>]... [--format text|json] [--deny-warnings]
ascribe build [--build <NAME>]... [--emit site,plain,json] [--format text|json]
ascribe diff  [--base <REV>] [--base-exact] [--build <NAME>]... [--format text|json|html] [--exit-code]
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
- `--anchors` marks each block of the site output with the source file and lines it came from, for review: an `<!--ascribe-anchor …-->` comment before each Markdown block, and `data-ascribe-source` (with `data-ascribe-via` for a block from a fragment) on each element Ascribe writes. The Astro integration turns them into attributes on every block's element ([site-render contract](contracts/site-render.md#7-source-anchors)). The site output's manifest records `"anchors": true`. Without it, the output has no anchors. The other outputs are the same either way.

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

## `ascribe diff`

Shows what changed between a git revision and the working tree, as readers will see it: which pages of each build changed, and which blocks on them were added, removed, changed, or moved. It compares **resolved pages**, not files, so a page whose own file didn't change but whose included fragment, phrase, or build settings did is listed, with what its change comes from. A change that doesn't reach the page (`ascribe fmt`, rewrapped lines) isn't.

- `--base <REV>`: the revision to compare with, anything git accepts (a branch, a tag, a commit). By default, the repository's default branch, the first of `origin/HEAD`, `origin/main`, `origin/master`, `main`, and `master` that exists. The comparison starts from the **merge base** of that revision and `HEAD`, as a pull request shows its changes, so commits made on the base branch since you branched aren't listed. A shallow clone (what `actions/checkout` makes by default) may not have the merge base: fetch more history (`fetch-depth: 0`) or use `--base-exact`.
- `--base-exact` compares with the revision itself instead of the merge base.
- `--build <NAME>` compares only that build. Repeat it for several. By default, every build in `ascribe.toml`.
- `--format json` writes one JSON document instead of text (see [Diff JSON](#diff-json)).
- `--format html` writes one self-contained HTML file that shows every changed page rendered, with its changes marked (see [The HTML report](#the-html-report)).
- `--exit-code` exits with `1` when anything changed, as `git diff --exit-code` does.

The other side is the working tree: the files on disk, committed or not, as `ascribe build` would read them now. Unsaved editor changes aren't included. The base is read from git, `ascribe.toml` included, so a change to the content model is compared too; if the project didn't exist at the base, every page is added. `ascribe diff` needs `git` on the path, and nothing else: no network and no GitHub account.

The text output lists each build's changed pages with counts, and where a change comes from when it isn't only the page's own file:

```text
compared with main (3f9c2ab), from its merge base with HEAD (8d01e4c)
site: 4 pages changed
  getting-started.md: 1 changed, 1 added (through _fragments/prereqs.md)
  guides/rollouts.md: 5 changed, 3 added, 1 removed, 1 moved
  guides/schedules.md: added
  reference/limits.md: title changed
cloud: no changes
```

Pages with errors are compared as they are, since comparing work in progress is useful, but a broken page can render oddly and read as part of the change. So when `ascribe check` would find errors for the builds compared, `ascribe diff` says so on standard error, and the [HTML report](#the-html-report) and review in the editor's preview and the site preview show a notice:

```text
warning: the working tree has 3 errors; `ascribe check` lists them
```

### Exit codes

| Code | Meaning |
|---|---|
| `0` | Compared, whether or not anything changed, and whether or not the working tree has errors |
| `1` | With `--exit-code`: something changed |
| `2` | It couldn't run: a usage error, an unknown build, not a git repository, an unknown revision, no merge base (a shallow clone, or unrelated histories), `git` not found, or a project that doesn't load at the base or in the working tree. The reason goes to standard error. |

### The HTML report

`--format html` writes one HTML file to standard output, with everything in it: its styles and script, the element library, and every image a changed page uses, up to 1 MB each (a larger one shows as a placeholder naming its file). It makes no network requests, so it opens from a CI artifact, an email, or a disk with nothing else:

```sh
ascribe diff --format html > review.html
```

It lists the changed pages of each build, with counts, and marks a page that changed only through something it uses ("via `_fragments/prereqs.md`"). With more than one build changed, a picker switches between them. Each page is rendered as it is now, the way the editor's page preview renders it: Ascribe's bare render, without the site's layout, navigation, or styles. On it:

- an **added** block has a solid bar in the margin and the label Added;
- a **changed** block has a dashed bar and the label Changed, with the words added highlighted and the words removed struck through;
- a **removed** block is shown where it was, from the base, struck through and collapsed to one line, with **Show** to expand it;
- a **moved** block has a double bar and the label Moved, with a link to a stub at its old place, and back.

**Show: Changes / As it will be / As it was** switches between the marks, the page as it will be with none, and the page as it was at the base. The arrows step through the changes ("3 of 10 on this page"), and after the last one offer the next changed page. Hovering over a block shows the source file and line it came from (`guides/install.md:12`). The colors work in light and dark, and every mark has a label as well as a color.

A report renders at most 300 changed pages; the rest are listed by name, and the report says so at the top. Each rendered page and image is stored once, however many builds or pages share it. [The report in CI](review.md#the-report-in-ci) has a GitHub Actions job that uploads the report on every pull request.

### Diff JSON

`--format json` writes one JSON document to standard output. It follows the [JSON output](#json-output)'s rules: `schema_version` changes only when a field is removed or changes meaning, so **ignore fields you don't know**.

| Field | Type | Meaning |
|---|---|---|
| `schema_version` | number | `1` |
| `ascribe_version` | string | The version of `ascribe` that wrote the report |
| `base` | object | `requested`: the revision asked for, or the default branch used. `commit`: the commit it names. `merge_base`: the merge base with `HEAD` that was compared with, or `null` with `--base-exact`. |
| `repository` | object | `root`: the repository's top-level directory. `project_prefix`: the project's folder in it, with a trailing `/`, or `""` at the root. |
| `working_tree_errors` | number | How many errors `ascribe check` finds in the working tree for the builds compared (with the same `--build` options). The comparison runs either way. |
| `builds` | array | One entry per build compared: `build`, its name, and `pages`, the pages that changed, in path order |

Each page:

| Field | Type | Meaning |
|---|---|---|
| `path` | string | The page's path, relative to the content root |
| `route` | string | Its route; for a removed page, the route it had |
| `status` | string | `"added"` (the build publishes it now and didn't), `"removed"`, or `"changed"` |
| `own_file_changed` | boolean | Whether the page's own file changed, or exists on one side only |
| `because` | array of strings | The other changed files its change comes from, relative to the content root: fragments it includes, and pages its links take their text or a heading id from (a link to a page that changed otherwise isn't a cause). `ascribe.toml` comes last when a change to the content model (a phrase's value, a label, a build's settings) is a cause. |
| `page_changed` | array of strings | What changed about the page as a whole, besides its blocks: `"title"`, `"frontmatter"` (fields other than `title` and `available`), `"availability"` (the page-level availability, as shown), and `"route"`, in that order. Empty for an added or removed page. |
| `counts` | object | `changed`, `added`, `removed`, and `moved`: how many changes of each kind |
| `changes` | array | The block changes, in the page's order, each removed block where it was. Empty for an added or removed page, and for a page whose only changes are in `page_changed`. |

Each change:

| Field | Type | Meaning |
|---|---|---|
| `kind` | string | `"changed"`, `"added"`, `"removed"`, or `"moved"` |
| `now` | object | Where the block is written now (absent for a removed block): `source` and `via`, below |
| `was` | object | Where it was written at the base (absent for an added block) |
| `words` | object | For changed prose: `now` and `was`, the ranges of words that differ, as `[start, end]` character offsets (Unicode characters, end exclusive) into `now_text` and `was_text`, the block's text with whitespace collapsed. Absent when the text is the same (only a link's target changed, say) or the block is too long to compare word by word. |
| `after` | object | For a removed block, and for a moved block's old place: the block it came after, as it is now. Absent when it was first in its container. |
| `parent` | object | For a removed block, and for a moved block's old place: the block it was inside, as it is now. Absent at the top of the page. |
| `text` | string | For a removed block: its text, whitespace collapsed |

A block is a heading, paragraph, code block, table, list, list item, block quote, directive, container, group, or a group's arm; changes inside a list, a container, or an arm are listed on the blocks inside it, so a one-word change in a step marks that step's paragraph. A line-form directive that renders as an element around the block after it (`@note`, `@steps`, `@details`, a widget) is one block with that block inside it, from the directive's line through the block's last, like the element. A block's `source` is `<path>:<first>-<last>`: the file its text is written in, relative to the content root with each path segment percent-encoded (except ASCII letters, digits, `-`, `.`, `_`, and `~`), and its first and last lines, from 1. `via` lists the includes it came through, outermost first, each `<path>:<line>`; it's empty for a block written in the page itself. These are the same strings the site output's source anchors carry, so a tool can find a block in a rendered page.

```json
{
  "schema_version": 1,
  "ascribe_version": "0.1.1",
  "base": { "requested": "main", "commit": "3f9c2ab…", "merge_base": "8d01e4c…" },
  "repository": { "root": "/home/me/lantern", "project_prefix": "docs/" },
  "working_tree_errors": 0,
  "builds": [
    {
      "build": "site",
      "pages": [
        {
          "path": "getting-started.md",
          "route": "/getting-started/",
          "status": "changed",
          "own_file_changed": false,
          "because": ["_fragments/prereqs.md"],
          "page_changed": [],
          "counts": { "changed": 1, "added": 0, "removed": 0, "moved": 0 },
          "changes": [
            {
              "kind": "changed",
              "now": { "source": "_fragments/prereqs.md:3-3", "via": ["getting-started.md:12"] },
              "was": { "source": "_fragments/prereqs.md:3-3", "via": ["getting-started.md:12"] },
              "words": {
                "now": [[14, 17]],
                "was": [[14, 17]],
                "now_text": "Lantern agent 2.4 or later",
                "was_text": "Lantern agent 2.2 or later"
              }
            }
          ]
        }
      ]
    }
  ]
}
```

How blocks are matched: blocks are compared by their content without positions, so moving a block down the file or rewrapping it is no change. Blocks that didn't stay are paired, in order, with a block of the same kind whose words overlap by at least half, as **changed**; a block that matches nothing is **added** or **removed**, and a removed block identical to an added one elsewhere on the page is **moved**. A block over about 2,500 words gets no word ranges, and a list or container with over 1,000 blocks in it that changed is marked changed as a whole.

## `ascribe fmt`

Rewrites Ascribe constructs into canonical form: the spacing of directive lines and attribute blocks, attribute order, quoting, and blank lines between directives and their blocks. It changes nothing else, never how a page renders, and leaves alone a construct that has an error. See [Canonical form](directives.md#canonical-form).

- With no paths, it formats every `.md` file under the content root. Given files or directories, it formats the `.md` files among them. Directories whose names start with `.`, `node_modules`, and directories inside the searched ones that hold an `ascribe.toml` other than the project's own (another project, formatted under its own model) are skipped.
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
