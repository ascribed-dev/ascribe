---
title: Command reference
description: "The ascribe commands: their options, outputs, and exit codes."
---

The `ascribe` command checks, builds, and formats an Ascribe project, and runs the language server the editor uses. Its `explain`, `model`, `outline`, `link`, `refs`, and `render` commands answer questions about a project, for writers and for agents, without changing anything.

@include: ../_generated/cli-synopsis.md

Install it in a project with `npm install --save-dev @ascribed/cli` (see [Getting started](../getting-started.md#install-the-command)) and run it with `npx ascribe`, or from a script in `package.json`.

## Options every command accepts

@include: ../_generated/cli-global-options.md

## `ascribe check`

Checks every source file for problems, without building anything: every file-level diagnostic, then the page-level ones for **every build** in `ascribe.toml`. The editor and `ascribe build` run the same checks, so they report the same diagnostics.

@include: ../_generated/cli-check-options.md

How diagnostics are reported:

- **Each problem is reported once**, however many builds it appears in. One that isn't in every build names the builds it's in, at the end of its message (`only in build cloud`).
- **A problem in included content is reported at the include**, with its place in the fragment as related information.
- **Content that no build publishes is checked too**, so a problem in an arm no build selects isn't missed. Its diagnostics say "in content that no build publishes". Not with `--build`, which is about one build.

Every diagnostic, with its fix, is in the [diagnostics reference](diagnostics.md).

### Checking some files

@available: next
`ascribe check docs/guides/install.md` reports on that file alone. Paths are relative to the current directory, and each is a file or a directory. Without `--config`, the project is the nearest `ascribe.toml` at or above the first path, so the command works from a repository's root as well as from inside the project.

@available: next
The whole project is still checked, since links, includes, and ids need it, and only the diagnostics that count for the paths are shown. **A diagnostic counts for a path when its file, or the file of one of its related places, is in that path.** So checking a page shows the problems its fragments cause on it, and checking a fragment shows the problems in it. A fragment's problem is reported at every page that includes it; checking the fragment shows it once, at the first include, and its `repeats` says at how many other includes it appears too. The diagnostics are the whole check's, filtered this way, in the same order.

@available: next
A path that doesn't exist, isn't in an Ascribe project, or is in another project than the first path's (a project nested in the content root, say) is exit code `2`, with a message naming it.

### Checking text before it's saved

@available: next
`--stdin --path docs/guides/new.md` checks standard input as that file's text, laid over the project on disk. Nothing is written, and the file doesn't have to exist, but it has to be where a source file could be: a `.md` file in the content root. Only the diagnostics that count for that file are shown, so a link from another page that the new text breaks isn't. `--stdin` without `--path`, or with other paths, is a usage error.

```sh
npx ascribe check --stdin --path docs/guides/new.md < draft.md
```

### A quick check after each edit

@available: next
`--editor-build` runs what the editor runs as you type: the file-level checks and the page-level checks of the editor's build only (`[editor] build`, or the first build), without the pass over content no build publishes. With paths that name files, only those files and the pages that include them are checked, which is quick enough to run after every edit; the timings are in [`tests/corpora/RESULTS.md`]({repo}/blob/main/tests/corpora/RESULTS.md#one-file). It can't be combined with `--build`. The summary line, and `builds_checked` in the JSON, name the build, so a clean result says what it covers. A full `ascribe check` before you finish still covers every build.

### Output for agents

@available: next
`--format concise` writes one line per diagnostic, `file:line: [code] message`, grouped by file in file order and by line within a file, then the summary line. It shows at most 50 diagnostics, then `and N more:` with the command that narrows the check to the next file:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/check-concise.txt

@available: next
`--format prompt` writes a prompt for an agent that fixes the problems: about the file, when the paths name one file, and otherwise about the paths or the whole project. It lists at most 20 problems, or 20 files, then names the command that lists the rest, and ends with how to check the result. It writes nothing when there are no problems, and the exit codes are the same. With `--stdin`, it says the file has unsaved changes. See [Prompt your agent](../guides/agents.md#prompt-your-agent).

@available: next
`--summary` replaces the list with how many diagnostics each code and each file has, most first, in any format. On a project with hundreds of warnings, it shows which rule or file to work through first.

### Exit codes

| Code | Meaning |
|---|---|
| `0` | No errors. Warnings don't fail the command unless you pass `--deny-warnings`. |
| `1` | There are errors, or warnings with `--deny-warnings`. |
| `2` | The project couldn't be checked: a usage error, no `ascribe.toml`, a content model with errors (they're shown, and nothing else is checked), or a path that doesn't exist or isn't in the project. A source file that can't be read, or isn't UTF-8, is a `source-unreadable` error in the list instead, and the rest of the project is still checked. |

### Text output

Each diagnostic shows its code, message, and source, and the output ends with a summary. With paths, the summary says how many files it reported on (`checked 12 files, reported on 1`):

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/check.txt

Diagnostics go to standard output. A failure that stops the command (exit code 2) goes to standard error.

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
| `diagnostics` | array | Every diagnostic, in file order, and in source order within a file. With paths, those that count for them. With `--summary`, none. |
| `truncated` | boolean | Whether `diagnostics` leaves some out, as it does with `--summary` |
| `shown` | number | How many diagnostics `diagnostics` lists |
| `total` | number | How many there are |
| `next_command` | string or null | When `truncated`, the command that lists the rest |
| `summary` | object | `errors` and `warnings`: how many of each. With `--summary`, also `by_code` (`{code, slug, severity, count}`) and `by_file` (`{file, errors, warnings}`), most first. |

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
| `fixes` | array | Edits that would fix it: `{title, file, edits, applicability}`, where each edit is `{range, new_text}` and replaces the text in `range`. `applicability` is `"safe"` when applying the edits can't change what the page says and leaves nothing to decide, such as linking to a page's file instead of its route, and `"unsafe"` otherwise, such as the nearest spelling of a misspelled name. |
| `builds` | array of strings | The builds a page-level diagnostic appears in, in `ascribe.toml`'s order. Empty for a file-level diagnostic, and for one in content no build publishes. With `--build`, only that build. |
| `unpublished` | boolean | `true` for a problem in content that no build publishes |
| `help` | string | How to fix it, in general: the advice in the [diagnostics reference](diagnostics.md) |
| `docs` | string | The address of its entry in the diagnostics reference |
| `repeats` | number | For a problem in a fragment, when checking the fragment: at how many other includes it's reported too. `0` otherwise. |

A position is `{line, column, offset}`: `line` and `column` start at 1, `column` counts Unicode characters (not bytes or UTF-16 units), and `offset` is the byte offset from the start of the file. A range's `end` is just past its last character; an edit that inserts text has equal positions.

@snippet {phrases=true}: code:crates/ascribe-cli/tests/output/check.json

## `ascribe build`

Checks the project, then writes each build's outputs. The checks run first, exactly as `ascribe check` runs them, and print what it prints. **If any check finds an error, nothing is written, for any build.**

@include: ../_generated/cli-build-options.md

@available: next
The Astro integration turns the source anchors that `--anchors` writes into attributes on every block's element ([site-render contract](../contracts/site-render.md#7-source-anchors)).

Progress (`built cloud/plain: 3 pages, 2 assets`) and warnings go to standard error.

### Outputs

Each build writes each output to `<output-dir>/<build>/<output>/`, `.ascribe/build/` being the default output directory, with a manifest beside it (`<output>.manifest.json`) listing every file it wrote. Every output is self-contained: images and other files that pages use are copied into it.

| Output | What it is | For |
|---|---|---|
| `site` | Markdown with web components (`<ascribe-note>`, `<ascribe-tabs>`, and the rest), heading ids and image attributes as markers the Astro integration applies, and the generated Zod schema in `_ascribe/schema.ts` | An Astro site, through `@ascribed/astro` ([Astro](../guides/astro.md)) |
| `plain` | Fully resolved CommonMark with no HTML: notes as quotes, [variants](directives.md#variant) as labeled sections, availability as text, and links as absolute URLs | Search indexes, LLMs, and export |
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

## `ascribe diff`
@available: next

Shows what changed between a git revision and the working tree, as readers will see it: which pages of each [build](content-model.md#16-buildsname) changed, and which blocks on them were added, removed, changed, or moved. It compares **resolved pages**, not files, so a page whose own file didn't change but whose included fragment, [phrase](directives.md#phrases), or build settings did is listed, with what its change comes from. A change that doesn't reach the page (`ascribe fmt`, rewrapped lines) isn't.

@include: ../_generated/cli-diff-options.md

The other side is the working tree: the files on disk, committed or not, as `ascribe build` would read them now. Unsaved editor changes aren't included. The base is read from git, `ascribe.toml` included, so a change to the content model is compared too; if the project didn't exist at the base, every page is added. `ascribe diff` needs `git` on the path, and nothing else: no network and no GitHub account.

The text output lists each build's changed pages with counts, and where a change comes from when it isn't only the page's own file:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/diff.txt

Pages with errors are compared as they are, since comparing work in progress is useful, but a broken page can render oddly and read as part of the change. So when `ascribe check` would find errors for the builds compared, `ascribe diff` says so on standard error, and the [HTML report](#the-html-report) and review in the editor's preview and the site preview show a notice:

```text
warning: the working tree has 3 errors; `ascribe check` lists them
```

### Exit codes
@id: diff-exit-codes

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

A tab's label and a `details`' summary say what changed in what they can hide: "new" for an added tab, else how many changes ("2 changes").

**Show: Changes / As it will be / As it was** switches between the marks, the page as it will be with none, and the page as it was at the base. The arrows step through the changes ("3 of 10 on this page"), and after the last one offer the next changed page. Hovering over a block shows the source file and line it came from (`guides/install.md:12`). The colors work in light and dark, and every mark has a label as well as a color.

A report renders at most 300 changed pages; the rest are listed by name, and the report says so at the top. Each rendered page and image is stored once, however many builds or pages share it. [The report in CI](../guides/review.md#the-report-in-ci) has a GitHub Actions job that uploads the report on every pull request.

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
| `because` | array of strings | The other changed files its change comes from, relative to the content root: fragments it includes, and pages its links take their text or a heading id from (a link to a page that changed otherwise isn't a cause). Then the [snippets](directives.md#snippet) whose code changed, by address (`code:service/client.py#connect`). `ascribe.toml` comes last when a change to the content model (a phrase's value, a label, a build's settings) is a cause. |
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

@snippet {phrases=true}: code:crates/ascribe-cli/tests/output/diff.json

How blocks are matched: blocks are compared by their content without positions, so moving a block down the file or rewrapping it is no change. Blocks that didn't stay are paired, in order, with a block of the same kind whose words overlap by at least half, as **changed**; a block that matches nothing is **added** or **removed**, and a removed block identical to an added one elsewhere on the page is **moved**. A block over about 2,500 words gets no word ranges, and a list or container with over 1,000 blocks in it that changed is marked changed as a whole.

## `ascribe drift`
@available: next

Lists the pages whose code examples changed between a git revision and the working tree, and whether the words around them changed too, and the pages whose examples no longer resolve. A page covers the code it shows, and nothing else: every region it takes a [snippet](directives.md#snippet) from, and every whole file it takes as one, including its fragments' snippets. It's for CI, where it says which pages a reviewer should reread; see [Drift](../guides/drift.md#when-an-example-changes).

@include: ../_generated/cli-drift-options.md

The base is found as `ascribe diff` finds it, and the other side is the working tree. A snippet's region is taken from its file on both sides and compared as text, dedented, with trailing spaces trimmed, so an edit elsewhere in the file, a region that only moved, or a change of indent isn't a change. A renamed file is followed. A region the file didn't have at the base is a new example, not a changed one. Then, for each page with a changed example, it compares the page itself as `ascribe diff` does, apart from its snippets: the page changed when its own file did, or when anything else it uses did, such as a fragment it includes or a phrase in `ascribe.toml`.

The pages come in up to three groups:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/drift.txt

An example **no longer resolves** when it did at the base and doesn't now: its region was renamed, its file moved, or its source is gone. `ascribe check` fails on it too, but the report names the page so the pull request that broke it says so. A `@snippet` the change adds broken isn't listed; that's `check`'s alone. Of the changed examples, the first group is the one to read: the example updated itself, and nobody changed the sentence that explains it. The second is listed so a reviewer can see the change reached the page. Each example says how many lines were added and removed. With nothing to report, it says `No examples changed.`

A changed example isn't always a changed behavior. The report says what changed and how much, and leaves the judgment to you.

It reads little: one `git diff` listing of the change, and at the base only the files that listing names that a snippet uses. The base's pages are read only when an example changed. `ascribe drift` needs `git` on the path, and nothing else: no network and no GitHub account.

### The summary

`--format summary` writes Markdown for a CI job's summary: the same groups, each page linked to its route on `[consumer] site` (plain text when there's no `site`), and nothing at all when no example changed, so it can be appended to `$GITHUB_STEP_SUMMARY` on every run.

### Exit codes
@id: drift-exit-codes

| Code | Meaning |
|---|---|
| `0` | Compared, whatever it found |
| `1` | With `--exit-code`: an example no longer resolves, or a page's example changed and the page didn't |
| `2` | It couldn't run: a usage error, an unknown build, not a git repository, an unknown revision, no merge base (a shallow clone, or unrelated histories), `git` not found, or a project that doesn't load at the base or in the working tree. The reason goes to standard error. |

### Drift JSON

`--format json` writes one JSON document to standard output, with the [JSON output](#json-output)'s rules: `schema_version` changes only when a field is removed or changes meaning, so **ignore fields you don't know**.

| Field | Type | Meaning |
|---|---|---|
| `schema_version` | number | `1` |
| `ascribe_version` | string | The version of `ascribe` that wrote the report |
| `base` | object | As in the [diff JSON](#diff-json): `requested`, `commit`, and `merge_base` |
| `repository` | object | As in the diff JSON: `root` and `project_prefix` |
| `pages` | array | Every page with an example that changed or no longer resolves, in path order |

Each page:

| Field | Type | Meaning |
|---|---|---|
| `path` | string | The page's path, relative to the content root |
| `route` | string | Its route, in the first build that shows it |
| `builds` | array of strings | The builds that show it with a changed example |
| `page_changed` | boolean | Whether the page changed apart from its examples, in any of those builds. With `false`, its changed examples are in the group to read. |
| `examples` | array | The examples that changed, by address |
| `broken` | array | The examples that resolved at the base and don't now, by address: `address`, `source`, `problem` (the slug `ascribe check` reports, such as `snippet-region-missing`), and `reason`, in a few words |

Each example:

| Field | Type | Meaning |
|---|---|---|
| `address` | string | The snippet's address, as the page writes it (`code:service/client.py#connect`) |
| `source` | string | The source it names |
| `file` | string | The code file, relative to the repository's root |
| `was_file` | string or null | The code file at the base, when it was renamed since |
| `added` | number | Lines only in the code now |
| `removed` | number | Lines only in the code at the base |

## `ascribe fmt`

Rewrites Ascribe constructs into canonical form: the spacing of directive lines and attribute blocks, attribute order, quoting, and blank lines between directives and their blocks. It changes nothing else, never how a page renders, and leaves alone a construct that has an error. See [Canonical form](directives.md#canonical-form).

It lists each file it changed.

@include: ../_generated/cli-fmt-options.md

It formats only what `ascribe check` reads: a file in the content root that's a symbolic link, or is in a linked folder, is formatted only when the link leads to a source file of the content root. One that leads anywhere else is left alone and reported on standard error, as `check` reports it ([`source-unreadable`](diagnostics.md#asc123-source-unreadable)), and the other files are still formatted.

| Code | Meaning |
|---|---|
| `0` | Every file was formatted, or, with `--check`, none needed it |
| `1` | With `--check`: a file would change |
| `2` | A problem: no `ascribe.toml`, a content model with errors, a path or file that can't be read, or a symbolic link that leads out of the content root |

## `ascribe lsp`

Runs the language server, speaking the Language Server Protocol over standard input and output. An editor starts it; you don't run it yourself. It takes no options of its own, and refuses `--config` (exit code `2`): its project is the nearest `ascribe.toml` at or above the workspace folder the editor gives it, never one below, and `ascribe.toml`'s `[editor] build` says which build's page-level diagnostics to report. Its logs go to standard error, starting with the project it uses.

The VS Code extension runs it for you, one server for each project in the workspace. See [Editing](../guides/editor.md). Any editor with an LSP client can run `ascribe lsp` too; for several projects, start one per project ([Other editors](../guides/editor.md#other-editors)).

## `ascribe sources`
@available: next

Copies code from [sources in other repositories](content-model.md#a-source-in-another-repository) into the project, and moves their pins. `check` and `build` read only the copies, so `ascribe sources fetch` and `ascribe sources update` are the only commands that reach another repository. They run `git`, with `git`'s own credentials: whatever lets `git fetch <url>` work in your shell lets them fetch. They never prompt, so a missing credential fails at once.

Fetched repositories are kept in a cache outside the project, one per URL, so a second run fetches only what's new: `ascribe` in your cache folder (`$XDG_CACHE_HOME`, `~/.cache`, `~/Library/Caches`, or `%LOCALAPPDATA%`), or `ASCRIBE_CACHE_DIR`. Nothing needs the cache but these commands; deleting it costs only a slower next run.

What's fetched is someone else's repository, so it's only ever read as text: `git` runs with no hooks, no submodules, and only the `https`, `http`, `ssh`, `git`, and `file` transports, and a file over 1 MB, or one that isn't text, isn't copied.

### `ascribe sources fetch`

Makes the copies match `ascribe.lock`: at each source's pin, it copies the files snippets name that have no copy or a wrong one, and removes the copies no snippet names. It's what you run after writing a new `@snippet` from a source in another repository, or after cloning if the copies were left out. It never moves a pin; a source with no pin yet is pinned to the head of its branch.

@include: ../_generated/cli-sources-fetch-options.md

It lists each copy it wrote or removed. The first time a source's files are copied, it says so: the copies are committed with the docs, so everyone who can read the docs repository can read them.

| Code | Meaning |
|---|---|
| `0` | The copies match the lock |
| `1` | A file a snippet names couldn't be copied: it isn't in the repository at the pin, it's over 1 MB, or it isn't text. The reason goes to standard error, and `ascribe check` reports the snippet. |
| `2` | It couldn't run: no `ascribe.toml`, a content model with errors, an `ascribe.lock` it can't read, a name that isn't a source in another repository, or `git` failing (no network, no access), with the source named and `git`'s own message |

### `ascribe sources update`

Moves each source's pin to the head of its `branch` (or to `--to`), copies the files snippets use again at the new commit, rewrites `ascribe.lock`, and says what changed: the commits between the old pin and the new (how many, and the first lines of the newest 20, or that the pin moved back), the copies that changed, and the pages whose examples changed, grouped as [`ascribe drift`](#ascribe-drift) groups them. With nothing to move, it changes no file and says so.

@include: ../_generated/cli-sources-update-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/sources-update.txt

The pages are found as `ascribe drift` finds them, comparing the working tree, with the new copies, against `HEAD`, so run it in a clean checkout. A snippet whose region is gone at the new commit is listed among the examples that no longer resolve, and the update still completes: `ascribe check` then fails on it, which is the signal to fix the page. When the project isn't in a git repository, or has no commit yet, the pages aren't listed, and it says why: commit the docs once before the first update.

`--format summary` writes Markdown for a pull request's description: each source that moved, with its commits and copies, then the pages, each linked to its route on `[consumer] site`. For a repository on github.com, the old and new pins link to GitHub's comparison of them. Commit subjects are code spans, so a subject like `Fixes #12 @someone` in the code's repository can't close an issue or notify anyone from the docs' pull request. It writes nothing when nothing moved. `--format json` writes one document: `schema_version` (`1`), `ascribe_version`, `changed`, `sources`, `pages` (as in the [drift JSON](#drift-json), or null), and `pages_unavailable` (why there are no pages, or null). Each source has `name`, `git`, `followed` (the branch, `HEAD`, or `--to`'s revision), `from` and `to` (the pins, `from` null for a first pin), `moved`, `back` (true when the new pin isn't after the old one: it moved back, or to another line of history, and no commits are counted), `commits` (`count` and `newest`, each with `commit` and `subject`, or null), `files` (each with `path` and `change`: `added`, `changed`, or `removed`), `failed` (each with `path` and `reason`), and `first_copy`.

| Code | Meaning |
|---|---|
| `0` | It ran, whether or not a pin moved. A file a snippet names that couldn't be copied is reported on standard error. |
| `2` | It couldn't: no network, no access, an unknown revision, `--to` with more than one source, or the reasons `fetch` gives. The source is named, with `git`'s own message. |

### `ascribe sources status`

Shows each source in another repository: its repository and branch, its pin, and each copy's state (current, changed here, missing, not in the lock, unused, not copied yet, or not in the repository at the pin). It reads only the project's files: no `git`, no network. It exits with `0` whatever it finds, and `2` when the project doesn't load.

@include: ../_generated/cli-sources-status-options.md

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, and `sources`, each with `name`, `git`, `branch` (or null), `commit` (the pin, or null), and `files`, each with `path` and `state`: `current`, `changed`, `missing`, `unlocked`, `unused`, `not_copied`, or `not_at_pin`.

## `ascribe explain`
@available: next

Says what a diagnostic means: its severity, its messages, how to fix it, a link to its entry in [Diagnostics](diagnostics.md), and, for the diagnostics people meet most, a short page that has the problem and the same page without it. It needs no project.

@include: ../_generated/cli-explain-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/explain.txt

An example that needs something in `ascribe.toml` to go wrong shows that too: a few lines of the model it was checked against. Every example is checked, so each wrong page reports its diagnostic and nothing else, and each right page reports nothing. `ascribe explain --list` lists every diagnostic's code and name, one per line.

| Code | Meaning |
|---|---|
| `0` | It explained the diagnostic, or listed them |
| `2` | No diagnostic has that code or name, and it names the closest; or a usage error |

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, `code`, `slug`, `severity`, `level` (`file` or `page`), `message`, `variants` (the other messages it can give, each with `name` and `message`), `fix`, `docs` (the link), and `example`, null or with `wrong`, `right`, `model` (the model it was checked against, when it matters, or null), and `files` (other files the right page needs, each with `path` and `text`). With `--list`, it has `diagnostics`, each with `code`, `slug`, and `severity`.

## `ascribe model`
@available: next

Shows the content model as `ascribe` reads it, with the defaults filled in: page types with their files and frontmatter fields, dimensions, phrases, features, glossary terms, project widgets, and builds. It's what an agent, or a new writer, reads before writing frontmatter, a directive's attributes, or a phrase.

@include: ../_generated/cli-model-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/model.txt

The text is Markdown, at most about 4,000 characters: when the model has more than fits, each long list is cut, and the cut says which `--section` shows the rest. A section shown with `--section` is never cut. A section the model leaves empty isn't shown. When `ascribe.toml` has errors, it says to run `ascribe check`, and exits with `2`.

| Code | Meaning |
|---|---|
| `0` | It showed the model |
| `2` | No `ascribe.toml`, or one with errors |

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, and a field for each section shown: `types`, `dimensions`, `phrases`, `features`, `glossary`, `widgets`, and `builds`. Without `--section`, every section is there, empty or not; with it, only that one. The [schema](../contracts/json-reports.md#ascribe-model) lists each section's fields.

## `ascribe outline`
@available: next

Shows a page's title and type, and the headings a link to it can name, with the ids links write after `#`: the page's own headings and those that come from the fragments it includes. They're the headings the editor offers after `page.md#`.

@include: ../_generated/cli-outline-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/outline.txt

A heading from a fragment says which fragment. With `--build`, it shows only the headings that build publishes: a heading in a variant the build leaves out, or in a block it filters out by availability, isn't listed. When the build doesn't publish the page at all, it says why, lists no headings, and exits with `1`. A fragment's outline lists its headings, and those of the fragments it includes.

| Code | Meaning |
|---|---|
| `0` | It showed the outline |
| `1` | With `--build`: the build doesn't publish the page |
| `2` | It couldn't run: no `ascribe.toml`, one with errors, a path that isn't a page or fragment of the project, or an unknown build |

With `--format json`, the document is:

@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/outline.json

`page` is the path from the content root, as links write it, and `file` is from the project root, as `ascribe check` reports it. `type` is null for a fragment. `explicit_id` is true when the heading has an `@id`, which stays when its text changes; prefer those in links. `fragment` is the fragment a heading comes from, or null.

## `ascribe link`
@available: next

Says whether a link works, written on a given page: whether its target exists, the page's title or the heading's text, and the destination to write there. It resolves the link as `ascribe check` and the editor do, so the three agree.

@include: ../_generated/cli-link-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/link.txt

When the target doesn't work, it says why and lists the closest targets that do: the page's headings for a heading that isn't there, pages with a similar path for a page that isn't, the pages that include a fragment, or the file that a published route belongs to.

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/link-missing.txt

A URL with a scheme isn't checked: it exists as far as `link` knows. A file that isn't a page, such as an image, exists when the project has it.

| Code | Meaning |
|---|---|
| `0` | The target exists |
| `1` | It doesn't |
| `2` | It couldn't run: no `ascribe.toml`, one with errors, or a `--from` that isn't a page or fragment of the project |

With `--format json`, the document is:

@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/link.json

`kind` is `page`, `heading`, `fragment`, `file`, `external`, or `missing`. `href` is null when the target doesn't work, and `problem` says why; `closest` lists other targets, best first, each with `href`, `path`, `id`, and `title`.

## `ascribe refs`
@available: next

Lists the places that use a page, a fragment, a heading, or an entry of the content model, each as `file:line:column`, with what kind of use it is. It's the editor's **Find All References**, on the command line: it finds a phrase only where it's a phrase, and a heading in a fragment through every page that includes it, which a text search can't.

@include: ../_generated/cli-refs-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/refs.txt

A target that's a path, with or without `#id`, finds its project from the path. An entry of the model is written with a prefix: `phrase:product` (or `phrase:{product}`), `feature:sso`, `term:api-key`, `dimension:deployment`, `note:warning`, or `widget:release-note`, and the project is found from `--project`, or from the current directory. The places are in path order, and in order within a file. A use is `link`, `include`, `phrase`, `availability`, `term`, `variant`, `note`, or `widget`.

| Code | Meaning |
|---|---|
| `0` | The target exists, used or not |
| `1` | It doesn't exist |
| `2` | It couldn't run: no `ascribe.toml`, one with errors, or a target that's neither a path nor an entry |

With `--format json`, the document is:

@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/refs.json

`total` is how many places use the target, and `shown` how many are listed. When the list was cut, `truncated` is true and `next_command` is the command that lists them all; otherwise it's null.

## `ascribe render`
@available: next

Writes a page as a reader of one build sees it: the plain Markdown the build's `plain` output writes for it, with its variants chosen, the blocks the build filters out removed, phrases and includes filled in, and links written as that output writes them. It writes nothing to disk.

@include: ../_generated/cli-render-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/render.txt

`--build` is needed when `ascribe.toml` has more than one build. When the build doesn't publish the page, it writes nothing to standard output, says why on standard error, and exits with `1`. A fragment isn't rendered on its own; render a page that includes it.

| Code | Meaning |
|---|---|
| `0` | It wrote the page |
| `1` | The build doesn't publish the page |
| `2` | It couldn't run: no `ascribe.toml`, one with errors, a path that isn't a page, more than one build and no `--build`, an unknown build, or a page that can't be rendered |

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, `page`, `build`, `not_published` (why the build doesn't publish it, or null), `route` (its route in the build's site, or null), and `text`, the page as text output writes it.

## `ascribe agents`
@available: next

Writes the files AI coding agents read on their own: your project's rules, from `ascribe.toml`, and the Ascribe skill. See [Agents](../guides/agents.md).

### `ascribe agents sync`

Writes `AGENTS.md` beside `ascribe.toml`, a short block in the repository root's `AGENTS.md` when the project is in a subfolder, and the skill in the root's `.agents/skills/ascribe/`. It also keeps up to date each other target whose files exist already, and writes those `--target` names. It finds the repository's root as `ascribe diff` does, with `git`; outside a repository, the project's folder is the root, and it says so.

@include: ../_generated/cli-agents-sync-options.md

It writes only between its markers in a file it shares with your team, and whole files where the file is its own; it lists each file, `wrote` or `unchanged`. With `--check`, it lists each file as `stale` or `up to date`, and writes nothing.

| Code | Meaning |
|---|---|
| `0` | The files are written, or with `--check`, up to date |
| `1` | With `--check`: a file is out of date. Run `ascribe agents sync`. |
| `2` | It couldn't run: no `ascribe.toml`, one with errors, a file whose markers are damaged, a file that would be under the content root (so one of the project's pages), `--target copilot` outside a git repository, or a file it can't read or write. Nothing is written. |

### `ascribe agents skill`

Prints the Ascribe skill's `SKILL.md`, the same for every project. It needs no project, and exits with `0`.
