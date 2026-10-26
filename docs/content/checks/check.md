---
type: reference
title: "`ascribe check`, `build`, and `fmt`"
description: "Check a project's sources, build its outputs, and format its Markdown: options, output, and exit codes."
---

The commands that check, build, and format a project. The options every command accepts are in the [command reference](../command/index.md#options-every-command-accepts).

## `ascribe check`
@id {permalink=true}: cli-check

@summary
Checks every source file for problems, without building anything: every file-level diagnostic, then the page-level ones for **every build** in `ascribe.toml`. The editor and `ascribe build` run the same checks, so they report the same diagnostics.

@include: ../_generated/cli-check-options.md

How diagnostics are reported:

- **Each problem is reported once**, however many builds it appears in. One that isn't in every build names the builds it's in, at the end of its message (`only in build cloud`).
- **A problem in included content is reported at the include**, with its place in the fragment as related information.
- **Content that no build publishes is checked too**, so a problem in an arm no build selects isn't missed. Its diagnostics say "in content that no build publishes". Not with `--build`, which is about one build.

Every diagnostic, with its fix, is in the [diagnostics reference](diagnostics.md).

### Checking as of another day

Two checks depend on the day they run: `review-overdue` reports a page whose review date has passed, and `release-line-ended` a live release line whose `ends` date has passed, each as of today's date in UTC. Set `ASCRIBE_TODAY=YYYY-MM-DD` to check as of another day, such as in a test that mustn't change with the calendar. Every command that checks, the editor and the hook's check server included, reads it.

### Checking some files

`ascribe check docs/guides/install.md` reports on that file alone, from a repository's root as well as from inside the project. A fragment's problem is reported at every page that includes it, so checking the fragment shows it once, and its `repeats` says at how many other includes it appears too. The problems shown are the whole check's, in the same order.

### Checking text before it's saved

`--stdin` checks text that isn't on disk yet, such as an editor's unsaved buffer or an agent's draft. The file `--path` names has to be where a source file could be: a `.md` file in the content root. A link from another page that the new text breaks isn't shown, since it doesn't count for that file.

```sh
npx ascribe check --stdin --path docs/guides/new.md < draft.md
```

### Checking the prose

`--vale` reports Vale's `suggestion`, `warning`, and `error` alerts as advice, warnings, and errors, up to [`[checks.vale] max-level`](../content-model/checks.md#checksvale). Vale is given the prose alone, and a Vale that takes more than ten minutes is given up, as `prose-not-checked`. See [Prose, through Vale](vale.md).

### A quick check after each edit

`--editor-build` leaves out what the editor doesn't report: the pass over content no build publishes, the image checks (`image-unused` and `image-large`), and, with paths that name files, the [content checks](diagnostics/content-checks.md) across the project. It's quick enough to run after every edit; the timings are in [`tests/corpora/RESULTS.md`]({repo}/blob/main/tests/corpora/RESULTS.md#one-file). It can't be combined with `--build`. The summary line, and `builds_checked` in the JSON, name the build, so a clean result says what it covers.

### Output for agents

`--format concise` writes a line for each problem, grouped by file in file order, and by line within a file, and ends with the summary line. A problem with a fix ends its line with `→` and what the fix does, the help the full output gives:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/check-concise.txt

It lists at most 50 problems. When there are more, the line before the summary says how many it left out, and the command that lists them all: the same check with `--limit all` (`and 11 more: ascribe check docs --code ASC212 --format concise --limit all`). `--limit <n>` sets how many it lists, and `--limit all` lists every one. `--limit` is only for `--format concise`; with another format, or with `--summary`, it's a usage error.

`--format prompt` writes the prompt [Prompt your agent](../agents/prompts.md) describes. It lists at most 20 problems, or 20 files, then names the command that lists the rest. With `--stdin`, it says the file has unsaved changes.

`--summary` works in any format. The text lists at most 20 files, then the command whose JSON lists them all. On a project with hundreds of warnings, it shows which rule or file to work through first.

### Checking one rule at a time

`--code` lets an agent, or a person, fix one rule across the project instead of working file by file. The summary line names the codes (``reported on 12 for code `ASC037` ``), and the commands a report names keep them: the concise list's `and N more:`, and the JSON's `next_command`.

```sh
npx ascribe check --code link-id-missing --format concise
```

### Applying the safe fixes

An `unsafe` fix, such as the nearest spelling of a misspelled name, is never applied by `--fix`: it stays in the report for you to decide. `ascribe fmt` is the other command that rewrites what it can.

```sh
npx ascribe check --fix
```

The output starts with a line for each file changed (`fixed 2 problems in docs/guides/install.md`). Two fixes that would edit the same text are one too many: the first is applied, and the other's problem is reported again, for the next run. `--fix` can't be combined with `--stdin`, whose text isn't on disk.

In VS Code, **Source Action… → Apply the safe fixes in this page** applies a page's safe fixes the same way, to the text in the editor, saved or not, for the problems the editor shows, and can run each time you save. See [Apply the safe fixes](../editor/problems.md#apply-the-safe-fixes).

### Exit codes

| Code | Meaning |
|---|---|
| `0` | No errors. Warnings don't fail the command unless you pass `--deny-warnings`, and advice never does. |
| `1` | There are errors, or warnings with `--deny-warnings`. |
| `2` | The project couldn't be checked: a usage error, a `--code` no diagnostic has, no `ascribe.toml`, a content model with errors (they're shown, and nothing else is checked), a path that doesn't exist or isn't in the project, an `ascribe.baseline` with a line that doesn't parse (the error names it; `--no-baseline` checks without the file), or, with `--fix`, a file it fixed that can't be written. A source file that can't be read, or isn't UTF-8, is a `source-unreadable` error in the list instead, and the rest of the project is still checked. |

### Text output

Each diagnostic shows its code, message, and source, and its next step as `Help:`: the title of its first fix, or, with none, the fix advice the JSON gives as `help`. Help that's the same for every diagnostic with a code is printed in full once; later diagnostics with that code point back to it, as `Help: as for ASC252 above (ascribe explain ASC252)`. The output ends with a summary in one form: the files checked, the builds (`for every build`, or with `--build`, `` for build `cloud` ``), the files reported on, which with no paths are all of them, and the counts of errors, warnings, and advice (`checked 12 files for every build, reported on 1: 1 error, 0 warnings, 0 advice`):

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/check.txt

A project with no pages yet, and nothing to report, says so in place of the summary, with the content root: `No pages in docs/ yet.` It exits with 0. The JSON is the same as for any check, with `files_checked` 0.

Diagnostics go to standard output. A failure that stops the command (exit code 2) goes to standard error.

Advice, a diagnostic below a warning, is listed after the errors and warnings, in every format, and the summary counts it apart: `0 errors, 2 warnings, 5 advice`. It never fails the command, even with `--deny-warnings`. A project sets how loudly a check about its content's quality speaks in [`[checks]`](../content-model/checks.md#19-checks).

A problem [acknowledged as intended](../writing/directives.md#intended) isn't listed, and doesn't fail the command; the summary counts it when there is one: `0 errors, 2 warnings, 0 advice, 1 acknowledged`. `ascribe build` counts it the same way.

Advice the project's [baseline](baseline.md), `ascribe.baseline`, holds back isn't listed either, and the summary counts it: `0 errors, 0 warnings, 2 advice, 3412 in the baseline`, and the JSON's `summary` as `baselined`. An entry of the baseline that matches nothing is reported, as `baseline-entry-unmatched`, when the check covers the whole project and every build. `--no-baseline` lists everything, without reading the file.

### Quieting a problem

There are four ways to quiet a problem, each with its own rules:

| Way | What it quiets | For how long | Reason | Read by |
|---|---|---|---|---|
| [`@intended`](../writing/directives.md#intended), the `intended` frontmatter key, or [`[[intended]]`](../content-model/checks.md#intended-acknowledgements) | One problem of a review check, at its place | For good | Required | Everything |
| [`[checks]`](../content-model/checks.md#19-checks) `<check> = "off"` | A content check, in the whole project | Until it's turned back on | None | Everything |
| `[checks]` `<check> = "advice"` or `"warning"` | Lowers a content check, in the whole project | Until it's set again | None | Everything |
| [`ascribe.baseline`](baseline.md) | Many advice, each by its problem | Until it's fixed and pruned | None | `ascribe check`, the editor, the hooks, and the MCP server; not `ascribe build` or `ascribe report`, for now |

- **An error about whether the project is valid is never quieted,** by any of the four: a broken link, an unclosed container, or an unknown attribute makes a page wrong, and is fixed instead. These are the specification's diagnostics ([SPEC §8.2]({repo}/blob/main/SPEC.md#82-diagnostics)), and none is configurable. The four are for the [content checks](diagnostics.md#setting-a-checks-level), Ascribe's own checks of the content's quality.
- **A content check raised to an error isn't the same.** `[checks]` can raise a content check, such as `page-orphan = "error"`: its problems then fail `ascribe check` and stop `ascribe build`, as an error does, but the page is still valid by the specification. A review check raised this way can still be acknowledged with `@intended`, at whatever level the project gives it.
- **The baseline holds advice only.** A check raised to a warning or an error is reported, whatever the baseline lists. To put a check's warnings off, lower it to `"advice"`, then run `ascribe baseline write` again.

Which to use: acknowledge a problem the team has looked at and means to keep, with why, beside it. Turn off or lower a check the project doesn't need, which a reviewer sees in `ascribe.toml`. Put a backlog of advice in the baseline, to fix at the team's own pace, so `ascribe check` reports only what's new.

### JSON output

`--format json` writes one JSON document to standard output, whatever the outcome, so a tool can always parse it. Its [schema](../contracts/json-reports.md#ascribe-check-and-ascribe-build) has every field, under the rules the [JSON report contract](../contracts/json-reports.md) sets for every document, such as ignoring fields you don't know:

@snippet {phrases=true}: code:crates/ascribe-cli/tests/output/check.json

## `ascribe build`
@id {permalink=true}: cli-build

@summary
Checks the project, then writes each build's outputs. The checks run first, exactly as `ascribe check` runs them. **If any check finds an error, nothing is written, for any build.**

@include: ../_generated/cli-build-options.md

The Astro integration turns the source anchors that `--anchors` writes into attributes on every block's element ([site-render contract](../contracts/site-render.md#7-source-anchors)).

The build prints each error in full, since errors stop it. Warnings and advice are only counted, in the check's summary line, with the command that lists each: `checked 8 files in every build: 0 errors, 1 warning, 3 advice: ascribe check for each`, or `checked 8 files in every build: no problems`. The advice the [baseline](baseline.md) holds back isn't counted with the rest, but after it, as `ascribe check` counts it: `no problems, 3412 in the baseline`. `--format json` prints exactly what `ascribe check --format json` does.

Then it prints one line for each build it wrote, with the outputs it wrote, on standard error, with its warnings:

```text
built site: 6 pages, 2 assets (site, plain, json)
built cloud: 3 pages, 2 assets (site, plain, json)
```

### Outputs

Each build writes each output to `<output-dir>/<build>/<output>/`, `.ascribe/build/` being the default output directory, with a manifest beside it (`<output>.manifest.json`) listing every file it wrote. Every output is self-contained: images and other files that pages use are copied into it.

| Output | What it is | For |
|---|---|---|
| `site` | Markdown with web components (`<ascribe-note>`, `<ascribe-tabs>`, and the rest), heading ids and image attributes as markers the Astro integration applies, and the generated Zod schema in `_ascribe/schema.ts` | An Astro site, through `@ascribed/astro` ([Astro](../astro/index.md)) |
| `plain` | Fully resolved CommonMark with no HTML: notes as quotes, [variants](../glossary.md#variant) as labeled sections, availability as text, and links as absolute URLs | Search indexes, LLMs, and export |
| `json` | The resolved tree of each page | Your own tools |

Without `[consumer] site` in `ascribe.toml`, plain-Markdown links are root-relative instead of absolute, and the build says so.

A rebuild replaces its previous output. It removes only files its own manifest listed, and never overwrites a file it didn't write: a file in the way is an error. Two builds can't write to one output directory at once.

### Exit codes
@id: build-exit-codes

| Code | Meaning |
|---|---|
| `0` | Built |
| `1` | The checks found errors; nothing was written |
| `2` | The build couldn't run: a usage error, no `ascribe.toml`, a content model with errors, an `ascribe.baseline` with a line that doesn't parse (`--no-baseline` builds without the file), another build writing to the output directory, a file in the output directory that Ascribe didn't write and would have to overwrite, or two pages of the site output with the same route |

## `ascribe fmt`
@id {permalink=true}: cli-fmt
@summary: Rewrites Ascribe constructs into canonical form: the spacing of directive lines and attribute blocks, attribute order, quoting, and blank lines. It never changes how a page renders.

Rewrites Ascribe constructs into canonical form: the spacing of directive lines and attribute blocks, attribute order, quoting, and blank lines between directives and their blocks. It changes nothing else, never how a page renders, and leaves alone a construct that has an error. See [Canonical form](../writing/syntax.md#canonical-form).

It lists each file it changed. With `--check`, it changes nothing and lists each change it would make: the file and line, then the lines as they are, marked `-`, and as they'd be, marked `+`:

```text
docs/install.md:5
- @note{ type = tip }:Run it from the project root.
+ @note {type=tip}: Run it from the project root.
```

`--diff` writes the changes as a unified diff instead, which `git apply` applies, and also changes nothing.

@include: ../_generated/cli-fmt-options.md

With `--format json` and `--check`, it writes the edits it would make and no file: what a tool that applies edits itself needs, such as the MCP server's `ascribe_fmt`. An edit has the shape of a fix's edit in [`ascribe check`'s JSON](check.md#json-output), and the [schema](../contracts/json-reports.md#ascribe-fmt) has every field.

A symbolic link it leaves alone is reported on standard error, as `check` reports it ([`source-unreadable`](diagnostics/pages.md#ASC123)), and the other files are still formatted.

| Code | Meaning |
|---|---|
| `0` | Every file was formatted, or, with `--check` or `--diff`, none needed it |
| `1` | With `--check` or `--diff`: a file would change |
| `2` | A problem: no `ascribe.toml`, a content model with errors, a path or file that can't be read, or a symbolic link that leads out of the content root |
