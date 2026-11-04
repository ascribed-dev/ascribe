---
type: reference
title: "`ascribe explain`, `model`, `outline`, `resolve`, and `search`"
description: "Ask a project about its diagnostics, content model, and pages, see a page as a build's readers see it, and find where its pages say something."
---

`ascribe explain`, `model`, `outline`, `resolve`, and `search` answer questions about a project, for writers and for agents, without changing anything. [Links, references, and renames](references.md) has `ascribe link`, `ascribe refs`, and `ascribe rename`.

@include: ../_fragments/command-options.md

## `ascribe explain`
@id {permalink=true}: cli-explain

@summary
Says what a diagnostic means: its severity, its messages, how to fix it, a link to its entry in [Diagnostics](../checks/diagnostics.md), and, for the diagnostics people meet most, a short page that has the problem and the same page without it. It needs no project.

@include: ../_generated/cli-explain-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/explain.txt

The text output is for a terminal, so a link in the fix is written as its text, with the address after it when it leads off the docs; `--format json` keeps the Markdown, links and all. An example that needs something in `ascribe.toml` to go wrong shows that too: a few lines of the model it was checked against. Every example is checked, so each wrong page reports its diagnostic and nothing else, and each right page reports nothing.

| Code | Meaning |
|---|---|
| `0` | It explained the diagnostic, or listed them |
| `2` | No diagnostic has that code or name, and it names the closest; or a usage error |

