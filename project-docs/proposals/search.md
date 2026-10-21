# Proposal: search

A proposal, not a plan. It says what the feature is and what it would look like, so the idea can be judged before anything is designed in detail. The plan for building it is [search/](../search/README.md). It comes from section 15 of the [brainstorm](../brainstorm.md), which also raised a cache and notes. Those are proposals of their own ([memos](https://github.com/ascribed-dev/ascribe/blob/395fe80d/project-docs/proposals/memos.md), and the cache's): the three share no layer, and this one mentions the others only where it touches them.

## The goal

An author or an agent can ask where a project's pages say something, and get the answer a reader would: the words as a build publishes them, each reported as the place in the source to edit.

A text search of the source can't answer it. In `examples/quill`, readers of `install-agent.md` are told to "Sign in to Quill Cloud". The source says `Sign in to {cloud}`, so `grep -ri "sign in to quill cloud" docs` finds nothing. A reader of the `site` build meets "Quill Cloud" seven times on that page, and the source's text has it once, in the frontmatter. The same goes for a fragment's text on the pages that include it, for the arm a build selects, and for what a build filters out: the source holds all of it or none of it, and a build's readers see something else.

The customer this matters most to is an agent. "Where do the docs already explain rate limits?" is the question to ask before writing a new page, and an agent that can't ask it writes the explanation a second time.

## What exists

**In Ascribe, for "where is X":**

| | Answers | Doesn't |
|---|---|---|
| `ascribe refs`, and the editor's **Find All References** and **Used by** | Where a named thing is used: a page, a heading, a fragment, a phrase, a feature, a term | Find words. It needs the name |
| `ascribe outline`, and the **Pages** view | A page's headings and ids; the project's pages by title | Look inside a page's text |
| `ascribe link` | Whether one target exists, and the closest ones that do | Search |
| `ascribe resolve` | One page as one build's readers see it, as plain Markdown | Cover more than a page, or say where a line was written |
| The **Content model** view | Every phrase, feature, and term, with a count of its uses | Find where a phrase's value is read |
| The editor's own search, `grep` | The source's text | See what Ascribe resolves |

Search adds the one thing none of them does: words, across every page, as resolved. It must not grow into a second `refs`. When a hit comes from a phrase or a fragment, search says so and names the `refs` command that lists the uses; it doesn't list them itself.

**On a site, for readers.** The docs site runs [Pagefind](https://pagefind.app) over its built HTML, and the index is served with the pages (`site/README.md`). That's search for a reader, in a browser, over what was published. This proposal is for the people and agents writing the pages, in the repository, before anything is published, and its answers are source lines. Ascribe isn't replacing a site's search, and builds no index for a site. The search inside a version's archive, in the [versions proposal](https://github.com/ascribed-dev/ascribe/blob/395fe80d/project-docs/proposals/versions.md), is a reader's search too.

**The pieces a search would stand on:**

- **A resolved page keeps where everything was written.** Each `ResolvedBlock` has its file and span, the includes it came through (`via`), the phrases substituted in it with their spans (`substitutions`), and for a `@snippet`, the code file and its lines.
- **`ascribe diff` already turns that into text.** For each block of a resolved page it builds the text "as a reader reads it, whitespace collapsed", with an anchor of the source file, its lines, and the includes it came through (`crates/ascribe-diff/src/tree.rs`). It's private to that crate, and exact to the block, not the character.
- **The Vale work already maps characters back.** `crates/ascribe-check/src/prose/extract.rs` writes a page's prose and keeps a list of pieces, each a range of the prose and the source span it stands for, marked as copied, made, or a phrase's value. `Prose::locate` turns a range of the prose into a source span and the phrase it's in. A property test holds every character to a place.

## Whether it needs an index

It doesn't. The brainstorm asked whether search would add a database to the binary. Measured, the scan is the cheap part by two orders of magnitude, and what costs time is resolving the project, which Ascribe already does quickly.

On an Apple M1 Pro, with a release build of `main` at `460259f6`, median wall time. The second project is the synthetic one with prose, lists, tables, and code blocks appended to each page until the source reached 21 MB, the size of the converted Elastic sample in [`tests/corpora/RESULTS.md`](../../tests/corpora/RESULTS.md):

| | 3,000 pages, 0.8 MB resolved | 3,000 pages, 21 MB |
|---|--:|--:|
| `ascribe refs phrase:product`: load and index, resolving nothing | 415 ms | 717 ms |
| `ascribe check`: every page resolved and checked | 533 ms | 1.53 s |
| `ascribe build --emit plain`: checked, and 3,000 files written | 1.02 s | 2.85 s |
| A literal scan of the plain output, held in memory | 0.5 ms | 15 ms |
| The same ignoring case, as a regular expression | 3.9 ms | 111 ms |
| `grep -rn` over the plain output on disk | 109 ms | 323 ms |

A laptop's numbers show proportion and aren't a budget; decision 41 wants one recorded on the runner before anything is tuned.

What they imply:

- **On the command line, a query resolves from cold and costs about what `ascribe check` does:** more than the first row, since every page is resolved, and less than the third, since nothing is written. Half a second at 3,000 small pages and about a second and a half at 21 MB. That's fine for a command a person or an agent runs now and then.
- **The MCP server keeps a project loaded and indexed between calls** (`crates/ascribe-cli/src/mcp/cache.rs`), and keeps each check's result. It would keep each build's searchable text the same way, until the listing of the project's files changes. An agent's first query pays for resolving; each one after is the scan.
- **The language server holds the snapshot and a cache of resolved pages per build.** It builds the text once per snapshot, and a query as you type is then the scan alone: 15 ms for the larger project, when case is handled by scanning a lowercased copy and not by a pattern.
- **Keeping the text costs memory about equal to the plain output:** 0.8 MB and 21 MB for these two, per build, against the 185 MB the language server holds for the first.

So the first version stores nothing and adds no dependency. [When an index would be worth it](#when-an-index-would-be-worth-it) says what would change that.

## A query

The command's name is `ascribe search`. The [surface audit](../surface-audit/findings.md#the-proposals-against-the-findings) checked it against the conventions for naming a command, and it stands.

```sh
ascribe search rate limit                 # both words, in one block
ascribe search '"rate limit"'             # the phrase
ascribe search token guides/ --build cloud
```

- **Words must all be in the same block:** a paragraph, a heading, a list item, a table row, a code block. Any order.
- **Quotes make a phrase,** matched as written. A run of spaces in the query matches any run of white space, so a phrase is found across a wrapped line.
- **It matches text, not markup.** `Settings → Keys` is found whether or not the source makes it bold, and a link is its text, not its address.
- **Case is ignored** unless `--case` is given. Nothing else is folded.
- **A word matches inside a longer one,** so `limit` finds "limits" and "unlimited". `--word` keeps to whole words.
- **A match never crosses blocks.**

Scope:

| Option | Narrows to |
|---|---|
| `--build <NAME>`, repeatable | Those builds. By default every build, as `check` does |
| Paths after the query | Pages in those folders, or those pages. A fragment's text counts for the pages there that show it |
| `--type <NAME>` | Pages of that content type |

Deliberately absent: ranking, stemming, synonyms, fuzzy matching, and anything that needs a model. The binary calls none, so there are no embeddings and no search by meaning. An agent supplies the meaning itself: it tries "rate limit", then "throttle", then "429", and each try costs milliseconds.

**The order is fixed and simple:** pages whose title matches, then headings, then everything else, each in path order and in order within a file. That's the order `refs` uses, with the two places a subject is most likely named put first.

## What's searched

Everything a build's plain output says, plus two frontmatter fields:

| Searched | Reported at |
|---|---|
| Headings, paragraphs, list items, block quotes, notes and other directives' text and titles | The line it's written on |
| Table cells | The row's line |
| Link text, and an image's alt text | Its line |
| A phrase's value | See [One hit, several places](#one-hit-several-places) |
| A fragment's text, on each page that includes it | The line in the fragment |
| Code blocks | The line in the page |
| A `@snippet`'s code | The code file, at the region's lines, with the `@snippet` line it's shown through |
| The page's title, and the field its type marks `role = "description"` | The key's line in the frontmatter |

A page is searched once per build, as that build resolves it: the arms it selects, without what its availability filter removes, and not at all when the build doesn't publish the page.

What it can't see, which the reference would say in as many words:

- **Content no build publishes.** A page, an arm, or a section every build drops has no reader, so it has no hits. With `--build`, neither does what that build drops.
- **The source's own spelling.** `{product}`, an `@include` line's path, an attribute, a link's address. A search for the literal `{product}` is what a text search is for, and a search for its uses is `ascribe refs phrase:product`.
- **Text Ascribe generates from the content model:** an availability badge's label, an arm's label. Each is one line of `ascribe.toml`.
- **Text inside images,** and raw HTML, which the plain output leaves out.
- **Whatever the site adds:** navigation, a sidebar, a footer.

## A result

One line per place in the source, as `file:line:`, then the block as a reader sees it, cut around the match:

```console
$ ascribe search '"sign in to quill cloud"'
docs/install-agent.md:61: Sign in to Quill Cloud and copy an API key from Settings → Keys, then add it to…
    Connect to Quill #connect · deployment=cloud arm · through phrase cloud

1 place matches, on 1 page.
```

The second line says where a reader meets it and how it got there, and has only the parts that apply:

- **The nearest heading and its id,** so an agent can link to the section with `install-agent.md#connect`, and the page's title when the hit is in a fragment.
- **The arm,** when the text is in a variant: `deployment=cloud arm`.
- **How it got there:** `through phrase cloud`, `through _fragments/prerequisites.md`, `through code:examples/connect.ts#connect`. The words are `ascribe diff`'s, which already says a page changed "(through _fragments/prereqs.md)".
- **The builds,** when it isn't in all of those searched: `builds site, cloud`. This line is in all three of the example's, so it names none.

### One hit, several places

A fragment's line appears on every page that includes it, and a phrase's value wherever the phrase is used. A result is a place in the source, so each of those is one result however many pages show it:

```console
$ ascribe search "docs repository"
docs/install-agent.md:7: The Quill agent watches your docs repository and syncs changes to Quill. This page…
docs/_fragments/prerequisites.md:4: A docs repository that Quill can read.
    on 1 page: install-agent.md #prerequisites

2 places match, on 1 page.
```

| The match is | One result at | It says |
|---|---|---|
| Written in the page | The page's line | The heading, the arm, the builds |
| Written in a fragment | The fragment's line | How many pages show it, the first three, and `ascribe refs <fragment>` for the rest |
| Partly a phrase's value, as "Sign in to Quill Cloud" | The line that uses the phrase | `through phrase cloud`, only when the match touches the value |
| Wholly inside a phrase's value, as "Quill Cloud" | The phrase's line in `ascribe.toml` | How many places use it, and `ascribe refs phrase:cloud` |
| In a snippet's code | The code file's lines | The pages that show it, as for a fragment |
| The same line in several builds | Once | The builds, when not all |

The last row but one is where the forty copies would have come from. A search for a product's name gives one line, at the place to change it.

The count of pages is of pages a reader can open, so the summary of a search that hit one fragment reads "1 place matches, on 30 pages."

### For agents

The text above is the compact form: it follows `check --format concise` and `refs`. It lists at most 50 places, then says how many it left out and the command that gives the rest, as decision 51 asks of any cut list:

```text
412 places match, on 96 pages.
Showing 50 of them; for all: `ascribe search token --limit 412`
```

`--summary` replaces the list with the pages and how many places each has, most first, as `check --summary` does. For a common word that's the better first answer.

With `--format json`, a document of its own under decision 52's rules: generated schema in `schemas/`, `schema_version` 1, snake_case keys.

```json
{
  "schema_version": 1,
  "ascribe_version": "0.3.0",
  "query": "\"sign in to quill cloud\"",
  "builds": ["site", "cloud", "self-managed-3.3"],
  "places": [
    {
      "file": "docs/install-agent.md",
      "line": 61,
      "column": 1,
      "in": "text",
      "text": "Sign in to Quill Cloud and copy an API key from Settings → Keys, then add it to quill.yaml:",
      "match": [0, 22],
      "through": [{ "phrase": "cloud" }],
      "arm": "deployment=cloud",
      "pages": [
        { "page": "install-agent.md", "title": "Install the Quill agent", "heading": "Connect to Quill", "id": "connect", "builds": ["site", "cloud", "self-managed-3.3"] }
      ],
      "pages_total": 1,
      "refs_command": null
    }
  ],
  "pages_matched": 1,
  "total": 1,
  "shown": 1,
  "truncated": false,
  "next_command": null
}
```

`in` is `title`, `description`, `heading`, `text`, `table`, or `code`, so a caller that wants only headings filters for itself. `pages` is cut at three, with `pages_total` and `refs_command` saying how to get the rest.

Exit codes follow decision 52: `0` when something matched, `1` when nothing did, `2` when it couldn't run.

**The MCP tool** is `ascribe_search`, with `query`, `path` (a file or folder, which finds the project and narrows the search), `build`, `limit`, and `response_format`, concise by default, as `ascribe_refs` has. It returns what the command writes, as every tool does. The tools' names are a contract (decision 56), so adding one is a recorded decision.

**Agents have to be told.** The rules `ascribe agents sync` writes, the skill, and the `new-page` prompt each gain a line: search for the subject before writing a page about it.

## In the editor

One command, **Ascribe: Search Pages as Readers See Them**, in the Command Palette. The actions bar holds only what applies at the cursor and has no searches of its own (the editor guide says so), and this doesn't change that.

It opens a quick pick. Results arrive as you type, each showing the reader's text with the match marked, and below it the file, the heading, and how it got there. Choosing one opens the source at that line. A hit inside a phrase's value opens the phrase in `ascribe.toml`.

It searches the build you're looking at, the one the status bar names and the preview renders, so the three agree. A button in the quick pick switches to every build.

The language server answers a new request, in the pattern of `ascribe/targets` and `ascribe/inventory`:

```text
ascribe/search
  params: { textDocument, query, build?, limit? }
  result: { places: [{ uri, range, in, text, match, through, arm, pages, pagesTotal }], total, truncated }
```

`textDocument` is any file of the project, which is how the server knows which project is meant. It answers from its current snapshot, so unsaved edits are searched as they stand, and it's the only surface where they are: the command and the MCP tool read the disk. Its result type goes through `crates/ascribe-cli/src/shapes.rs` to the extension's generated types, as the other requests' do.

## The seam: from a reader's text to a source line

Everything above depends on one thing being right: a match found in resolved text names the line a person would edit.

What's already there covers less than it looks:

| | The Vale map (`prose/extract.rs`) | `diff`'s block text | Search needs |
|---|---|---|---|
| Works on | One source file, as parsed | A resolved page, for a build | A resolved page, for a build |
| Variants | Every arm | The build's | The build's |
| Includes | Left as a directive | Expanded, with the path through them | Expanded, with the path through them |
| Phrases | Replaced, and the key remembered | Replaced | Replaced, and the key remembered |
| Markup | Kept, for Vale to read | Removed | Removed |
| Code blocks, snippets | Left out | Present | Present |
| Frontmatter | Left out | The title, to compare | The title and the description, with their lines |
| Exact to | The character | The block's first line | The line |

So search reuses the Vale map's method and not its code. The method is the list of pieces, each a range of the output with the span it stands for and whether it was copied, made, or a phrase's value, and the property test that every character has a place. It carries over because a resolved page's leaves are the parsed blocks themselves, with spans in the file each was written in, so the same walk can run over a resolved page and add the file and the includes to each piece.

What's new, and each is a place the map could be wrong:

- **A match that starts in copied text and ends in a phrase's value,** or covers two phrases.
- **A hit in a fragment shown on a page in one build and filtered out of it in another.**
- **A snippet's code,** whose lines aren't the code file's lines one for one when tags are taken out of a region. The result gives the region's lines when it can't give the line.
- **A link with no text,** which a reader sees as its target's title. The hit is at the link, through the other page's title.
- **A table row,** where the text is cells and the source is one line.
- **The title,** which comes from YAML, not from a block.

The text search matches is the third reading of "what a reader sees" in the code, after the plain emitter and `diff`'s tree. It gets a test that holds it to the plain output, in the pattern of the repository's other two-implementation tests: every block's searchable text is in what `ascribe resolve` writes for that page. The text and the matching live in a crate below both `ascribe-query` and `ascribe-lsp`, since the language server doesn't depend on `ascribe-query`.

## When an index would be worth it

Not at any size measured here. An index speeds up the scan, and the scan is 15 ms of a second and a half. What an index can't skip is knowing whether the project changed, which means reading every source.

If a cold query on the command line ever becomes too slow, it will be because resolving is, and `check` will be too slow with it. The cure is then the one [`RESULTS.md`](../../tests/corpora/RESULTS.md#one-file) already names for single-file checks: don't index and resolve the project again on every run. That would be a build's searchable text kept as a plain file under `.ascribe/`, keyed by a hash of the sources and the content model, in the store the cache proposal describes and under its rule that deleting it changes nothing but time. Still no database.

A real inverted index, and the engine that comes with it, would be worth weighing only if the scan itself became the cost: hundreds of megabytes of resolved text, or ranking. Neither is in sight, and neither is proposed.

## What this leaves out

- **Search for readers,** and anything a site serves. That's Pagefind's job, or the site's choice.
- **Search by meaning,** ranking, stemming, and fuzzy matching.
- **Replace.** Search finds the line; the edit is the author's or the agent's. Changing a phrase's value is already one line, and renaming is the editor's.
- **Searching a git revision,** or what changed. `ascribe diff` answers "what changed for readers".
- **A persistent index,** as above.
- **A search view in the sidebar.** The quick pick first; a view if people keep results open. Microsoft Learn's authoring pack and Dendron's Lookup each put their finding and inserting in one quick pick, and VS Code's UX guidelines keep the sidebar for what a command can't do.

Later: [memos](https://github.com/ascribed-dev/ascribe/blob/395fe80d/project-docs/proposals/memos.md), once they exist, could be searched alongside pages.

## The thread

For a later plan, by the repository's [steel threads](../../.claude/skills/steel-threads/SKILL.md) skill. This isn't the plan.

- **The layers:** the resolved page (`ascribe-resolve`, read and not changed); the searchable text with its map, and the matching, in a crate both callers reach; `ascribe-query` for the command's entry; `ascribe-cli` for the command, its JSON shape, and the MCP tool; `ascribe-lsp` for `ascribe/search`; `packages/vscode` for the quick pick; the docs, the skill, and the agents' rules.
- **The riskiest seam** is the map from a match in resolved text back to a source line. If it can't be made exact through phrases and includes, results fall back to a block's first line, and the result's shape changes: a contract that shouldn't ship before that's known.
- **The thinnest use case:** in `examples/quill`, `ascribe search '"sign in to quill cloud"'` prints `docs/install-agent.md:61`, says `through phrase cloud`, and exits with `0`, where `grep` finds nothing. One build, one page, text output only. It crosses resolve, the map, the query, and the command, and goes straight through the seam.
- **Left for the widening, in order of risk:** fragments and their grouping; snippets; titles and descriptions; every build, and merging results across them; the JSON and the MCP tool; the editor.

## Waiting on the audit

**What the command is called.** `ascribe search`, with the tool `ascribe_search` and the request `ascribe/search`. The [surface audit](../surface-audit/findings.md#the-proposals-against-the-findings) checked the names against its conventions: a verb alone, and a tool named for its command. A path given to it is a path from the current folder, as the audit's findings 13 and 26 ask of every command.

## Settled

Four questions this proposal left open, answered by the maintainer on 11 October 2026.

- **Search gets its own map, and `diff` moves onto it; Vale keeps its own.** The map is built by the Vale map's method, as the seam section says, with the property test that every character has a place, and held to the plain output by test. `ascribe diff` reads the same input, a resolved page for a build, and needs less of it, so once search's map exists `diff`'s block text is drawn from it, as the plan's last widening and under the clean-up rule: `node scripts/compare/outputs.ts` shows the diff's text and JSON unchanged, or the move waits. Vale's map stays where it is: it reads one source file as parsed, with markup kept and every arm present, which is what prose checks need, and moving it onto a build's resolved text would change which text Vale sees and report each shared line once per build. So two of the three readings become one, and the third differs on purpose and is said to. _Why:_ a map shared by two readers that want the same thing removes a way for them to disagree; one shared by a third that wants something else adds a flag to it.
- **No regular expressions.** Words, a phrase, `--case`, and `--word` are a substring search, and the binary links no engine today: the only crate that pulls one in is `proptest`, for tests. A pattern is the one query that needs an engine, and an agent, the user this is for, gets what a pattern gives by trying words in turn at milliseconds each. The way to run a real pattern over resolved text stays `ascribe build --emit plain` and `grep`, and the reference says so, with what it loses. If a second use for an engine arrives, `--regex` is additive and the option's name is reserved for it. _Why:_ a dependency is paid for by every build of the binary, and this one would serve a query the audit's sessions never made.
- **The title and the field with `role = "description"`, and no more.** They're the two fields a build itself publishes: in the site output's frontmatter, `llms.txt`, and the page record the [search and answers plan](https://github.com/ascribed-dev/ascribe/blob/5b22286fd2f370e796219e68a7c9d7cdb2ba8acc/project-docs/search-and-answers/README.md) writes. A field a site shows on its own is the site's business, as the site's navigation and footer are, and search says in its reference that it sees what a build publishes. No new key on a type: if a field ever needs to be marked as shown to readers, that's a fourth role, added when a second consumer of it exists, since a role changes nothing in the outputs today and this would. _Why:_ what a reader sees is what Ascribe writes, and search's claim stops where Ascribe's does.
- **When nothing matches, search says whether the source has the words as written.** The source index already holds every file's text, so a substring scan of it costs milliseconds and resolves nothing. The summary then reads one of three ways: the words are in no page; they're in the source at `file:line` and no searched build publishes them; or, with `--build`, they're published by builds the query didn't name, which are listed. A phrase's value isn't the source's text, so the scan can miss what resolution made, and the sentence says "as written". The exit code stays `1`, and the JSON carries `source_places`, cut at three, beside the empty `places`. _Why:_ "not written" and "not published" are different answers, an agent acts differently on each, and the second scan is the cheap part of a query that already resolved every page.
