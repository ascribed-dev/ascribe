---
title: "`ascribe sources`, `permalinks`, and `vale`"
description: "Fetch and update source repositories, keep the list of permalinks, and eject Vale's configuration."
---

The commands that look after what a project keeps beside its pages: its source repositories, its permalinks, and Vale's configuration. The options every command accepts are in the [command reference](../cli.md#options-every-command-accepts).

## `ascribe sources`
@id {permalink=true}: cli-sources
@available: ascribe 0.2.0

Copies code from [sources in other repositories](../content-model/output.md#a-source-in-another-repository) into the project, and moves their pins. `check` and `build` read only the copies, so `ascribe sources fetch` and `ascribe sources update` are the only commands that reach another repository. They run `git`, with `git`'s own credentials: whatever lets `git fetch <url>` work in your shell lets them fetch. They never prompt, so a missing credential fails at once.

Fetched repositories are kept in a cache outside the project, one per URL, so a second run fetches only what's new: `ascribe` in your cache folder (`$XDG_CACHE_HOME`, `~/.cache`, `~/Library/Caches`, or `%LOCALAPPDATA%`), or `ASCRIBE_CACHE_DIR`. Nothing needs the cache but these commands; deleting it costs only a slower next run.

What's fetched is someone else's repository, so it's only ever read as text: `git` runs with no hooks, no submodules, and only the `https`, `http`, `ssh`, `git`, and `file` transports, and a file over 1 MB, or one that isn't text, isn't copied.

### `ascribe sources fetch`
@id {permalink=true}: cli-sources-fetch

Makes the copies match `ascribe.lock`: at each source's pin, it copies the files snippets name that have no copy or a wrong one, and removes the copies no snippet names. It's what you run after writing a new `@snippet` from a source in another repository, or after cloning if the copies were left out. It never moves a pin; a source with no pin yet is pinned to the head of its branch.

@include: ../../_generated/cli-sources-fetch-options.md

It lists each copy it wrote or removed. The first time a source's files are copied, it says so: the copies are committed with the docs, so everyone who can read the docs repository can read them.

| Code | Meaning |
|---|---|
| `0` | The copies match the lock |
| `1` | A file a snippet names couldn't be copied: it isn't in the repository at the pin, it's over 1 MB, or it isn't text. The reason goes to standard error, and `ascribe check` reports the snippet. |
| `2` | It couldn't run: no `ascribe.toml`, a content model with errors, an `ascribe.lock` it can't read, a name that isn't a source in another repository, or `git` failing (no network, no access), with the source named and `git`'s own message |

### `ascribe sources update`
@id {permalink=true}: cli-sources-update

Moves each source's pin to the head of its `branch` (or to `--to`), copies the files snippets use again at the new commit, rewrites `ascribe.lock`, and says what changed: the commits between the old pin and the new (how many, and the first lines of the newest 20, or that the pin moved back), the copies that changed, and the pages whose examples changed, grouped as [`ascribe drift`](diff.md#cli-drift) groups them. With nothing to move, it changes no file and says so.

@include: ../../_generated/cli-sources-update-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/sources-update.txt

The pages are found as `ascribe drift` finds them, comparing the working tree, with the new copies, against `HEAD`, so run it in a clean checkout. A snippet whose region is gone at the new commit is listed among the examples that no longer resolve, and the update still completes: `ascribe check` then fails on it, which is the signal to fix the page. When the project isn't in a git repository, or has no commit yet, the pages aren't listed, and it says why: commit the docs once before the first update.

`--format markdown` writes Markdown for a pull request's description: each source that moved, with its commits and copies, then the pages, each linked to its route on `[consumer] site`. For a repository on github.com, the old and new pins link to GitHub's comparison of them. Commit subjects are code spans, so a subject like `Fixes #12 @someone` in the code's repository can't close an issue or notify anyone from the docs' pull request. It writes nothing when nothing moved. `--format json` writes one document: `schema_version` (`1`), `ascribe_version`, `changed`, `sources`, `pages` (as in the [drift JSON](diff.md#drift-json), or null), and `pages_unavailable` (why there are no pages, or null). Each source has `name`, `git`, `followed` (the branch, `HEAD`, or `--to`'s revision), `from` and `to` (the pins, `from` null for a first pin), `moved`, `back` (true when the new pin isn't after the old one: it moved back, or to another line of history, and no commits are counted), `commits` (`count` and `newest`, each with `commit` and `subject`, or null), `files` (each with `path` and `change`: `added`, `changed`, or `removed`), `failed` (each with `path` and `reason`), and `first_copy`.

| Code | Meaning |
|---|---|
| `0` | It ran, whether or not a pin moved. A file a snippet names that couldn't be copied is reported on standard error. |
| `2` | It couldn't: no network, no access, an unknown revision, `--to` with more than one source, or the reasons `fetch` gives. The source is named, with `git`'s own message. |

### `ascribe sources status`
@id {permalink=true}: cli-sources-status

Shows each source in another repository: its repository and branch, its pin, and each copy's state (current, changed here, missing, not in the lock, unused, not copied yet, or not in the repository at the pin). It reads only the project's files: no `git`, no network.

@include: ../../_generated/cli-sources-status-options.md

| Code | Meaning |
|---|---|
| `0` | It showed the sources, whatever state their copies are in |
| `2` | It couldn't run: a usage error, no `ascribe.toml`, or one with errors |

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, and `sources`, each with `name`, `git`, `branch` (or null), `commit` (the pin, or null), and `files`, each with `path` and `state`: `current`, `changed`, `missing`, `unlocked`, `unused`, `not_copied`, or `not_at_pin`.

## `ascribe permalinks`
@id {permalink=true}: cli-permalinks
@available: next

Keeps the project's list of [permalinks](../directives.md#permalinks), `ascribe.permalinks` beside `ascribe.toml`, checks names against it, and retires permalinks. The list names each permalink the content declares and where, and each alias and retired permalink in `ascribe.toml`'s [`[permalinks]`](../content-model/checks.md#20-permalinks), and is committed with the docs: with it, `ascribe check` reports a permalink that goes away (`permalink-removed`) and one the list doesn't have yet (`permalinks-list-behind`), and a product's CI can check the names it links to with `verify`. A project without the list isn't told about either. Its format is in the [permalinks list contract](../../contracts/permalinks-list.md), and [Permalinks](../../guides/permalinks.md) walks through using them.

### `ascribe permalinks sync`
@id {permalink=true}: cli-permalinks-sync

Brings `ascribe.permalinks` up to date with the project, and makes it when there's none: it adds each permalink the list doesn't have, records where each is declared now, so moving a heading to another page and running `sync` changes one line, and records each alias and retired permalink as `ascribe.toml` has it. It never drops a name the project no longer has: that stays in the list, and `ascribe check` reports it until it's back or retired, so running `sync` can't hide a removal. It lists what it added, moved, and recorded, and warns about each name it kept. With `--check`, it writes nothing, says what it would change, and exits with `1` when the list would change, so CI can fail on a list that wasn't committed with the change; `ascribe check` reports the same thing as `permalinks-list-behind`.

@include: ../../_generated/cli-permalinks-sync-options.md

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, `changed` (whether the file was written, or with `--check` would be), `created`, `added` (names), `moved` (each with `name`, `from`, and `to`), `aliased` and `retired` (the aliases and retired permalinks recorded, new or changed), and `removed` (the names kept that the project no longer has). 
| Code | Meaning |
|---|---|
| `0` | The list is up to date, whether or not it changed; with `--check`, it was already |
| `1` | With `--check`, the list would change: it's behind the content, or there's none |
| `2` | It couldn't run: the project doesn't load, or the list can't be read or written |

### `ascribe permalinks list`
@id {permalink=true}: cli-permalinks-list

Shows every permalink the content declares, with the file and line it's declared on, and each alias and retired permalink in `ascribe.toml`, and says which ones `ascribe.permalinks` doesn't have yet, has somewhere else, or has that the project no longer has. It changes nothing. It exits with `0` whatever it finds, and `2` when the project doesn't load or the list can't be read.

@include: ../../_generated/cli-permalinks-list-options.md

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, `has_list`, `permalinks`, each with `name`, `place` (as the list writes it), `file`, `line`, `state` (`listed`, `moved`, or `not_listed`), and `listed_at` (the list's place for a moved one, or null); `aliases`, each with `name`, `target`, and `state` (`listed` or `not_listed`); `retired`, each with `name`, `reason`, `to` (or null), and `state`; and `removed`, each with `name`, `was` (`permalink`, `alias`, or `retired`: the list's table), and `place` (the list's place for a permalink, the name an alias led to, or a retired permalink's reason).

### `ascribe permalinks verify`
@id {permalink=true}: cli-permalinks-verify

Checks that each name is a permalink or an alias in a list, and names each one that isn't, and each one that was retired, with the reason the docs give. It reads only the list, not the docs, so a product's repository can run it against a copy of the docs' list: [Checking a product's links](../../guides/permalinks.md#checking-a-products-links) shows the step for CI.

```sh
ascribe permalinks verify cli-check ASC036
ascribe permalinks verify --against docs/ascribe.permalinks --from permalinks.txt
```

@include: ../../_generated/cli-permalinks-verify-options.md

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, `list` (the list's path, as given), `found` and `missing`, each a list of names in the order given, and `retired`, each with `name` and `reason`.

| Code | Meaning |
|---|---|
| `0` | The list has every name |
| `1` | It doesn't have one, or has it as retired; each is named on standard error, a retired one with its reason |
| `2` | It couldn't check: no list where it looked, or a list or a `--from` file it can't read |

### `ascribe permalinks retire`
@id {permalink=true}: cli-permalinks-retire

Retires permalinks on purpose, with a reason: it writes an entry for each in `ascribe.toml`'s [`[permalinks.retired]`](../content-model/checks.md#20-permalinks), in place, keeping the file's comments and order, and, when the project has `ascribe.permalinks`, records there each one the content no longer declares. A retired permalink's address shows the reason, or leads to `--to`; `ascribe check` no longer reports it as removed; and `verify` tells a product it was retired, with the reason.

```sh
ascribe permalinks retire rotate-keys --reason "Merged into the security guide" --to guides/security.md
ascribe permalinks retire --page guides/legacy.md --reason "The sync agent was removed in 4.0"
ascribe permalinks retire --under guides/legacy/ --reason "The sync agent was removed in 4.0"
```

It retires a name given only when the content no longer declares it and `ascribe.permalinks` has it, and with `--page` or `--under`, each permalink declared in that page or in a file under that folder, or that the list has there. `--page`, `--under`, and `--to` take a path from the current folder, as `ascribe check` prints it, or from the content root; what it writes in `ascribe.toml` is from the content root, as links there are. It refuses a name still declared anywhere else, an alias, or one already retired, and changes nothing then. Retiring what a page holds before the page is deleted leaves each name both declared and retired (`permalink-name-conflict`) until it is. It lists each name it retired.

@include: ../../_generated/cli-permalinks-retire-options.md

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, `retired` (the names, in order), `reason`, `to` (or null), and `list_changed` (whether `ascribe.permalinks` was written). It exits with `0` when the names are retired, and `2` when the project doesn't load, the reason is empty, a name can't be retired, or a file can't be written.

## `ascribe vale`
@id {permalink=true}: cli-vale
@available: next

Sets up [Vale](https://vale.sh), which checks the prose when `[checks.vale]` in `ascribe.toml` turns it on (see [Prose, through Vale](../../guides/vale.md)). Vale itself is run by `ascribe check --vale` and the editor.

### `ascribe vale eject`
@id {permalink=true}: cli-vale-eject

Writes the preset `[checks.vale]` names out as the project's own Vale configuration, for a project that wants more than turning a preset's rules off: `.vale.ini` and a `.vale/` styles folder at the project root, with the rules `off` names turned off in `.vale.ini`. Then it changes `[checks.vale]` in `ascribe.toml` to `config = ".vale.ini"`, in place of `preset` and `off`, keeping its other keys and comments. The project's vocabulary is still Ascribe's to write, under `.ascribe/vale/`.

It takes no options. It lists each file it wrote, `ascribe.toml` last.

| Code | Meaning |
|---|---|
| `0` | The preset is the project's own now |
| `2` | It wrote nothing: no `ascribe.toml`, a content model with errors, a `[checks.vale]` that names no preset, or a `.vale.ini` or `.vale` that's there already. Or a file couldn't be written; the files it wrote before that stay. |