With `--format json`, the document is as follows. Its [schema](../contracts/json-reports/questions.md#ascribe-explain) has every field, and so does the schema of what `--list` writes:

@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/explain.json

## `ascribe model`
@id {permalink=true}: cli-model

@summary
Shows the content model as `ascribe` reads it, with the defaults filled in: page types with their files and frontmatter fields, dimensions, phrases, features, glossary terms, project widgets, builds, and sources with their folders and the files they make readable.

A source with an `ascribe.examples.toml` also says how many of its files are [tested examples](../examples/tested.md), and names each observation with what it was tested with and how many of its files are as it found them. For a source in another repository, that's read from its copies, and an observation's files are counted only where there's a copy. It's what an agent, or a new writer, reads before writing frontmatter, a directive's attributes, or a phrase.

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

Every other table has a section too. `lifecycle` and `notes` have built-in entries, which the text without `--section` shows only when `ascribe.toml` writes them.

`--section navigation` shows the project's [navigation](../navigation/index.md), which the text without `--section` leaves out: a nested list in the sidebar's order, each section with its label and folder, then its pages, each with its label and its path from the content root. A section whose own page sits beside its folder is listed where that page is. In JSON, it's the `navigation` field alone.

The rest of `ascribe.toml` is there too, a section of settings for each of its tables: `project`, `versions`, `consumer`, `editor`, `checks`, `crawlers`, and `listing`. The text without `--section` names them at its end, and `--section` shows one, each key with the value in effect, and its default: `ascribe model --section consumer` shows `[consumer]`'s `site`, `base-path`, and the rest, the ones the file leaves out at their defaults. `[checks]` has the keys of `[checks.vale]` and `[checks.links]`, written `vale.preset` and `links.reuse-days`, and each check the file sets a level for. In JSON, each section is a list of keys, in the [schema](../contracts/json-reports/model.md#ascribe-model). The keys and defaults are the ones the loader reads, so `model` shows no key `ascribe check` would refuse.

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

`page` is the path from the content root, as links write it, and `file` is from the project's folder, as `ascribe check` reports it. `type` is null for a fragment. `explicit_id` is true when the heading has an `@id`, which stays when its text changes; prefer those in links. `fragment` is the fragment a heading comes from, or null.

`memos`, on the page and on each heading, lists the memo files about it, in path order: `file`, from the project's folder; `state`, `placed` or `outdated`; `quote`, or null; `first_line`, the first line of its text; and `personal`, which is false for the project's `memos/`.

## `ascribe resolve`
@id {permalink=true}: cli-resolve
@summary: Writes a page as one build's readers see it, in the build's plain Markdown or its site output: its variants chosen, filtered blocks removed, and phrases and includes filled in. It writes nothing to disk.

Resolves a page for one build and writes it as the build's plain Markdown output by default, as `ascribe build --emit plain` writes it, or, with `--emit site`, as its site output. That's the page as a reader of that build sees it: its variants chosen, the blocks the build filters out removed, phrases and includes filled in, and links written as that output writes them. It writes nothing to disk.

@include: ../_generated/cli-resolve-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/resolve.txt

A variant group the build keeps more than one arm of, which the site shows as tabs, is written as the plain Markdown output writes it: a line saying so, `A reader sees one of the next 3 sections at a time, chosen by operating system.`, then each arm under a heading that names its dimension and value, such as `### Operating system: macOS`, or its title for labeled arms.

When the build doesn't publish the page, it writes nothing to standard output, says why on standard error, and exits with `1`. A fragment isn't resolved on its own; resolve a page that includes it.

A line of a [versioned build](../versions/index.md), such as `--build site@3.4`, resolves the page as that line publishes it, its route and links under the line's folder.

`--emit site` writes the page as the site output writes it, as `ascribe build --emit site` writes its file and `@ascribed/astro` receives it: Markdown with Ascribe's elements, such as `<ascribe-tabs>`, `<ascribe-note>`, and availability badges, from the code the build and the editor's preview use. It's what a site's integration is given, which the plain form can't show: what a switcher, a badge, or a glossary link becomes before a site draws it. `--emit plain`, the default, is the plain form above. `--emit` takes one output, not a list as `ascribe build --emit` does; a list is an error, exit `2`. From `examples/quill`, `ascribe resolve install-agent.md --build site --emit site` writes its Connect section as tabs, where the plain form writes a section for each arm:

@snippet {lang=markdown}: code:crates/ascribe-cli/tests/output/resolve-site-connect.txt

The site form's frontmatter, under `--frontmatter`, is the site output's, a line of a versioned build writes the line's links, and its elements read without their script as [a page in the site output](site-output.md) says.

`go:` and a permalink's name, in place of the page, shows the help a [product shows](../permalinks/in-your-product.md) for that permalink: its entry in the build's `_ascribe/help.json`, from the code that writes the file. The project is the current folder's, or `--config`'s. The text is the entry's title, its summary when the text doesn't start with it, its text, and "Read more" with its address, which needs [`[consumer] site`](../content-model/output.md#consumer):

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

`emit` says which output `text` is written as: `plain`, or `site` under `--emit site`, where `text` is the site output's file, with its frontmatter under `--frontmatter`.

For `go:<name>`, the document is the entry as `help.json` writes it, with the file's `version`, `build`, and `line`, and the `name` asked for, after `schema_version` and `ascribe_version`. `version` is `help.json`'s format version, which a reader checks as it checks the file's. Its [schema](../contracts/json-reports/questions.md#ascribe-resolve-go) has every field:

@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/resolve-permalink.json

## `ascribe search`
@id {permalink=true}: cli-search

@summary
Finds where the pages say something, as their readers read it, and names the line to edit for each place.

A text search reads the source, where a page says `Sign in to {cloud}`; `ascribe search` reads each page as every build publishes it, where it says "Sign in to Quill Cloud", and says which line wrote that, and through which phrase.

@include: ../_generated/cli-search-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/search.txt

Each place is `file:line:` and the block as a reader reads it, cut to about 80 characters around the match's first word, then where a reader meets it. That's the heading it's under and that heading's id, the arm of a variant group it's in, the phrases the match reads through, and, when it isn't in every build searched, the builds it's in. A line is listed once however many pages and builds show it. The places are in a fixed order: titles, then headings, then the rest. In each, the places written in pages come before those in fragments and code files, each in path order and in order within a file.

The query is one argument, and paths come after it. Words must all be in one block, in any order, and a match never crosses from one block into another.

- **Phrases.** A phrase in double quotes is matched as written, a run of spaces matching any run of white space. Quote several words together, and a phrase inside them: `ascribe search "rate limit"` finds both words in a block, and `ascribe search '"rate limit"'` finds them together.
- **Common words.** Common English words outside quotes, such as `how`, `I`, and `the`, needn't be, unless they're all the query has: `how do I publish llms.txt` finds what `publish llms.txt` does.
- **Case.** Case is ignored, and nothing else is: `cafe` doesn't find "café". With `--case`, case is matched as written, so `ascribe search Vale --case` doesn't find "vale".
- **Whole words.** A word matches inside a longer one, so `sync` finds "synchronize"; with `--word`, a word or a phrase matches only where the characters on either side of it aren't letters, digits, or `_`, so `ascribe search limit --word` finds "the rate limit." and not "limits".

There are no regular expressions. To run a pattern over the text readers read, write the plain Markdown output with `ascribe build --emit plain` and use `grep` on it. That loses what `ascribe search` adds: `grep` names a line of the build's output, not the line of the source to edit; it can't say that a line came from a fragment, a phrase, or a snippet, or list a line many pages show once; and it reads one build's output at a time.

Paths after the query, from the current folder, keep the search to the pages in those folders, or to those pages: `ascribe search sidebar docs/content/navigation/`. A fragment's text counts for the pages there that show it, and its place lists only them. `--type how-to` keeps to the pages of that page type. A path that has no page or fragment of the project is an error, and when every word after the query could be one of its words, the message says to quote them with it.

What's searched is each page's title and the field its type marks `role = "description"`, as each build publishes them. So is what each build's plain output says of the page: its headings, paragraphs, lists, block quotes, tables (a row at a time), code blocks, and the titles and text of notes and details. Each counts wherever it's written: in the page, in a fragment an include brings in (generated ones under `_generated/` included, since readers see them), or in the code file a `@snippet` shows. A match is a place in the source, listed once however many pages show it:

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

What isn't searched:

- What no build publishes, and the source's own spelling of a phrase.
- Frontmatter fields other than the title and the description, and the navigation's labels.
- Text the content model writes: labels, availability lines, a widget's fallback.
- Text in images, raw HTML, and what a site adds around a page.

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

When no block has every word, search lists the blocks that have the most of them, at most 10, as partial matches, each with the words it lacks. They replace "The words aren't in any page.", and the exit code is still `1`. A partial match is a pointer: read it before linking to it, and search again with other words, such as `sign` for "signature", since a word's other forms don't match. `ascribe search "webhook signature" --config examples/quill`:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/search-partial.txt

| Code | Meaning |
|---|---|
| `0` | Something matched |
| `1` | Nothing matched, partial matches aside |
| `2` | It couldn't run: no `ascribe.toml`, one with errors, an empty query, a quote that isn't closed, a path with no page or fragment of the project, or an unknown build or type |

With `--format json`, the document is a contract of its own, whose schema is in the [JSON report contract](../contracts/json-reports/questions.md#ascribe-search). `ascribe search '"sign in to quill cloud"' --config examples/quill --format json`:

@snippet {lang=json, phrases=true}: code:crates/ascribe-cli/tests/output/search.json

Each place has its `file` from the project's folder, its `line`, and the `column` of the match's first character, in Unicode characters. A match of several words can span a whole code block, and `line` is the line its first word is on, within the block. The other fields:

- `in` is what it is: `title`, `description`, `heading`, `text`, `table`, `code`, or `phrase`, a phrase's value at its line in `ascribe.toml`.
- `text` is the block as a reader reads it: whole when it's at most 240 characters, and otherwise cut to about that many around the match's first character, at words, with `…` where it's cut, so a long code block or table row costs no more than a paragraph. `line` and `column` say where to read the rest.
- `match` is where the match is in `text`: its first character and the one after its last, in characters, ending where `text` is cut when the match runs past it.
- `through` lists what the match reads through that the line doesn't spell, in order: `{"phrase": key}`, `{"title": page}` for a link with no text, which reads as the title of the page it links to, or the text of the heading it names (`page.md#id`), and `{"code": address}` for a snippet's code.
- `arm` is the arm of a variant group the line is in, or null.

`pages` names the pages that show the place, at most three, each with its `title`, the `heading` the place is under there and its `id`, and the `builds` that show it there. `pages_total` is how many pages show it; for a phrase, how many places in the source use it. `refs_command` is the `ascribe refs` command that lists them: for a phrase, and for a fragment's line on more pages than `pages` names; otherwise it's null. A snippet's code that can't be lined up with its file is at the first line the snippet shows, and `end_line` is the last; every other place's `end_line` is null.

`total` is how many places match, `pages_matched` how many pages show one, and `shown` how many places are listed, 500 unless `--limit` says otherwise. When the list was cut, `truncated` is true and `next_command` is the command that lists them all. With `--summary`, `places` is empty and `pages` lists each page, from the project's folder, with how many `places` it shows, and `shown` and `truncated` count pages; without it, `pages` is null.

When nothing matched, `source_places` is where the source writes the words, at most three places, each a `file` and `line`, when no build searched publishes them, and `not_searched_builds` is the builds that weren't searched that publish them; both are empty otherwise. `stop_words` is the common words a place needn't have. `partial_places` is the partial matches, each a place with how many words it `has` and those it `lacks`; `places` and `total` count only full matches.
