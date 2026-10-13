---
type: reference
title: "`ascribe explain`, `model`, `outline`, `link`, `refs`, `rename`, and `render`"
description: "Ask a project about its diagnostics, content model, pages, links, and references, and rename what a page links to."
---

The commands that answer questions about a project, for writers and for agents, without changing anything, and `ascribe rename`, which changes a name and every reference to it. The options every command accepts are in the [command reference](index.md#options-every-command-accepts).

## `ascribe explain`
@id {permalink=true}: cli-explain
@available: ascribe 0.3.0

Says what a diagnostic means: its severity, its messages, how to fix it, a link to its entry in [Diagnostics](../checks/diagnostics.md), and, for the diagnostics people meet most, a short page that has the problem and the same page without it. It needs no project.

@include: ../_generated/cli-explain-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/explain.txt

An example that needs something in `ascribe.toml` to go wrong shows that too: a few lines of the model it was checked against. Every example is checked, so each wrong page reports its diagnostic and nothing else, and each right page reports nothing. `ascribe explain --list` lists every diagnostic's code and name, one per line; with no code and no `--list`, it says so, and exits with `2`.

| Code | Meaning |
|---|---|
| `0` | It explained the diagnostic, or listed them |
| `2` | No diagnostic has that code or name, and it names the closest; or a usage error |

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, `code`, `slug`, `severity`, `next` (the [kind of next step](../checks/diagnostics.md#what-to-do-next)), `level` (`file` or `page`), `message`, `variants` (the other messages it can give, each with `name` and `message`), `fix`, `docs` (the link), and `example`, null or with `wrong`, `right`, `model` (the model it was checked against, when it matters, or null), and `files` (other files the right page needs, each with `path` and `text`). With `--list`, it has `diagnostics`, each with `code`, `slug`, and `severity`.

## `ascribe model`
@id {permalink=true}: cli-model
@available: ascribe 0.3.0

Shows the content model as `ascribe` reads it, with the defaults filled in: page types with their files and frontmatter fields, dimensions, phrases, features, glossary terms, project widgets, builds, and sources with their folders and the files they make readable. It's what an agent, or a new writer, reads before writing frontmatter, a directive's attributes, or a phrase.

@include: ../_generated/cli-model-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/model.txt

The text is Markdown, at most about 4,000 characters: when the model has more than fits, each long list is cut, and the cut says which `--section` shows the rest. A section shown with `--section` is never cut. A section the model leaves empty isn't shown. When `ascribe.toml` has errors, it says to run `ascribe check`, and exits with `2`.

| Code | Meaning |
|---|---|
| `0` | It showed the model |
| `2` | No `ascribe.toml`, or one with errors |

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, and a field for each section shown: `types`, `dimensions`, `phrases`, `features`, `glossary`, `widgets`, `builds`, and `sources`. Without `--section`, every section is there, empty or not; with it, only that one. The [schema](../contracts/json-reports/questions.md#ascribe-model) lists each section's fields.

## `ascribe outline`
@id {permalink=true}: cli-outline
@available: ascribe 0.3.0

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
@id {permalink=true}: cli-link
@available: ascribe 0.3.0

Says whether a link works, written on a given page: whether its target exists, the page's title or the heading's text, and the destination to write there. It resolves the link as `ascribe check` and the editor do, so the three agree.

@include: ../_generated/cli-link-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/link.txt

When the target doesn't work, it says why and lists the closest targets that do: the page's headings for a heading that isn't there; for a page that isn't, the pages with the heading the link names, wherever they are, then pages with a similar path; the pages that include a fragment; or the file that a published route belongs to. `ascribe check` offers the same targets as the broken link's fixes, marked `unsafe`.

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/link-missing.txt

For a page or a heading, `Write:` gives the link with its title and with empty text, `[](../keys.md#rotate-keys)`, which shows the same title: either does, and the empty one is the smaller edit to a link that has no text.

A URL with a scheme isn't checked: it exists as far as `link` knows. A file that isn't a page, such as an image, exists when the project has it.

| Code | Meaning |
|---|---|
| `0` | The target exists |
| `1` | It doesn't |
| `2` | It couldn't run: no `ascribe.toml`, one with errors, or a `--from` that isn't a page or fragment of the project |

With `--format json`, the document is:

@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/link.json

`from` is the page as `--from` gave it, from the folder the command was run in, the way `ascribe check` names a file, whether it was given that way or from the content root. `kind` is `page`, `heading`, `fragment`, `file`, `external`, or `missing`. `href` is null when the target doesn't work, and `problem` says why; `closest` lists other targets, best first, each with `href`, `path`, `id`, and `title`.

## `ascribe refs`
@id {permalink=true}: cli-refs
@available: ascribe 0.3.0

Lists the places that use a page, a fragment, a heading, or an entry of the content model, each as `file:line:column`, with what kind of use it is. It's the editor's **Find All References**, on the command line: it finds a phrase only where it's a phrase, and a heading in a fragment through every page that includes it, which a text search can't.

@include: ../_generated/cli-refs-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/refs.txt

A target that's a path, with or without `#id`, finds its project from the path. An entry of the model is written with a prefix: `phrase:product` (or `phrase:{product}`), `feature:sso`, `term:api-key`, `dimension:deployment`, `note:warning`, or `widget:release-note`, and the project is found from the current directory, or from `--config`. The places are in path order, and in order within a file. A use is `link`, `include`, `phrase`, `availability`, `term`, `variant`, `note`, or `widget`.

| Code | Meaning |
|---|---|
| `0` | The target exists, used or not |
| `1` | It doesn't exist |
| `2` | It couldn't run: no `ascribe.toml`, one with errors, or a target that's neither a path nor an entry |

With `--format json`, the document is:

@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/refs.json

`total` is how many places use the target, and `shown` how many are listed. When the list was cut, `truncated` is true and `next_command` is the command that lists them all; otherwise it's null.

## `ascribe rename`
@id {permalink=true}: cli-rename
@available: ascribe 0.3.0

Renames a phrase, a dimension's value, a heading's id, or a page, and changes everything that uses it, as the editor's rename does: it makes the same edits. A text search and replace finds a phrase in the pages, but misses what a rename knows about: frontmatter fields with `phrases = true`, code blocks with `phrases=true`, the code a `@snippet {phrases=true}` shows, and a link whose path changes when the page it's on moves.

@include: ../_generated/cli-rename-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/rename.txt

What it renames, written as `ascribe refs` writes a target:

| Target | What changes |
|---|---|
| `phrase:cloud` (or `phrase:{cloud}`) | Its key under `[phrases]`, and each `{cloud}` that uses it. |
| `value:self-managed` | A dimension's value: in `ascribe.toml` (its `values`, `labels`, and `versionless`, features' `available`, and builds' `variants` and `filter`), and in pages (`@variant` attributes, `variant` and `available` frontmatter, `@available` lines, and table rows' `available`). |
| `keys.md#rotate-keys` | A heading's id, and each link and include that names it. A heading with an `@id` has it changed. One whose id is made from its text gets an `@id` line under it, so its text stays, and links to the headings after it that were numbered after it follow. |
| `keys.md` | The page or fragment moves to the new path, and each link and include to it, or in it, whose path changes is changed. The new path is read as the target was: from the current folder, or from the content root. |

A path is from the current folder, or from the content root, and the project is found from it; for a phrase or a value, the project is found from the current folder, or from `--config`. It writes the files it changes, then moves the page, and prints each file it changed, from the current folder. A permalink's id isn't renamed, since links from outside the docs use it: change its `@id` by hand, then retire the old name with [`ascribe permalinks retire`](../examples/sources.md#cli-permalinks-retire). A page isn't moved through a folder that's a symbolic link. A phrase used in the code of a source in another repository isn't renamed either, since that code is a copy.

| Code | Meaning |
|---|---|
| `0` | It renamed |
| `1` | What it names doesn't exist: a phrase, a value, a page, or a heading |
| `2` | It couldn't rename: no `ascribe.toml`, one with errors, a new name that can't be its name or is taken, a target it doesn't rename, or a file it can't write |

## `ascribe render`
@id {permalink=true}: cli-render
@available: ascribe 0.3.0

Writes a page as a reader of one build sees it: the plain Markdown the build's `plain` output writes for it, with its variants chosen, the blocks the build filters out removed, phrases and includes filled in, and links written as that output writes them. It writes nothing to disk.

@include: ../_generated/cli-render-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/render.txt

A variant group the build keeps more than one arm of, which the site shows as tabs, is a line saying so, `A reader sees one of the next 3 sections at a time, chosen by operating system.`, then each arm under a heading that names its dimension and value, such as `### Operating system: macOS`, or its title for labeled arms, as the plain output writes it.

`--build` is needed when `ascribe.toml` has more than one build. When the build doesn't publish the page, it writes nothing to standard output, says why on standard error, and exits with `1`. A fragment isn't rendered on its own; render a page that includes it.

| Code | Meaning |
|---|---|
| `0` | It wrote the page |
| `1` | The build doesn't publish the page |
| `2` | It couldn't run: no `ascribe.toml`, one with errors, a path that isn't a page, more than one build and no `--build`, an unknown build, or a page that can't be rendered |

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, `page`, `build`, `not_published` (why the build doesn't publish it, or null), `route` (its route in the build's site, or null), and `text`, the page as text output writes it.
