---
type: reference
title: "`ascribe check`, `build`, and `fmt`"
description: "Check a project's sources, build its outputs, and format its Markdown: options, output, and exit codes."
---

The commands that check, build, and format a project. The options every command accepts are in the [command reference](../command/index.md#options-every-command-accepts).

## `ascribe check`
@id {permalink=true}: cli-check

Checks every source file for problems, without building anything: every file-level diagnostic, then the page-level ones for **every build** in `ascribe.toml`. The editor and `ascribe build` run the same checks, so they report the same diagnostics.

@include: ../_generated/cli-check-options.md

How diagnostics are reported:

- **Each problem is reported once**, however many builds it appears in. One that isn't in every build names the builds it's in, at the end of its message (`only in build cloud`).
- **A problem in included content is reported at the include**, with its place in the fragment as related information.
- **Content that no build publishes is checked too**, so a problem in an arm no build selects isn't missed. Its diagnostics say "in content that no build publishes". Not with `--build`, which is about one build.

@available: ascribe 0.3.0
Two checks depend on the day they run: `review-overdue` reports a page whose review date has passed, and `release-line-ended` a live release line whose `ends` date has passed, each as of today's date in UTC. Set `ASCRIBE_TODAY=YYYY-MM-DD` to check as of another day, such as in a test that mustn't change with the calendar. Every command that checks, the editor and the hook's check server included, reads it.

Every diagnostic, with its fix, is in the [diagnostics reference](diagnostics.md).

### Checking some files

@available: ascribe 0.3.0
`ascribe check docs/guides/install.md` reports on that file alone. Paths are relative to the current directory, and each is a file or a directory. Without `--config`, the project is the nearest `ascribe.toml` at or above the first path, so the command works from a repository's root as well as from inside the project.

@available: ascribe 0.3.0
The whole project is still checked, since links, includes, and ids need it, and only the problems that count for the paths are shown. **A problem counts for a path when its file, or the file of one of its related places, is in that path.** So checking a page shows the problems its fragments cause on it, and checking a fragment shows the problems in it. A fragment's problem is reported at every page that includes it; checking the fragment shows it once, at the first include, and its `repeats` says at how many other includes it appears too. The problems are the whole check's, filtered this way, in the same order.

@available: ascribe 0.3.0
A path that doesn't exist, isn't in an Ascribe project, or is in another project than the first path's (a project nested in the content root, say) is exit code `2`, with a message naming it.

### Checking text before it's saved

@available: ascribe 0.3.0
`--stdin --path docs/guides/new.md` checks standard input as that file's text, laid over the project on disk. Nothing is written, and the file doesn't have to exist, but it has to be where a source file could be: a `.md` file in the content root. Only the problems that count for that file are shown, so a link from another page that the new text breaks isn't. `--stdin` without `--path`, or with other paths, is a usage error.

```sh
npx ascribe check --stdin --path docs/guides/new.md < draft.md
```

### Checking the prose

@available: ascribe 0.3.0
With `--vale`, or with `in-check = true` in [`[checks.vale]`](../content-model/checks.md#checksvale), `ascribe check` also runs [Vale](https://vale.sh) on the prose of the files it reports on, once for all of them, and reports each alert as a `prose` diagnostic at its place in the source: Vale's `suggestion`, `warning`, and `error` are advice, a warning, and an error, up to `[checks.vale] max-level`. Vale is given the prose alone, with phrases replaced by their text, and never frontmatter, directive lines, attributes, or code. A Vale that isn't installed, fails, or takes more than ten minutes is one `prose-not-checked` advice on `ascribe.toml`, and the rest of the check is the same. `--vale` without `[checks.vale]` is exit code `2`. See [Prose, through Vale](vale.md).

### A quick check after each edit

@available: ascribe 0.3.0
`--editor-build` runs what the editor runs: the file-level checks and the page-level checks of the editor's build only (`[editor] build`, or the first build), without the pass over content no build publishes, and without the image checks (`image-unused` and `image-large`), which the editor doesn't report. With paths that name files, only those files and the pages that include them are checked, and none of the [content checks](diagnostics/content-checks.md) across the project, which need every page; that's quick enough to run after every edit; the timings are in [`tests/corpora/RESULTS.md`]({repo}/blob/main/tests/corpora/RESULTS.md#one-file). It can't be combined with `--build`. The summary line, and `builds_checked` in the JSON, name the build, so a clean result says what it covers. A full `ascribe check` before you finish still covers every build.

### Output for agents

@available: ascribe 0.3.0
`--format concise` writes one line per diagnostic, `file:line: [code] message`, grouped by file in file order and by line within a file, then the summary line. It shows at most 50 diagnostics, then `and N more:` with the command that narrows the check to the next file:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/check-concise.txt

@available: ascribe 0.3.0
`--format prompt` writes a prompt for an agent that fixes the problems: about the file, when the paths name one file, and otherwise about the paths or the whole project. It counts them as the summary does, and asks the agent to fix the errors and warnings (`Fix the 1 error`), with advice after them as something to look at, since it never fails the check. It lists at most 20 problems, or 20 files, then names the command that lists the rest, and ends with how to check the result. It writes nothing when there are no problems, and the exit codes are the same. With `--stdin`, it says the file has unsaved changes. See [Prompt your agent](../agents/index.md#prompt-your-agent).

@available: ascribe 0.3.0
`--summary` replaces the list with how many problems each code and each file has, most first, in any format. On a project with hundreds of warnings, it shows which rule or file to work through first. The text lists at most 20 files, then the command whose JSON lists them all, and ends, before the summary line, with the command that lists the first code's problems: `To work through the first code: ascribe check --code ASC037`.

### Checking one rule at a time

@available: ascribe 0.3.0
`--code ASC037` shows only the problems with that code, or with that name (`--code link-id-missing`), as [`ascribe explain`](../command/questions.md#cli-explain) takes it. Repeat it for several. The whole project is still checked, and the rest is left out afterwards, as paths leave out what doesn't count for them, so an agent, or a person, can fix one rule across the project instead of working file by file. Everything after is about what's left: the counts, and the summary line, which names the codes (``reported on 12 for code `ASC037` ``); the exit code, `1` only when a problem shown is an error; `--format prompt`, about the problems left; and `--fix`, which applies only their fixes. The commands a report names keep the codes: the concise list's `and N more:`, and the JSON's `next_command`. A code no diagnostic has is exit code `2`, with the closest named, as on `explain`.

```sh
npx ascribe check --code link-id-missing --format concise
```

### Applying the safe fixes

@available: ascribe 0.3.0
`--fix` applies every fix marked `safe`, then reports what's left, as `ascribe fmt` rewrites what it can. A fix is safe only when applying it can't change what the page says and leaves nothing to decide, such as linking to a page's file instead of its route, or removing an acknowledgement that covers nothing. An `unsafe` fix, such as the nearest spelling of a misspelled name, is never applied: it stays in the report for you to decide.

```sh
npx ascribe check --fix
```

@available: ascribe 0.3.0
Only the fixes of the diagnostics reported are applied, so `ascribe check docs/guides/install.md --fix` changes nothing outside what counts for that file. The output starts with a line for each file changed (`fixed 2 problems in docs/guides/install.md`), and the rest of the report, and the exit code, are those of the check after the fixes. Two fixes that would edit the same text are one too many: the first is applied, and the other's problem is reported again, for the next run. `--fix` can't be combined with `--stdin`, whose text isn't on disk.

@available: next
In VS Code, **Source Action… → Apply the safe fixes in this page** applies a page's safe fixes the same way, to the text in the editor, saved or not, for the problems the editor shows, and can run each time you save. See [Apply the safe fixes](../editor/index.md#apply-the-safe-fixes).

### Exit codes

| Code | Meaning |
|---|---|
| `0` | No errors. Warnings don't fail the command unless you pass `--deny-warnings`, and advice never does. |
| `1` | There are errors, or warnings with `--deny-warnings`. |
| `2` | The project couldn't be checked: a usage error, a `--code` no diagnostic has, no `ascribe.toml`, a content model with errors (they're shown, and nothing else is checked), a path that doesn't exist or isn't in the project, or, with `--fix`, a file it fixed that can't be written. A source file that can't be read, or isn't UTF-8, is a `source-unreadable` error in the list instead, and the rest of the project is still checked. |

### Text output

Each diagnostic shows its code, message, and source, and its next step as `Help:`: the title of its first fix, or, with none, the fix advice the JSON gives as `help`. The output ends with a summary in one form: the files checked, the builds (`for every build`, or with `--build`, `` for build `cloud` ``), the files reported on, which with no paths are all of them, and the counts of errors, warnings, and advice (`checked 12 files for every build, reported on 1: 1 error, 0 warnings, 0 advice`):

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/check.txt

Diagnostics go to standard output. A failure that stops the command (exit code 2) goes to standard error.

@available: ascribe 0.3.0
Advice, a diagnostic below a warning, is listed after the errors and warnings, in every format, and the summary counts it apart: `0 errors, 2 warnings, 5 advice`. It never fails the command, even with `--deny-warnings`. A project sets how loudly a check about its content's quality speaks in [`[checks]`](../content-model/checks.md#19-checks).

A problem [acknowledged as intended](../writing/directives.md#intended) isn't listed, and doesn't fail the command; the summary counts it when there is one: `0 errors, 2 warnings, 0 advice, 1 acknowledged`. `ascribe build` counts it the same way.

### JSON output

`--format json` writes one JSON document to standard output, whatever the outcome, so a tool can always parse it. The schema is versioned: `schema_version` changes only when a field is removed or changes meaning. New fields can appear without a new version, so **ignore fields you don't know**. The [JSON report contract](../contracts/json-reports.md) has each command's schema.

| Field | Type | Meaning |
|---|---|---|
| `schema_version` | number | `1` |
| `ascribe_version` | string | The version of `ascribe` that wrote the report |
| `error` | string or null | Why the project couldn't be checked (exit code 2), or `null`. When it isn't `null`, `diagnostics` holds what was found first: the content model's problems. |
| `files_checked` | number | How many source files were checked: the project's |
| `files_reported` | number | How many of them the report covers: the source files in the paths named, or every one |
| `builds_checked` | array of strings | The builds whose page-level checks ran: every build, those named with `--build`, or the editor's with `--editor-build` |
| `diagnostics` | array | Every diagnostic, in file order, and in source order within a file; advice after the errors and warnings, in the same order. With paths, those that count for them. With `--summary`, none. |
| `truncated` | boolean | Whether `diagnostics` leaves some out, as it does with `--summary` |
| `shown` | number | How many diagnostics `diagnostics` lists |
| `total` | number | How many there are |
| `next_command` | string or null | When `truncated`, the command that lists the rest |
| `acknowledged` | array | The problems [acknowledged as intended](../writing/directives.md#intended), which `diagnostics` leaves out, in file order: `{code, slug, message, file, range, builds, reason, at}`, where `reason` is the acknowledgement's and `at` (`{file, range}`) is where it's written. With paths, those in the files named. Left out when there are none, and with `--summary`. |
| `fixed` | array | With `--fix`, the safe fixes it applied, in file order: `{code, slug, title, file}`, where `code` and `slug` are the fixed diagnostic's and `title` is the fix's. The rest of the report is the check after them. Left out when none was applied. |
| `summary` | object | `errors`, `warnings`, and `advice`: how many of each, and `acknowledged`, how many problems are acknowledged, when there are some. With `--summary`, also `by_code` (`{code, slug, severity, count}`) and `by_file` (`{file, errors, warnings, advice}`), most first. |

Each diagnostic:

| Field | Type | Meaning |
|---|---|---|
| `code` | string | The code, such as `ASC036` |
| `slug` | string | The diagnostic's name, such as `link-target-missing` |
| `severity` | string | `"error"`, `"warning"`, or `"advice"`. More severities may be added: treat one you don't know as advice. |
| `next` | string | The kind of next step: `"fix"` when Ascribe can make the edit, `"choose"` when you pick among things Ascribe can list, `"write"` when it needs writing or judgment, `"outside"` when nothing in the source can fix it, `"review"` when it may be fine as it is. See [What to do next](diagnostics.md#what-to-do-next). |
| `rule` | string | Only on a `prose` diagnostic: the Vale rule that found it, such as `Ascribe.Repeated`. Absent otherwise. |
| `message` | string | What's wrong, and what to do about it |
| `file` | string | The file, relative to the project root (the directory of `ascribe.toml`), with `/` separators. `ascribe.toml` for a content-model problem. |
| `range` | object | Where: `start` and `end` positions |
| `related` | array | Other places that explain it: `{file, range, message}` |
| `fixes` | array | Edits that would fix it: `{title, file, edits, applicability}`, where each edit is `{range, new_text}` and replaces the text in `range`. `applicability` is `"safe"` when applying the edits can't change what the page says and leaves nothing to decide, such as linking to a page's file instead of its route, and `"unsafe"` otherwise, such as the nearest spelling of a misspelled name. |
| `builds` | array of strings | The builds a page-level diagnostic appears in, in `ascribe.toml`'s order. Empty for a file-level diagnostic, and for one in content no build publishes. With `--build`, only that build. |
| `unpublished` | boolean | `true` for a problem in content that no build publishes |
| `help` | string | How to fix it, in general: the advice in the [diagnostics reference](diagnostics.md) |
| `docs` | string | The address of its entry in the diagnostics reference: the docs' permalink for its code, such as `https://ascribed-dev.com/go/ASC036/` |
| `repeats` | number | For a problem in a fragment, when checking the fragment: at how many other includes it's reported too. `0` otherwise. |

A position is `{line, column, offset}`: `line` and `column` start at 1, `column` counts Unicode characters (not bytes or UTF-16 units), and `offset` is the byte offset from the start of the file. A range's `end` is just past its last character; an edit that inserts text has equal positions.

@snippet {phrases=true}: code:crates/ascribe-cli/tests/output/check.json

## `ascribe build`
@id {permalink=true}: cli-build

Checks the project, then writes each build's outputs. The checks run first, exactly as `ascribe check` runs them, and print what it prints. **If any check finds an error, nothing is written, for any build.**

@include: ../_generated/cli-build-options.md

@available: ascribe 0.2.0
The Astro integration turns the source anchors that `--anchors` writes into attributes on every block's element ([site-render contract](../contracts/site-render.md#7-source-anchors)).

Progress (`built cloud/plain: 3 pages, 2 assets`) and warnings go to standard error.

### Outputs

Each build writes each output to `<output-dir>/<build>/<output>/`, `.ascribe/build/` being the default output directory, with a manifest beside it (`<output>.manifest.json`) listing every file it wrote. Every output is self-contained: images and other files that pages use are copied into it.

| Output | What it is | For |
|---|---|---|
| `site` | Markdown with web components (`<ascribe-note>`, `<ascribe-tabs>`, and the rest), heading ids and image attributes as markers the Astro integration applies, and the generated Zod schema in `_ascribe/schema.ts` | An Astro site, through `@ascribed/astro` ([Astro](../astro/index.md)) |
| `plain` | Fully resolved CommonMark with no HTML: notes as quotes, [variants](../writing/directives.md#variant) as labeled sections, availability as text, and links as absolute URLs | Search indexes, LLMs, and export |
| `json` | The resolved tree of each page | Your own tools |

Without `[consumer] site` in `ascribe.toml`, plain-Markdown links are root-relative instead of absolute, and the build says so.

A rebuild replaces its previous output. It removes only files its own manifest listed, and never overwrites a file it didn't write: a file in the way is an error. Two builds can't write to one output directory at once.

### Exit codes
@id: build-exit-codes

| Code | Meaning |
|---|---|
| `0` | Built |
| `1` | The checks found errors; nothing was written |
| `2` | The build couldn't run: a usage error, no `ascribe.toml`, a content model with errors, another build writing to the output directory, a file in the output directory that Ascribe didn't write and would have to overwrite, or two pages of the site output with the same route |

## `ascribe fmt`
@id {permalink=true}: cli-fmt

Rewrites Ascribe constructs into canonical form: the spacing of directive lines and attribute blocks, attribute order, quoting, and blank lines between directives and their blocks. It changes nothing else, never how a page renders, and leaves alone a construct that has an error. See [Canonical form](../writing/directives.md#canonical-form).

It lists each file it changed.

@include: ../_generated/cli-fmt-options.md

With `--format json`, it writes one JSON document instead: each file that changed, relative to the project's folder, with the edits that format it, each a `range` and its `new_text`, the shape of a fix's edits in [`ascribe check`'s JSON](check.md#json-output). The ranges are in the file as it was. With `--check` too, it writes no file, and the edits are the ones it would make: what a tool that applies edits itself needs. `written` says whether the files were rewritten, and `refused` lists the files left alone, each with its `reason`. The [schema](../contracts/json-reports.md#ascribe-fmt) has every field.

It formats only what `ascribe check` reads: a file in the content root that's a symbolic link, or is in a linked folder, is formatted only when the link leads to a source file of the content root. One that leads anywhere else is left alone and reported on standard error, as `check` reports it ([`source-unreadable`](diagnostics/pages.md#ASC123)), and the other files are still formatted.

| Code | Meaning |
|---|---|
| `0` | Every file was formatted, or, with `--check`, none needed it |
| `1` | With `--check`: a file would change |
| `2` | A problem: no `ascribe.toml`, a content model with errors, a path or file that can't be read, or a symbolic link that leads out of the content root |
