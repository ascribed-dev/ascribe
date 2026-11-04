---
type: reference
title: "`ascribe link`, `refs`, and `rename`"
description: "Check a link before writing it, list where a page, a heading, or a phrase is used, and rename one with everything that uses it."
---

`ascribe link` and `ascribe refs` answer questions about what a page links to, without changing anything, and `ascribe rename` changes a name and every reference to it. The other commands that answer questions, such as `ascribe outline` and `ascribe search`, are in [Questions about a project](questions.md).

@include: ../_fragments/command-options.md

## `ascribe link`
@id {permalink=true}: cli-link

@summary
Says whether a link works, written on a given page: whether its target exists, the page's title or the heading's text, and the destination to write there. It resolves the link as `ascribe check` and the editor do, so the three agree.

@include: ../_generated/cli-link-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/link.txt

When the target doesn't work, it says why and lists the closest targets that do:

- For a heading that isn't there, the page's headings: first those whose ids start with the words of the one written, as `ask-instead` does `ask-instead-of-guessing`, then those whose words start the written one's, then the rest, closest first.
- For a page that isn't, the pages with the heading the link names, wherever they are, then pages with a similar path.
- For a fragment, the pages that include it.
- For a published route, the file it belongs to.

`ascribe check` offers the same targets as the broken link's fixes, marked `unsafe`. When one clearly wins, such as the only heading whose id starts with the written id's words, its message ends with "did you mean `loop.md#ask-instead-of-guessing`?", so the concise output and the edit hook carry it too.

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/link-missing.txt

For a page or a heading, `Write:` gives the link with its title and with empty text, `[](../keys.md#rotate-keys)`, which shows the same title: either does, and the empty one is the smaller edit to a link that has no text.

A URL with a scheme isn't checked: it exists as far as `link` knows. A file that isn't a page, such as an image, exists when the project has it. A link to an API's operation exists when the spec has it, and the answer says which: see [Link to operations in the editor](../openapi/editor.md#for-agents).

| Code | Meaning |
|---|---|
| `0` | The target exists |
| `1` | It doesn't |
| `2` | It couldn't run: no `ascribe.toml`, one with errors, or a `--from` that isn't a page or fragment of the project |

With `--format json`, the document is:

@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/link.json

- `from` is the page as `--from` gave it, from the folder the command was run in, the way `ascribe check` names a file, whether it was given that way or from the content root. `target` is the target as given.
- `exists` says whether the link works. `path` is the file the target names, from the content root, `id` the heading's id, and `title` the page's title or the heading's text.
- `kind` is `page`, `heading`, `fragment`, `file`, `operation`, `external`, or `missing`.
- `href` is null when the target doesn't work, and `problem` says why. `closest` lists other targets, best first, each with `href`, `path`, `id`, and `title`.
- With a versioned build, `lines` says where a link to a page goes in each line.
- For an operation, `operation` has its `api`, `operation_id`, `method`, `path`, `summary`, `tag`, `deprecated`, and `address`.

