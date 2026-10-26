---
type: reference
title: "`ascribe explain`, `model`, `outline`, `link`, `refs`, `rename`, `resolve`, and `search`"
description: "Ask a project about its diagnostics, content model, pages, links, and references, find where its pages say something, and rename what a page links to."
---

The commands that answer questions about a project, for writers and for agents, without changing anything, and `ascribe rename`, which changes a name and every reference to it. The options every command accepts are in the [command reference](index.md#options-every-command-accepts).

## `ascribe explain`
@id {permalink=true}: cli-explain

@summary
Says what a diagnostic means: its severity, its messages, how to fix it, a link to its entry in [Diagnostics](../checks/diagnostics.md), and, for the diagnostics people meet most, a short page that has the problem and the same page without it. It needs no project.

@include: ../_generated/cli-explain-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/explain.txt

An example that needs something in `ascribe.toml` to go wrong shows that too: a few lines of the model it was checked against. Every example is checked, so each wrong page reports its diagnostic and nothing else, and each right page reports nothing.

| Code | Meaning |
|---|---|
| `0` | It explained the diagnostic, or listed them |
| `2` | No diagnostic has that code or name, and it names the closest; or a usage error |

With `--format json`, the document is as follows. Its [schema](../contracts/json-reports/questions.md#ascribe-explain) has every field, and so does the schema of what `--list` writes:

@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/explain.json

## `ascribe model`
@id {permalink=true}: cli-model
@summary: Shows the content model as `ascribe` reads it, with the defaults filled in: page types and their frontmatter, dimensions, phrases, features, glossary terms, widgets, builds, and sources.

Shows the content model as `ascribe` reads it, with the defaults filled in: page types with their files and frontmatter fields, dimensions, phrases, features, glossary terms, project widgets, builds, and sources with their folders and the files they make readable. A source with an `ascribe.examples.toml` also says how many of its files are [tested examples](../examples/tested.md), and names each observation with what it was tested with and how many of its files are as it found them. For a source in another repository, that's read from its copies, and an observation's files are counted only where there's a copy. It's what an agent, or a new writer, reads before writing frontmatter, a directive's attributes, or a phrase.

@include: ../_generated/cli-model-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/model.txt

The text is Markdown, at most about 4,000 characters: when the model has more than fits, each long list is cut, and the cut says which `--section` shows the rest. A section shown with `--section` is never cut. A section the model leaves empty isn't shown. When `ascribe.toml` has errors, it says to run `ascribe check`, and exits with `2`.

| Code | Meaning |
|---|---|
| `0` | It showed the model |
| `2` | No `ascribe.toml`, or one with errors |

With `--format json`, the document has a field for each section shown, and without `--section`, every section, empty or not, and the project's [navigation](../navigation/index.md) as the sidebar shows it: a label with a code span in it is plain text, without the backticks. Its [schema](../contracts/json-reports/model.md#ascribe-model) has each section's fields. `--section builds`:

@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/model-builds.json

A project with [release lines](../versions/lines.md) has a section for them, `releases` in JSON: each target's lines, with their status and the build that is each line. A versioned build says which target versions it and which line it is.

The rest of `ascribe.toml` is there too, a section of settings for each of its tables: `project`, `consumer`, `editor`, `checks`, `crawlers`, `listing`, and `permalinks`. The text without `--section` names them at its end, and `--section` shows one, each key with the value in effect, and its default: `ascribe model --section consumer` shows `[consumer]`'s `site`, `base-path`, and the rest, the ones the file leaves out at their defaults. `[checks]` has the keys of `[checks.vale]` and `[checks.links]`, written `vale.preset` and `links.reuse-days`, and each check the file sets a level for. In JSON, each section is a list of keys, in the [schema](../contracts/json-reports/model.md#ascribe-model). The keys and defaults are the ones the loader reads, so `model` shows no key `ascribe check` would refuse.

## `ascribe outline`
@id {permalink=true}: cli-outline

@summary
Shows a page's title and type, and the headings a link to it can name, with the ids links write after `#`: the page's own headings and those that come from the fragments it includes. They're the headings the editor offers after `page.md#`.

@include: ../_generated/cli-outline-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/outline.txt

A heading from a fragment says which fragment. With `--build`, it shows only the headings that build publishes: a heading in a variant the build leaves out, or in a block it filters out by availability, isn't listed. When the build doesn't publish the page at all, it says why, lists no headings, and exits with `1`. A fragment's outline lists its headings, and those of the fragments it includes.

Under the page, and under each heading, a line lists each [memo file](../writing/memo-files.md) about it, with its first line, and `(outdated)` when the words it quotes aren't there any more. A detached memo isn't listed.

| Code | Meaning |
|---|---|
| `0` | It showed the outline |
| `1` | With `--build`: the build doesn't publish the page |
| `2` | It couldn't run: no `ascribe.toml`, one with errors, a path that isn't a page or fragment of the project, or an unknown build |

With `--format json`, the document is:

@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/outline.json

`page` is the path from the content root, as links write it, and `file` is from the project root, as `ascribe check` reports it. `type` is null for a fragment. `explicit_id` is true when the heading has an `@id`, which stays when its text changes; prefer those in links. `fragment` is the fragment a heading comes from, or null.

`memos`, on the page and on each heading, lists the memo files about it, in path order: `file`, from the project root; `state`, `placed` or `outdated`; `quote`, or null; `first_line`, the first line of its text; and `personal`, which is false for the project's `memos/`.

## `ascribe link`
@id {permalink=true}: cli-link

@summary
Says whether a link works, written on a given page: whether its target exists, the page's title or the heading's text, and the destination to write there. It resolves the link as `ascribe check` and the editor do, so the three agree.

@include: ../_generated/cli-link-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/link.txt

When the target doesn't work, it says why and lists the closest targets that do: the page's headings for a heading that isn't there; for a page that isn't, the pages with the heading the link names, wherever they are, then pages with a similar path; the pages that include a fragment; or the file that a published route belongs to. `ascribe check` offers the same targets as the broken link's fixes, marked `unsafe`.

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

`from` is the page as `--from` gave it, from the folder the command was run in, the way `ascribe check` names a file, whether it was given that way or from the content root, and `target` is the target as given. `exists` says whether the link works. `path` is the file the target names, from the content root, `id` the heading's id, and `title` the page's title or the heading's text. `kind` is `page`, `heading`, `fragment`, `file`, `operation`, `external`, or `missing`. `href` is null when the target doesn't work, and `problem` says why; `closest` lists other targets, best first, each with `href`, `path`, `id`, and `title`. With a versioned build, `lines` says where a link to a page goes in each line. For an operation, `operation` has its `api`, `operation_id`, `method`, `path`, `summary`, `tag`, `deprecated`, and `address`. Every field is in [the schema](../contracts/json-reports/questions.md#ascribe-link).

## `ascribe refs`
@id {permalink=true}: cli-refs
@summary: Lists the places that use a page, a fragment, a heading, an image, or an entry of the content model, each as `file:line:column`, with what kind of use it is.

Lists the places that use a page, a fragment, a heading, an image, or an entry of the content model, each as `file:line:column`, with what kind of use it is. It's the editor's **Find All References**, on the command line: it finds a phrase only where it's a phrase, and a heading in a fragment through every page that includes it, which a text search can't.

@include: ../_generated/cli-refs-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/refs.txt

A target that's a path, with or without `#id`, finds its project from the path. An entry of the model, or a glossary term, is written with a prefix: `phrase:product` (or `phrase:{product}`), `feature:sso`, `term:api-key`, `dimension:deployment`, `note:warning`, or `widget:release-note`, and the project is found from the current directory, or from `--config`. The places are in path order, and in order within a file. A use is `link`, `include`, `phrase`, `availability`, `term`, `variant`, `note`, or `widget`.

An `availability` place shows its spec as written, after the use: ``docs/keys.md:25:13: availability `self-managed 3.5` ``.

A line of a release target, one with `[releases.<target>]`, is written `release:<target>@<version>`. Its places are the availability specs that name a version in that line, as each is written, with `planned` read as the target's planned line. A line holds every version that starts with its parts, so `release:ascribe@0.4` finds `ascribe 0.4.1` and `ascribe (preview 0.4.0, ga 0.5.0)`, and `release:ascribe@0.4.1` only the first. A spec that names the target's dimension, such as `release`, names no version of it. A target with no `[releases.<target>]` doesn't exist.

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/refs-release.txt

An image under the content root is a target too, named as a page is: `ascribe refs images/keys.png`. Its places are each file that shows it (`image`), links to it (`link`), names it in raw HTML such as `<img src>` (`html`), or names it in a frontmatter field such as `cover` (`frontmatter`): the uses that keep [`image-unused`](../checks/diagnostics/content-checks.md#ASC159) from reporting it. An image nothing uses has no places, and exits with `0`, as any target that exists does.

A [memo file](../writing/memo-files.md) about the target is a place too, with the use `memo`, at its `about` line, after the uses. One whose `about` names a target that doesn't exist is listed under that target, so `ascribe refs` finds the memos a rename left behind.

| Code | Meaning |
|---|---|
| `0` | The target exists, used or not |
| `1` | It doesn't exist |
| `2` | It couldn't run: no `ascribe.toml`, one with errors, a target that's neither a path nor an entry, or a file that's neither a page, a fragment, nor an image |

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
| `keys.md#rotate-keys` | A heading's id, and each link and include that names it, each retired permalink's `to` in `ascribe.toml` that names it, and each [memo file](../writing/memo-files.md)'s `about` that names it, in `memos/` and `.ascribe/memos/`. A heading with an `@id` has it changed. One whose id is made from its text gets an `@id` line under it, so its text stays, and links to the headings after it that were numbered after it follow. |
| `keys.md` | The page or fragment moves to the new path, and each link and include to it, or in it, whose path changes is changed. So is what `ascribe.toml` names it by: its path in `[glossary] pages`, a retired permalink's `to`, keeping its `#id`, and a phrase that's a link to it. So is each memo file's `about` that names it or a heading in it, keeping its `#id`, and its entry in its section's `pages` in [`ascribe.nav`](../navigation/index.md#the-file), which is taken out when it moves to another folder. A section written inline, as `sections = [{ … }]`, isn't followed: the file is left as it is, and `ascribe check` reports the name that's gone. The new path is read as the target was: from the current folder, or from the content root. A new path that changes only the file's case, such as `Keys.md`, renames it on a file system that matches names without case too, as macOS's and Windows' do. |

A path is from the current folder, or from the content root, and the project is found from it; for a phrase or a value, the project is found from the current folder. `--config` names the project for any target, so `ascribe rename keys.md guides/keys.md --config docs` works from the repository's root. A page that isn't found is named with both places it was looked for. It writes the files it changes, then moves the page, and prints each file it changed, from the current folder. A permalink's id isn't renamed, since links from outside the docs use it: change its `@id` by hand, then retire the old name with [`ascribe permalinks retire`](../permalinks/command.md#cli-permalinks-retire). A page isn't moved through a folder that's a symbolic link. A phrase used in the code of a source in another repository isn't renamed either, since that code is a copy.

| Code | Meaning |
|---|---|
| `0` | It renamed |
| `1` | What it names doesn't exist: a phrase, a value, a page, or a heading |
| `2` | It couldn't rename: no `ascribe.toml`, one with errors, a new name that can't be its name or is taken, a target it doesn't rename, or a file it can't write |

## `ascribe resolve`
@id {permalink=true}: cli-resolve
@summary: Writes a page as one build's readers see it, in the build's plain Markdown or its site output: its variants chosen, filtered blocks removed, and phrases and includes filled in. It writes nothing to disk.

Resolves a page for one build and writes it as the build's plain Markdown output by default, as `ascribe build --emit plain` writes it, or, with `--emit site`, as its site output: the page as a reader of that build sees it, with its variants chosen, the blocks the build filters out removed, phrases and includes filled in, and links written as that output writes them. It writes nothing to disk.

@include: ../_generated/cli-resolve-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/resolve.txt

A variant group the build keeps more than one arm of, which the site shows as tabs, is a line saying so, `A reader sees one of the next 3 sections at a time, chosen by operating system.`, then each arm under a heading that names its dimension and value, such as `### Operating system: macOS`, or its title for labeled arms, as the plain output writes it.

When the build doesn't publish the page, it writes nothing to standard output, says why on standard error, and exits with `1`. A fragment isn't resolved on its own; resolve a page that includes it.

A line of a [versioned build](../versions/index.md), such as `--build site@3.4`, resolves the page as that line publishes it, its route and links under the line's folder.

@available: ascribe 0.4.4
`--emit site` writes the page as the site output writes it, as `ascribe build --emit site` writes its file and `@ascribed/astro` receives it: Markdown with Ascribe's elements, such as `<ascribe-tabs>`, `<ascribe-note>`, and availability badges, from the code the build and the editor's preview use. It's what a site's integration is given, which the plain form can't show: what a switcher, a badge, or a glossary link becomes before a site draws it. `--emit plain`, the default, is the plain form above. `--emit` takes one output, not a list as `ascribe build --emit` does; a list is an error, exit `2`. From `examples/quill`, `ascribe resolve install-agent.md --build site --emit site` writes its Connect section as tabs, where the plain form writes a section for each arm:

@snippet {lang=markdown}: code:crates/ascribe-cli/tests/output/resolve-site-connect.txt

@available: ascribe 0.4.4
The site form's frontmatter, under `--frontmatter`, is the site output's, a line of a versioned build writes the line's links, and its elements read without their script as [a page in the site output](site-output.md) says.

`go:` and a permalink's name, in place of the page, shows the help a [product shows](../permalinks/in-your-product.md) for that permalink: its entry in the build's `_ascribe/help.json`, from the code that writes the file. The project is the current directory's, or `--config`'s. The text is the entry's title, its summary when the text doesn't start with it, its text, and "Read more" with its address, which needs [`[consumer] site`](../content-model/output.md#consumer):

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/resolve-permalink.txt

A name the build doesn't publish exits with `1`, and standard error names the closest it does publish; a retired permalink exits with `1`, and standard error gives its reason. An alias shows its target's entry.

`--emit`, either value, with `go:<name>` is an error, exit `2`: a permalink's help has one form, `help.json`'s.

| Code | Meaning |
|---|---|
| `0` | It wrote the page, or the permalink's help |
| `1` | The build doesn't publish the page, or the permalink; or the permalink is retired |
| `2` | It couldn't run: no `ascribe.toml`, one with errors, a path that isn't a page, more than one build and no `--build`, an unknown build, an `--emit` that isn't one output or is given with `go:<name>`, or a page that can't be written |

With `--format json`, the document is as follows, and its [schema](../contracts/json-reports/questions.md#ascribe-resolve) has every field:

@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/resolve.json

@available: ascribe 0.4.4
`emit` says which output `text` is written as: `plain`, or `site` under `--emit site`, where `text` is the site output's file, with its frontmatter under `--frontmatter`.

For `go:<name>`, the document is the entry as `help.json` writes it, with the file's `version`, `build`, and `line`, and the `name` asked for, after `schema_version` and `ascribe_version`. `version` is `help.json`'s format version, which a reader checks as it checks the file's. Its [schema](../contracts/json-reports/questions.md#ascribe-resolve-go) has every field:

@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/resolve-permalink.json

## `ascribe search`
@id {permalink=true}: cli-search
@summary: Finds where the pages say something, as their readers read it, and names the line to edit for each place, including words a phrase, a fragment, or a snippet puts there.

Finds where the pages say something, as their readers read it, and names the line to edit for each place. A text search reads the source, where a page says `Sign in to {cloud}`; `ascribe search` reads each page as every build publishes it, where it says "Sign in to Quill Cloud", and says which line wrote that, and through which phrase.

@include: ../_generated/cli-search-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/search.txt

Each place is `file:line:` and the block as a reader reads it, cut around the match, then where a reader meets it: the heading it's under and that heading's id, the arm of a variant group it's in, the phrases the match reads through, and, when it isn't in every build searched, the builds it's in. A line is listed once however many pages and builds show it. The places are in a fixed order: titles, then headings, then the rest; in each, the places written in pages come before those in fragments and code files, each in path order and in order within a file.

The query is one argument, and paths come after it. Words must all be in one block, in any order; a phrase in double quotes is matched as written, a run of spaces matching any run of white space. Quote several words together, and a phrase inside them: `ascribe search "rate limit"` finds both words in a block, and `ascribe search '"rate limit"'` finds them together. Case is ignored, and nothing else is: `cafe` doesn't find "café". With `--case`, case is matched as written, so `ascribe search Vale --case` doesn't find "vale". A word matches inside a longer one, so `sync` finds "synchronize"; with `--word`, a word or a phrase matches only where the characters on either side of it aren't letters, digits, or `_`, so `ascribe search limit --word` finds "the rate limit." and not "limits". A match never crosses from one block into another.

There are no regular expressions. To run a pattern over the text readers read, write the plain output with `ascribe build --emit plain` and use `grep` on it. That loses what `ascribe search` adds: `grep` names a line of the build's output, not the line of the source to edit, and it can't say that a line came from a fragment, a phrase, or a snippet, or list a line many pages show once; and it reads one build's output at a time.

Paths after the query, from the current folder, keep the search to the pages in those folders, or to those pages: `ascribe search sidebar docs/content/navigation/`. A fragment's text counts for the pages there that show it, and its place lists only them. `--type how-to` keeps to the pages of that content type. A path that has no page or fragment of the project is an error, and when every word after the query could be one of its words, the message says to quote them with it.

What's searched is each page's title and the field its type marks `role = "description"`, as each build publishes them, and what each build's plain output says of the page: its headings, paragraphs, lists, block quotes, tables (a row at a time), code blocks, and the titles and text of notes and details, wherever each is written: in the page, in a fragment an include brings in, or in the code file a `@snippet` shows. A match is a place in the source, listed once however many pages show it:

| The match is | The place is | Where a reader meets it says |
|---|---|---|
| Written in the page | The page's line | The heading and its id, the arm, the builds when not all |
| Written in a fragment | The fragment's line | `on 5 pages:` and the first three, each with its heading's id, then `ascribe refs` with the fragment for the rest |
| Partly a phrase's value, as "Sign in to Quill Cloud" where the page says `Sign in to {cloud}` | The line that uses the phrase | `through phrase cloud` |
| Wholly inside a phrase's value, as "Quill Cloud" | The phrase's line in `ascribe.toml` | How many places use it, and `ascribe refs phrase:cloud` |
| In a link with no text, which reads as its target's title | The link | `through upgrade.md's title` |
| In a snippet's code | The code file's line | `through code:examples/greet.ts#greet`, and the pages that show it, as for a fragment |
| In a page's title or description | The key's line in the page's frontmatter | The builds when not all |

A page that shows a place in fewer builds than the place is in names them after it, as `install.md #prerequisites (in build cloud)`. A code file outside the project's folder is named as the snippet's address names it, such as `code:greet.ts`. A snippet's code shown on more than three pages lists the first three and how many more. When the code a snippet shows can't be lined up with its file, as when a phrase's value has a line break, the place is the first line the snippet shows, and says `at the first line it shows`.

After a match wholly inside a phrase's value, the rest of the block is still searched: where `Sign in to {cloud}, then edit quill.yaml` is searched for `quill`, the phrase is one place and the line, at `quill.yaml`, is another.

`ascribe search "docs repository" --config examples/quill` finds a line of the page and a line of the fragment it includes:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/search-fragment.txt

`ascribe search '"upgrade the agent"' --config examples/quill` finds the page's title first, then the links that read as it:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/search-title.txt

What isn't searched: what no build publishes, the source's own spelling of a phrase, frontmatter fields other than the title and the description, the navigation's labels, text the content model writes (labels, availability lines, a widget's fallback), text in images, raw HTML, and what a site adds around a page.

`ascribe search key examples/quill/docs/keys.md --word`, from the repository's root, keeps to one page and to the whole word:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/search-scope.txt

With `--summary`, the places aren't listed: the pages that show them are, with how many each shows, most first, then in path order, and the total. A place a fragment puts on several pages counts for each. It lists 20 pages, or as many as `--limit` says, then how many more and the command that lists them all. `ascribe search agent --config examples/quill --summary`:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/search-summary.txt

When nothing matches, search looks for the query, by the same rules, in the source as written: every page's and fragment's text, and `ascribe.toml`'s, leaving out memos, which nothing reads. With paths or `--type`, it reads only the pages they keep to, the fragments those pages include, and `ascribe.toml`, so words other pages publish don't count. The source's text is read in blocks between blank lines, with its markup as written, so `{cloud}` is the phrase's key, not its value. The last line says which it is:

| The last line | When |
|---|---|
| "The words aren't in any page." | The source doesn't have them either. A word only in a memo is in no page. |
| "No build searched publishes them. They're written at `docs/x.md:12` and 2 more places." | The source has them, where no build searched publishes them, or where a phrase's value hides them. |
| "Builds not searched publish them: `site`, `self-managed-3.3`." | With `--build`, when a build that wasn't named has a place. |

`ascribe search "zebra" --config examples/quill`:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/search-none.txt

`ascribe search '"point the agent at your server"' --build cloud --config examples/quill`, words only the self-managed arm says:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/search-other-builds.txt

`ascribe search '"{cloud}"' --config examples/quill`, the source's spelling of a phrase:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/search-written.txt

| Code | Meaning |
|---|---|
| `0` | Something matched |
| `1` | Nothing matched |
| `2` | It couldn't run: no `ascribe.toml`, one with errors, an empty query, a quote that isn't closed, a path with no page or fragment of the project, or an unknown build or type |

With `--format json`, the document is a contract of its own, whose schema is in the [JSON report contract](../contracts/json-reports/questions.md#ascribe-search). `ascribe search '"sign in to quill cloud"' --config examples/quill --format json`:

@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/search.json

Each place has its `file` from the project root, its `line`, and the `column` of the match's first character, in Unicode characters. `in` is what it is: `title`, `description`, `heading`, `text`, `table`, `code`, or `phrase`, a phrase's value at its line in `ascribe.toml`. `text` is the whole block as a reader reads it, and `match` is where the match is in it: its first character and the one after its last, in characters. `through` lists what the match reads through that the line doesn't spell, in order: `{"phrase": key}`, `{"title": page}` for a link with no text, which reads as the title of the page it links to, or the text of the heading it names (`page.md#id`), and `{"code": address}` for a snippet's code. `arm` is the arm of a variant group the line is in, or null.

`pages` names the pages that show the place, at most three, each with its `title`, the `heading` the place is under there and its `id`, and the `builds` that show it there. `pages_total` is how many pages show it; for a phrase, how many places in the source use it. `refs_command` is the `ascribe refs` command that lists them: for a phrase, and for a fragment's line on more pages than `pages` names; otherwise it's null. A snippet's code that can't be lined up with its file is at the first line the snippet shows, and `end_line` is the last; every other place's `end_line` is null.

`total` is how many places match, `pages_matched` how many pages show one, and `shown` how many places are listed, 500 unless `--limit` says otherwise. When the list was cut, `truncated` is true and `next_command` is the command that lists them all. With `--summary`, `places` is empty and `pages` lists each page, from the project root, with how many `places` it shows, and `shown` and `truncated` count pages; without it, `pages` is null. When nothing matched, `source_places` is where the source writes the words, at most three places, each a `file` and `line`, when no build searched publishes them, and `not_searched_builds` is the builds that weren't searched that publish them; both are empty otherwise.
