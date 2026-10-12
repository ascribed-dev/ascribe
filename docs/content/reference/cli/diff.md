---
title: "`ascribe diff`, `drift`, and `report`"
description: "Compare a project's output with another revision, find sources that drifted, and report on a project's health."
---

The commands that compare a project with another revision of itself, and report on it. The options every command accepts are in the [command reference](../cli.md#options-every-command-accepts).

## `ascribe diff`
@id {permalink=true}: cli-diff

Shows what changed between a git revision and the working tree, as readers will see it: which pages of each [build](../content-model/output.md#16-buildsname) changed, and which blocks on them were added, removed, changed, or moved. It compares **resolved pages**, not files, so a page whose own file didn't change but whose included fragment, [phrase](../directives.md#phrases), or build settings did is listed, with what its change comes from. A change that doesn't reach the page (`ascribe fmt`, rewrapped lines) isn't.

@include: ../../_generated/cli-diff-options.md

The other side is the working tree: the files on disk, committed or not, as `ascribe build` would read them now. Unsaved editor changes aren't included. The base is read from git, `ascribe.toml` included, so a change to the content model is compared too; if the project didn't exist at the base, every page is added. `ascribe diff` needs `git` on the path, and nothing else: no network and no GitHub account. It runs `git` so that the repository can't make it run a command of its own: with no file system monitor, and taking a bare repository only when one is named, never a folder in the project shaped like one, where `git` says it can't use the repository.

The text output lists each build's changed pages with counts, and where a change comes from when it isn't only the page's own file:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/diff.txt

@available: ascribe 0.3.0
Name a page to see its changes: every changed block, as it was (`-`) and as it is (`+`), with where it's written. In changed prose, the words removed are marked `[-like this-]` and the words added `{+like this+}`, as `git diff --word-diff` marks them; a block that isn't prose, such as a code block, shows its lines on both sides, and a moved block shows its lines once. A build the page didn't change in says so. Name a fragment, or another file pages take text from, to see the pages that changed through it. `--format json` and `--format html` are narrowed the same way, with the same shape and fewer pages, and `--exit-code` exits with `1` only when what's shown changed. A PAGE that isn't a file of the project, at the base or now, is a usage error.

@available: ascribe 0.3.0
@snippet {lang=text}: code:crates/ascribe-cli/tests/output/diff-page.txt

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

**Copy prompt** copies the page's [agent prompt](#the-agent-prompt), the one `ascribe diff --format prompt <page>` writes, to paste into your agent. The report has no link that opens an agent: a file opened from a disk or a CI artifact can't tell which editor you use, so it only copies.

A report renders at most 300 changed pages; the rest are listed by name, and the report says so at the top. Each rendered page and image is stored once, however many builds or pages share it. [The report in CI](../../guides/review.md#the-report-in-ci) has a GitHub Actions job that uploads the report on every pull request.

### The agent prompt

`--format prompt` writes a prompt that asks your agent to review the change as readers will see it, in the [agent prompt format](agents.md#what-a-prompt-says), and writes nothing when nothing changed. Without a PAGE, it's about every changed page, grouped by build when more than one changed. With a PAGE, it's about that page in the first build it changed in, listing each change with its source lines; for a fragment, or any other file pages changed through, it's about the pages that show it, and asks the agent to check that the new text fits each. A PAGE that isn't a file of the project, at the base or now, is a usage error.

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/diff-prompt.txt

The prompt asks for a report, not edits: a review reads the change, and its author decides what to do. Its paths and commands (`ascribe render`, `git diff`, `ascribe diff`) are written from the folder `ascribe diff` was run in, which it names once, with the base, the build, and `--config` as this run had them. The HTML report's **Copy prompt** gives the same prompt, and the editor's **Prompt agent** gives it from the repository's root.

### Diff JSON

`--format json` writes one JSON document to standard output. It follows the [JSON output](check.md#json-output)'s rules: `schema_version` changes only when a field is removed or changes meaning, so **ignore fields you don't know**.

| Field | Type | Meaning |
|---|---|---|
| `schema_version` | number | `1` |
| `ascribe_version` | string | The version of `ascribe` that wrote the report |
| `base` | object | `requested`: the revision asked for, or the default branch used. `commit`: the commit it names. `merge_base`: the merge base with `HEAD` that was compared with, or `null` with `--base-exact`. |
| `repository` | object | `root`: the repository's top-level directory. `project_prefix`: the project's folder in it, with a trailing `/`, or `""` at the root. |
| `working_tree_errors` | number | How many errors `ascribe check` finds in the working tree for the builds compared (with the same `--build` options). The comparison runs either way. |
| `blocks_omitted` | boolean | Only with `--pages-only`, and then `true`: each page's `changes` is empty, and its `counts` still count them. |
| `builds` | array | One entry per build compared: `build`, its name, and `pages`, the pages that changed, in path order |

Each page:

| Field | Type | Meaning |
|---|---|---|
| `path` | string | The page's path, relative to the content root |
| `route` | string | Its route; for a removed page, the route it had |
| `status` | string | `"added"` (the build publishes it now and didn't), `"removed"`, or `"changed"` |
| `own_file_changed` | boolean | Whether the page's own file changed, or exists on one side only |
| `because` | array of strings | The other changed files its change comes from, relative to the content root: fragments it includes, and pages its links take their text or a heading id from (a link to a page that changed otherwise isn't a cause). Then the [snippets](../directives.md#snippet) whose code changed, by address (`code:service/client.py#connect`). `ascribe.toml` comes last when a change to the content model (a phrase's value, a label, a build's settings) is a cause. |
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
@id {permalink=true}: cli-drift
@available: ascribe 0.2.0

Lists the pages whose code examples changed between a git revision and the working tree, and whether the words around them changed too, and the pages whose examples no longer resolve. A page covers the code it shows, and nothing else: every region it takes a [snippet](../directives.md#snippet) from, and every whole file it takes as one, including its fragments' snippets. It's for CI, where it says which pages a reviewer should reread; see [Drift](../../guides/drift.md#when-an-example-changes).

@include: ../../_generated/cli-drift-options.md

The base is found as `ascribe diff` finds it, and the other side is the working tree. A snippet's region is taken from its file on both sides and compared as text, dedented, with trailing spaces trimmed, so an edit elsewhere in the file, a region that only moved, or a change of indent isn't a change. A renamed file is followed. A region the file didn't have at the base is a new example, not a changed one. Then, for each page with a changed example, it compares the page itself as `ascribe diff` does, apart from its snippets: the page changed when its own file did, or when anything else it uses did, such as a fragment it includes or a phrase in `ascribe.toml`.

The pages come in up to three groups:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/drift.txt

An example **no longer resolves** when it did at the base and doesn't now: its region was renamed, its file moved, or its source is gone. `ascribe check` fails on it too, but the report names the page so the pull request that broke it says so. A `@snippet` the change adds broken isn't listed; that's `check`'s alone. Of the changed examples, the first group is the one to read: the example updated itself, and nobody changed the sentence that explains it. The second is listed so a reviewer can see the change reached the page. Each example says how many lines were added and removed. With nothing to report, it says `No examples changed.`

A changed example isn't always a changed behavior. The report says what changed and how much, and leaves the judgment to you.

It reads little: one `git diff` listing of the change, and at the base only the files that listing names that a snippet uses. The base's pages are read only when an example changed. `ascribe drift` needs `git` on the path, and nothing else: no network and no GitHub account.

### The summary

`--format markdown` writes Markdown for a CI job's summary: the same groups, each page linked to its route on `[consumer] site` (plain text when there's no `site`), and nothing at all when no example changed, so it can be appended to `$GITHUB_STEP_SUMMARY` on every run.

### Exit codes
@id: drift-exit-codes

| Code | Meaning |
|---|---|
| `0` | Compared, whatever it found |
| `1` | With `--exit-code`: an example no longer resolves, or a page's example changed and the page didn't |
| `2` | It couldn't run: a usage error, an unknown build, not a git repository, an unknown revision, no merge base (a shallow clone, or unrelated histories), `git` not found, or a project that doesn't load at the base or in the working tree. The reason goes to standard error. |

### Drift JSON

`--format json` writes one JSON document to standard output, with the [JSON output](check.md#json-output)'s rules: `schema_version` changes only when a field is removed or changes meaning, so **ignore fields you don't know**.

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

## `ascribe report`
@id {permalink=true}: cli-report
@available: ascribe 0.3.0

Reports what state a project is in, section by section: what's too slow, needs the network, or is too broad for `ascribe check`. Every problem in it carries its kind of next step, as in the editor. It's for a scheduled job, or for a look at the whole project; [Reading the report](../../guides/report.md) says how to act on each section.

@include: ../../_generated/cli-report-options.md

| Section | What it reports | Needs |
|---|---|---|
| `problems` | What `ascribe check` finds, counted by check, by file, and by kind of next step; and every problem acknowledged as intended, with its reason | Nothing |
| `inventory` | The pages by content type and by owner; the overdue reviews, the pages nothing links to, and the entries nothing uses | Nothing |
| `builds` | For each build, the pages and the content it leaves out that another build keeps, and why | Nothing |
| `pages` | Every page's record, by build and sorted by address: its title, description, canonical address, and listing, and where the listing comes from | Nothing |
| `links` | The external links that fail, time out, or moved permanently, each at its place in the source | [lychee](https://lychee.cli.rs), installed, and the network |
| `agents` | The [delivery spec](https://agentdocsspec.com/spec/web/)'s checks on a built site | `afdocs`, installed (`npm install -g afdocs@0.22.2`); the network; `--site` |

With no section named, it runs the first three; `pages` lists every page rather than saying what state the project is in, so it runs when it's named. Each section is independent: one that can't run, because its program isn't installed or `--site` is missing, says why and what would let it, and the others still run. `ascribe report` makes no request itself: `links` and `agents` run the programs the project installs, which do.

**`pages`** lists, for each build, the record of every page it publishes: what every reader that isn't a person is told about the page, as the site output's [`_ascribe/pages.json`](../../contracts/output-layout.md#13-the-page-records) holds it, read from the build's resolved pages, so nothing is built and nothing is fetched. As text, a table of each page's address, its listing, where that comes from (`default` when nothing sets it, `page` for its own `listing`, or a rule of [`[listing]`](../content-model/checks.md#22-listing) and its line in `ascribe.toml`), its title, and its description, cut at `--limit`, with what's left out counted; the line above it gives the site's address, how many pages the build publishes, and how many of them aren't `listed`. `--not-listed` lists only those, `unquoted` and `unlisted`: what a site keeps out of search, and why. As JSON, each build's `records`, sorted by `route`, each with the fields of its entry in `_ascribe/pages.json`, by the same names, and `route`, the key it's under there.

**`links`** lists every `http` and `https` link of the project's sources, hands the addresses to lychee once each, and reports each link it says is broken (`link-external-broken`, to review) or moved permanently with a `301` or `308` (`link-external-moved`, with a fix that writes the new address) at the link. A site's root that redirects to another of its own pages, a landing page, hasn't moved. An address that answered within `[checks.links] reuse-days`, 14 by default, with nothing to report, isn't asked about again; the section says how many it asked about and how many answers it reused, and how old the oldest is. A link that moved or failed is asked about on every run, so every problem was found in this run. `--recheck` asks about every address. A link that answered on an earlier run and fails now, other than with a 404 or a 410, is listed as unconfirmed, not reported, until it has kept failing for `[checks.links] confirm-days`, 3 by default; it isn't counted, doesn't fail `--exit-code`, and is left out of `--format prompt`. The section says how many of its problems are new since the last run, and how many that run reported are gone. `--no-history` reports every failure, as a first run does. `[checks.links]` in `ascribe.toml` names the command, and the hosts to leave out; a link that's fine as it is, on a site that turns checkers away, is [acknowledged](../directives.md#intended) with `@intended` above its block.

**`agents`** runs the delivery spec's checker on `--site` and reports each check that fails or warns by who changes what it checks: the hosting (`delivery-hosting`, with the setting to change), or what Ascribe writes for agents (`delivery-output`, a bug to report). A check about the pages, such as their size, is `ascribe check`'s to report, so it's named and not repeated. Nothing here asks you to edit a page.

### The summary and the prompt

`--format markdown` writes the same sections as Markdown, for a CI job's summary or an issue's body. `--format prompt` writes one prompt for an agent about the problems of one section, `problems`, `links`, or `agents`, with what each kind carries (the sentence around a link, the setting to change) and the command that checks the work; nothing when the section finds nothing. Each list holds at most `--limit` items; what's left out is counted, with the command that lists it.

### Exit codes
@id: report-exit-codes

| Code | Meaning |
|---|---|
| `0` | It ran, whatever it found |
| `1` | With `--exit-code`: a section found something at the level given, or above (`advice` when none is: any problem) |
| `2` | With `--exit-code`: a section asked for couldn't run. Without it, too: a usage error, an unknown build, or a project that doesn't load. The reason goes to standard error. |

### Report JSON

`--format json` writes one JSON document to standard output, with the [JSON output](check.md#json-output)'s rules: `schema_version` changes only when a field is removed or changes meaning, so **ignore fields you don't know**. Its schema is in the [JSON report contract](../../contracts/json-reports/report.md#ascribe-report).

| Field | Type | Meaning |
|---|---|---|
| `schema_version` | number | `1` |
| `ascribe_version` | string | The version of `ascribe` that wrote the report |
| `error` | string or null | Why the report couldn't be made (exit code `2`); no section is there then |
| `sections` | array of strings | The sections asked for, in the report's order |
| `builds_checked` | array of strings | The builds `problems` checked and `builds` and `pages` report on |
| `not_run` | array of strings | The sections asked for that couldn't run; each says why in its own `not_run` |
| `summary` | object | How many `errors`, `warnings`, and `advice` the sections found, and how many of each kind of next step (`by_next`) |
| `next_command` | string or null | The command that lists everything a list leaves out |
| `problems`, `inventory`, `builds`, `pages`, `links`, `agents` | | Each section asked for |

A problem is a diagnostic as `ascribe check` writes it in [its JSON](check.md#json-output), with `next`, `help`, and `docs`. Each list is an object: `items`, at most `--limit` of them, `shown`, `total`, and `truncated`. A section that runs a program has `not_run`, `null` when it ran, or its `reason` and `how` to get it to run.