Every field is in [the schema](../contracts/json-reports/questions.md#ascribe-link).

## `ascribe refs`
@id {permalink=true}: cli-refs
@summary
Lists the places that use a page, a fragment, a heading, an image, or an entry of the content model, each as `file:line:column`, with what kind of use it is.

It's the editor's **Find All References**, on the command line: it finds a phrase only where it's a phrase, and a heading in a fragment through every page that includes it, which a text search can't.

@include: ../_generated/cli-refs-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/refs.txt

A target that's a path, with or without `#id`, finds its project from the path. An entry of the model, or a glossary term, is written with a prefix: `phrase:product` (or `phrase:{product}`), `feature:sso`, `term:api-key`, `dimension:deployment`, `note:warning`, or `widget:release-note`, and the project is found from the current folder, or from `--config`. A target whose prefix isn't one of these, such as `foo:bar`, is read as a path: one that names no file, so it doesn't exist, and exits with `1`. The places are in path order, and in order within a file. A use is `link`, `include`, `phrase`, `availability`, `term`, `variant`, `note`, or `widget`.

An `availability` place shows its spec as written, after the use: ``docs/keys.md:25:13: availability `self-managed 3.5` ``.

A line of a release target, one with `[releases.<target>]`, is written `release:<target>@<version>`. Its places are the availability specs that name a version in that line, as each is written, with `planned` read as the target's planned line. A line holds every version that starts with its parts, so `release:ascribe@0.4` finds `ascribe 0.4.1` and `ascribe (preview 0.4.0, ga 0.5.0)`, and `release:ascribe@0.4.1` only the first. A spec that names the target's dimension, such as `release`, names no version of it. A target with no `[releases.<target>]` doesn't exist.

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/refs-release.txt

An image under the content root is a target too, named as a page is: `ascribe refs images/keys.png`. Its places are each file that shows it (`image`), links to it (`link`), names it in raw HTML such as `<img src>` (`html`), or names it in a frontmatter field such as `cover` (`frontmatter`): the uses that keep [`image-unused`](../checks/diagnostics/project-checks.md#ASC159) from reporting it. An image nothing uses has no places, and exits with `0`, as any target that exists does.

A [memo file](../writing/memo-files.md) about the target is a place too, with the use `memo`, at its `about` line, after the uses. One whose `about` names a target that doesn't exist is listed under that target, so `ascribe refs` finds the memos a rename left behind.

| Code | Meaning |
|---|---|
| `0` | The target exists, used or not |
| `1` | It doesn't exist |
| `2` | It couldn't run: no `ascribe.toml`, one with errors, an entry it can't read, such as `phrase:` with no name, a heading named on an image, or a file that's neither a page, a fragment, nor an image |

With `--format json`, the document is:

@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/refs.json

`kind` is what the target is: `page`, `fragment`, `heading`, `image`, `phrase`, `feature`, `term`, `dimension`, `note`, `widget`, or `release`. Each place's `spec` is its availability spec as written, for an `availability` place, and null for any other. `total` is how many places use the target, and `shown` how many are listed. When the list was cut, `truncated` is true and `next_command` is the command that lists them all; otherwise it's null.

## `ascribe rename`
@id {permalink=true}: cli-rename
@summary: Renames a phrase, a dimension's value, a heading's id, or a page, and changes everything that uses it, including what a text search and replace misses.

Renames a phrase, a dimension's value, a heading's id, or a page, and changes everything that uses it, as the editor's rename does: it makes the same edits. A text search and replace finds a phrase in the pages, but misses what a rename knows about: frontmatter fields with `phrases = true`, code blocks with `phrases=true`, the code a `@snippet {phrases=true}` shows, and a link whose path changes when the page it's on moves.

@include: ../_generated/cli-rename-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/rename.txt

What it renames, written as `ascribe refs` writes a target:

| Target | What changes |
|---|---|
| `phrase:cloud` (or `phrase:{cloud}`) | Its key under `[phrases]`, and each `{cloud}` that uses it. |
| `value:self-managed` | A dimension's value: in `ascribe.toml` (its `values`, `labels`, and `versionless`, features' `available`, and builds' `variants` and `filter`), and in pages (`@variant` attributes, `variant` and `available` frontmatter, `@available` lines, and table rows' `available`). |
| `keys.md#rotate-keys` | A heading's id, and each link and include that names it, each `[moved]` value in `ascribe.addresses` that names it, and each [memo file](../writing/memo-files.md)'s `about` that names it, in `memos/` and `.ascribe/memos/`. A heading with an `@id` has it changed. One whose id is made from its text gets an `@id` line under it, so its text stays, and links to the headings after it that were numbered after it follow. |
| `keys.md` | The page or fragment moves to the new path, and each link and include to it, or in it, whose path changes is changed. So is what `ascribe.toml` names it by: its path in `[glossary] pages`, and a phrase that's a link to it; and each `[moved]` value in `ascribe.addresses` that leads to it, keeping its `#id`. So is each memo file's `about` that names it or a heading in it, keeping its `#id`, and its entry in its section's `pages` in [`ascribe.nav`](../navigation/index.md#the-file), which is taken out when it moves to another folder. A section written inline, as `sections = [{ … }]`, isn't followed: the file is left as it is, and `ascribe check` reports the name that's gone. The new path is read as the target was: from the current folder, or from the content root. A new path that changes only the file's case, such as `Keys.md`, renames it on a file system that matches names without case too, as macOS's and Windows' do. |

A path is from the current folder, or from the content root, and the project is found from it; for a phrase or a value, the project is found from the current folder. `--config` names the project for any target, so `ascribe rename keys.md guides/keys.md --config docs` works from the repository's root. A page that isn't found is named with both places it was looked for.

It writes the files it changes, then moves the page, and prints each file it changed, from the current folder. It changes only files git could put back, as every [command that writes](index.md#commands-that-write) does: a file with changes not committed is refused, and named, unless you pass `--force`. With `--dry-run`, it says what it would change and move, and changes nothing.

Some things aren't renamed or moved:

- A permalink's id isn't renamed, since links from outside the docs use it: change its `@id` by hand, then move the old name to the new one, or retire it, with [`ascribe permalinks retire`](../permalinks/command.md#cli-permalinks-retire).
- A page isn't moved through a folder that's a symbolic link.
- A phrase used in the code of a source in another repository isn't renamed either, since that code is a copy.

| Code | Meaning |
|---|---|
| `0` | It renamed |
| `1` | What it names doesn't exist: a phrase, a value, a page, or a heading |
| `2` | It couldn't rename: no `ascribe.toml`, one with errors, a new name that can't be its name or is taken, a target it doesn't rename, a file it can't write, or, without `--force`, a file git couldn't put back |

With `--format json`, it writes what it changed as one document, instead of a line for each file:

- `files`: each file it changed, from the current folder, with its whole `text` after and how many places changed (`edits`), in path order, the page that moved under where it was.
- `moved`: the page's `from` and `to`, when one moved.
- `recorded`: each entry of `ascribe.addresses`'s `[moved]` it recorded, as a `key` and a `value`, as [`ascribe addresses move`](../publishing/addresses.md#cli-addresses-move) writes one.
- `written`: `true`, and `false` with `--dry-run`. The MCP server's [`ascribe_rename`](../agents/commands.md#cli-mcp) gives the same document with `written` false, and writes nothing.

Its [schema](../contracts/json-reports/questions.md#ascribe-rename) has every field.
