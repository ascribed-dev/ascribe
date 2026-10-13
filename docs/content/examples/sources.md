---
type: reference
title: "`ascribe init`, `sources`, `observe`, `permalinks`, `templates`, and `vale`"
description: "Start a project, fetch and update source repositories, record what tested a folder's examples, keep the list of permalinks, write page templates into a project, and eject Vale's configuration."
---

The commands that start a project, and look after what it keeps beside its pages: its source repositories, the record of what tested its examples, its permalinks, its page templates, and Vale's configuration. The options every command accepts are in the [command reference](../command/index.md#options-every-command-accepts).

## `ascribe sources`
@id {permalink=true}: cli-sources
@available: ascribe 0.2.0

Copies code from [sources in other repositories](../content-model/output.md#a-source-in-another-repository) into the project, and moves their pins. `check` and `build` read only the copies, so `ascribe sources fetch` and `ascribe sources update` are the only commands that reach another repository. They run `git`, with `git`'s own credentials: whatever lets `git fetch <url>` work in your shell lets them fetch. They never prompt, so a missing credential fails at once.

Fetched repositories are kept in a cache outside the project, one per URL, so a second run fetches only what's new: `ascribe` in your cache folder (`$XDG_CACHE_HOME`, `~/.cache`, `~/Library/Caches`, or `%LOCALAPPDATA%`), or `ASCRIBE_CACHE_DIR`. Nothing needs the cache but these commands; deleting it costs only a slower next run.

What's fetched is someone else's repository, so it's only ever read as text: `git` runs with no hooks, no submodules, and only the `https`, `http`, `ssh`, `git`, and `file` transports, and a file over 1 MB, or one that isn't text, isn't copied.

### `ascribe sources fetch`
@id {permalink=true}: cli-sources-fetch

Makes the copies match `ascribe.lock`: at each source's pin, it copies the files snippets name that have no copy or a wrong one, and removes the copies no snippet names. It's what you run after writing a new `@snippet` from a source in another repository, or after cloning if the copies were left out. It never moves a pin; a source with no pin yet is pinned to the head of its branch.

Beside the files snippets name, it copies the repository's [tested examples](tested.md#when-the-code-is-in-another-repository) config, `ascribe.examples.toml`, and every file in its `examples.observations/`, whatever the source's `include` says, and the lock lists them as it does every copy. They're read as text, and nothing in them is run. A source whose copies are all in order is left alone, without reaching its repository, so a pin made by an `ascribe` that didn't copy them gets them with the next `update`, or the next time a copy is fetched.

@include: ../_generated/cli-sources-fetch-options.md

It lists each copy it wrote or removed. The first time a source's files are copied, it says so: the copies are committed with the docs, so everyone who can read the docs repository can read them.

| Code | Meaning |
|---|---|
| `0` | The copies match the lock |
| `1` | A file a snippet names couldn't be copied: it isn't in the repository at the pin, it's over 1 MB, or it isn't text. The reason goes to standard error, and `ascribe check` reports the snippet. |
| `2` | It couldn't run: no `ascribe.toml`, a content model with errors, an `ascribe.lock` it can't read, a name that isn't a source in another repository, or `git` failing (no network, no access), with the source named and `git`'s own message |

### `ascribe sources update`
@id {permalink=true}: cli-sources-update

Moves each source's pin to the head of its `branch` (or to `--to`), copies the files snippets use again at the new commit, rewrites `ascribe.lock`, and says what changed: the commits between the old pin and the new (how many, and the first lines of the newest 20, or that the pin moved back), the copies that changed, the [tested examples](tested.md#when-the-code-is-in-another-repository) whose status changed, and the pages whose examples changed, grouped as [`ascribe drift`](../review/diff.md#cli-drift) groups them. With nothing to move, it changes no file and says so.

A tested example snippets use is listed when the new pin changed its status: it went stale (the file changed, and its observation didn't follow), no observation lists it any more, or it's tested as it is now. Each is read from the copies, before and after.

@include: ../_generated/cli-sources-update-options.md

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/sources-update.txt

The pages are found as `ascribe drift` finds them, comparing the working tree, with the new copies, against `HEAD`, so run it in a clean checkout. A snippet whose region is gone at the new commit is listed among the examples that no longer resolve, and the update still completes: `ascribe check` then fails on it, which is the signal to fix the page. When the project isn't in a git repository, or has no commit yet, the pages aren't listed, and it says why: commit the docs once before the first update.

`--format markdown` writes Markdown for a pull request's description: each source that moved, with its commits, its copies, and its tested examples whose status changed, then the pages, each linked to its route on `[consumer] site`. For a repository on github.com, the old and new pins link to GitHub's comparison of them. Commit subjects are code spans, so a subject like `Fixes #12 @someone` in the code's repository can't close an issue or notify anyone from the docs' pull request. It writes nothing when nothing moved. `--format json` writes one document: `schema_version` (`1`), `ascribe_version`, `changed`, `sources`, `pages` (as in the [drift JSON](../review/diff.md#drift-json), or null), and `pages_unavailable` (why there are no pages, or null). Each source has `name`, `git`, `followed` (the branch, `HEAD`, or `--to`'s revision), `from` and `to` (the pins, `from` null for a first pin), `moved`, `back` (true when the new pin isn't after the old one: it moved back, or to another line of history, and no commits are counted), `commits` (`count` and `newest`, each with `commit` and `subject`, or null), `files` (each with `path` and `change`: `added`, `changed`, or `removed`), `examples` (each tested example whose status changed, with `path`, `status`, and `was`: `current`, `stale`, or `unobserved`, `was` null when it wasn't an example before), `failed` (each with `path` and `reason`), and `first_copy`.

| Code | Meaning |
|---|---|
| `0` | It ran, whether or not a pin moved. A file a snippet names that couldn't be copied is reported on standard error. |
| `2` | It couldn't: no network, no access, an unknown revision, `--to` with more than one source, or the reasons `fetch` gives. The source is named, with `git`'s own message. |

### `ascribe sources status`
@id {permalink=true}: cli-sources-status

Shows each source in another repository: its repository and branch, its pin, and each copy's state (current, changed here, missing, not in the lock, unused, not copied yet, or not in the repository at the pin). A tested examples' config or observation is current when it's the file the lock pins, though no snippet uses it. It reads only the project's files: no `git`, no network.

@include: ../_generated/cli-sources-status-options.md

| Code | Meaning |
|---|---|
| `0` | It showed the sources, whatever state their copies are in |
| `2` | It couldn't run: a usage error, no `ascribe.toml`, or one with errors |

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, and `sources`, each with `name`, `git`, `branch` (or null), `commit` (the pin, or null), and `files`, each with `path` and `state`: `current`, `changed`, `missing`, `unlocked`, `unused`, `not_copied`, or `not_at_pin`.

## `ascribe observe`
@id {permalink=true}: cli-observe
@available: next

Runs the test of a folder of [tested examples](tested.md), and when it passes, writes down what it passed with: `examples.observations/default.toml`, beside `ascribe.examples.toml`. It's run in the code's folder, the one that holds `ascribe.examples.toml`, not the docs project's: it doesn't look in parent folders for a config, and doesn't read `ascribe.toml` or `--config`. For code in another repository, that's the code's repository: it refuses to run in a source's copies, which it knows from the `ascribe.lock` above them. So `ascribe` has to be installed where the code is, and today that means Node, the way the [command is installed](../getting-started.md#install-the-command).

@include: ../_generated/cli-observe-options.md

It prints the config's path and every command it will run, then runs each `[environment]` command, then `test`, through the platform's shell (`sh`, or `cmd` on Windows), in the folder, with your environment and standard input closed. The test's output goes to the terminal as it comes, and there's no time limit: you started it, and you can stop it.

```console
$ ascribe observe
Running the commands in ascribe.examples.toml:
  test         node --test test/greet.test.mjs
  environment  node --version
node --test test/greet.test.mjs passed.
Wrote examples.observations/default.toml: 2 files, Node 24.9.0 on Linux.
```

When `test` exits 0, it hashes every example and writes the observation whole, to a file beside it that it then renames, so a run that's stopped part way leaves the old one. When it doesn't, it writes nothing. A file that changes while the test runs is hashed as it is at the end, so run it on a clean tree.

| Code | Meaning |
|---|---|
| `0` | The test passed, and the observation is written |
| `1` | The test failed. Nothing was written, and the old observation is as it was. |
| `2` | It couldn't run: the current folder is a source's copies (`sources/<name>/` of a project whose `ascribe.lock` pins it), whose config is never run; no `ascribe.examples.toml` in the current folder, one with errors or with no `test`, an `[environment]` command that failed or printed nothing, no shell, or an observation it couldn't write |

With `--format json`, what the commands print goes to standard error, and standard output has one document: `schema_version` (`1`), `ascribe_version`, `config`, `environment` (each with `key`, `command`, and `kept`, the line the observation records), `test` (`command`, `passed`, and `exit_code`, null when a signal stopped it), `observation` (the file it wrote, or null), `recorded` (the observation's `[environment]`, by name, `os` and `arch` among them), `files` (each example's hash, by path), and `tested` (what a page says it was tested with, or absent). When the test fails, the last four are empty. Its schema is in the [JSON report contract](../contracts/json-reports/sources.md#ascribe-observe).

## `ascribe permalinks`
@id {permalink=true}: cli-permalinks
@available: ascribe 0.3.0

Keeps the project's list of [permalinks](../writing/directives.md#permalinks), `ascribe.permalinks` beside `ascribe.toml`, checks names against it, and retires permalinks. The list names each permalink the content declares and where, and each alias and retired permalink in `ascribe.toml`'s [`[permalinks]`](../content-model/checks.md#20-permalinks), and is committed with the docs: with it, `ascribe check` reports a permalink that goes away (`permalink-removed`) and one the list doesn't have yet (`permalinks-list-behind`), and a product's CI can check the names it links to with `verify`. A project without the list isn't told about either. Its format is in the [permalinks list contract](../contracts/permalinks-list.md), and [Permalinks](../permalinks/index.md) walks through using them.

### `ascribe permalinks sync`
@id {permalink=true}: cli-permalinks-sync

Brings `ascribe.permalinks` up to date with the project, and makes it when there's none: it adds each permalink the list doesn't have, records where each is declared now, so moving a heading to another page and running `sync` changes one line, and records each alias and retired permalink as `ascribe.toml` has it. It never drops a name the project no longer has: that stays in the list, and `ascribe check` reports it until it's back or retired, so running `sync` can't hide a removal. It lists what it added, moved, and recorded, and warns about each name it kept. With `--check`, it writes nothing, says what it would change, and exits with `1` when the list would change, so CI can fail on a list that wasn't committed with the change; `ascribe check` reports the same thing as `permalinks-list-behind`.

@include: ../_generated/cli-permalinks-sync-options.md

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, `changed` (whether the file was written, or with `--check` would be), `created`, `added` (names), `moved` (each with `name`, `from`, and `to`), `aliased` and `retired` (the aliases and retired permalinks recorded, new or changed), and `removed` (the names kept that the project no longer has). 
| Code | Meaning |
|---|---|
| `0` | The list is up to date, whether or not it changed; with `--check`, it was already |
| `1` | With `--check`, the list would change: it's behind the content, or there's none |
| `2` | It couldn't run: the project doesn't load, or the list can't be read or written |

### `ascribe permalinks list`
@id {permalink=true}: cli-permalinks-list

Shows every permalink the content declares, with the file and line it's declared on, and each alias and retired permalink in `ascribe.toml`, and says which ones `ascribe.permalinks` doesn't have yet, has somewhere else, or has that the project no longer has. It changes nothing. It exits with `0` whatever it finds, and `2` when the project doesn't load or the list can't be read.

@include: ../_generated/cli-permalinks-list-options.md

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, `has_list`, `permalinks`, each with `name`, `place` (as the list writes it), `file`, `line`, `state` (`listed`, `moved`, or `not_listed`), and `listed_at` (the list's place for a moved one, or null); `aliases`, each with `name`, `target`, and `state` (`listed` or `not_listed`); `retired`, each with `name`, `reason`, `to` (or null), and `state`; and `removed`, each with `name`, `was` (`permalink`, `alias`, or `retired`: the list's table), and `place` (the list's place for a permalink, the name an alias led to, or a retired permalink's reason).

### `ascribe permalinks verify`
@id {permalink=true}: cli-permalinks-verify

Checks that each name is a permalink or an alias in a list, and names each one that isn't, and each one that was retired, with the reason the docs give. It reads only the list, not the docs, so a product's repository can run it against a copy of the docs' list: [Checking a product's links](../permalinks/index.md#checking-a-products-links) shows the step for CI.

```sh
ascribe permalinks verify cli-check ASC036
ascribe permalinks verify --against docs/ascribe.permalinks --from permalinks.txt
```

@include: ../_generated/cli-permalinks-verify-options.md

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

@include: ../_generated/cli-permalinks-retire-options.md

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, `retired` (the names, in order), `reason`, `to` (or null), and `list_changed` (whether `ascribe.permalinks` was written). It exits with `0` when the names are retired, and `2` when the project doesn't load, the reason is empty, a name can't be retired, or a file can't be written.

## `ascribe init`
@id {permalink=true}: cli-init
@available: next

Starts a project: writes its first `ascribe.toml`, and makes the folder its pages go in, `docs/`. With `--templates`, it takes the page templates too, as [`ascribe templates eject`](sources.md#cli-templates-eject) with no name does.

```sh
mkdir handbook && cd handbook
ascribe init --templates
```

@include: ../_generated/cli-init-options.md

It writes in the current folder, or in `DIR`, a path from the current folder, which it makes if it isn't there. It asks nothing. Where there's an `ascribe.toml` in the folder or above it, it refuses, names the one it found, and writes nothing: `ascribe templates eject`, run there, adds the templates to that project. To start a second project in a repository that has one, name a folder outside it. It takes no `--config`, since the project is the one it starts.

The file it writes is small, and says what each part is for:

@snippet: code:crates/ascribe-cli/tests/output/init-ascribe.toml

It sets the content root, `docs`, though that's the default, since it's the first thing a project changes. It declares one type, `page`, with a title and a description; since it declares a type, each page names it in its frontmatter, with `type: page`. Its comments name what most projects add next, each with its section of the [`ascribe.toml` reference](../content-model/index.md), and it adds none of them: no site address, no builds, and no dimensions.

With `--templates`, the six types are added after `page`, and the templates, their guides, Ascribe's notes, and `templates.lock` are written into `templates/`, as `eject` writes them. When a file is where one would be written, it's named, as `eject` names it, and nothing is written.

It lists each file it wrote, `ascribe.toml` last, from the current folder; the content root, when it made it; with `--templates`, each type it added, as `eject` says it; and last, the [Astro guide](../astro/index.md), to publish the pages as a site.

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/init.txt

With `--format json`, the document has `schema_version` (`2`), `ascribe_version`, `written` (the files, from the new project's folder, in the order written), `content_root` (from the new project's folder), `content_root_made` (whether it made it), `templates` (what `eject` did, with the fields its own JSON has, or null without `--templates`), and `next` (the Astro guide's address). When `templates.refused` isn't empty, `written` is empty.

| Code | Meaning |
|---|---|
| `0` | The project is there: `ascribe.toml` is written, the content root is there, and with `--templates`, every template and its type |
| `2` | It wrote nothing: there's an `ascribe.toml` in the folder or above it, the folder is a file, a template was refused, or it was given `--config`. Or a file couldn't be written; the files it wrote before that stay. |

## `ascribe templates`
@id {permalink=true}: cli-templates
@available: next

Writes the page templates Ascribe carries into a project, as the project's own files, and declares a content type for each, whose [`template` and `guide`](../content-model/index.md#41-page-types) name them. The templates are [The Good Docs Project](https://www.thegooddocsproject.dev/template)'s, converted into Ascribe pages; [Page templates](../templates/index.md) says where they come from, and how to use them.

The set has six templates, which `ascribe templates list` names. A page is started by copying the template's file to where the page goes; the template names its type in its frontmatter, as the page must. `eject` adds to a project, and never starts one: [`ascribe init --templates`](sources.md#cli-init) does, with the set. `status` says what's changed in the set since the project's copy, and changes nothing.

### `ascribe templates list`
@id {permalink=true}: cli-templates-list

Shows each template the set has, in a line, with what it's for, and which release of theirs the set was converted from. It reads only what the binary carries, so it works anywhere, in a project or not. It takes no options, and exits with `0`.

### `ascribe templates eject`
@id {permalink=true}: cli-templates-eject

Writes each template named, with its guide and Ascribe's notes on its type, into the project, and adds its type to `ascribe.toml`. With no name, it takes every template of the set.

```sh
ascribe templates eject how-to
```

@include: ../_generated/cli-templates-eject-options.md

For each template, it writes the template, its guide, and Ascribe's notes on the type into `templates/`, beside `ascribe.toml`, or into `--to`'s folder, which is a path from the current folder. The folder must be inside the folder `ascribe.toml` is in, and outside the content root and the output directory, where no type can name a file. Then it adds the type at the end of `ascribe.toml`, changing nothing above it, so its comments and order stay as they are. The paths in it are from the folder `ascribe.toml` is in, whatever folder `--to` was given from:

```toml
[types.how-to]
template = "templates/how-to.md"
guide = ["templates/how-to.guide.md", "templates/how-to.ascribe.md"]

[types.how-to.frontmatter]
title = "string"
description = "string"
```

The type's `guide` lists their guide, then Ascribe's notes, in the order to read them. Its frontmatter has the fields the template's has, apart from `type`, each a string a page must have. When one of the project's types declares one of those fields as a string with more to it, such as `title = { type = "string", inline = "code" }` or `description = { type = "string", role = "description" }`, the type added declares it the same way, so its pages' titles and descriptions read as the project's other pages' do. A project that declared no types had every page of the implicit `page` type, so `eject` writes that type out first, as it was:

```toml
[types.page.frontmatter]
title = "string"
```

A project that declares types has each page name one, so its pages that named none, which were of `page`, now need `type: page`: `eject` lists them, and `ascribe check` reports each with a fix that adds it.

Beside the templates, it writes `templates.lock`: for each template, the files written, with a hash of each as written, their template and guide under `files` and Ascribe's notes under `ascribe-files`, then their repository, tag, commit, and license, their summary of the type, and the version of Ascribe that wrote it. A later `eject` adds to it. Commit it with the templates, and don't edit it.

A project that declares a template's type already, with its own frontmatter, takes the template for it with `--keep-type`:

```sh
ascribe templates eject reference --keep-type
```

It writes the same files and `templates.lock`, and adds only `template` and `guide` to the type's table, after its last entry, leaving its frontmatter, its other keys, and every comment as they are. A template whose type isn't declared is added as without it. It refuses a type written as an inline or a dotted table, which it can't add a line to, rather than a table under a `[types.<name>]` header of its own; a type that names a `template` or a `guide` already; and a type whose frontmatter doesn't declare each field the template's frontmatter has as a string, since no page started from the template would check. It names the field.

It checks everything before it writes anything. A type `ascribe.toml` declares already, without `--keep-type`, a file where one would be written, and a name the set doesn't have are each named, on standard error, and then nothing is written. Without `ascribe.toml`, it says so, and that `ascribe init --templates` starts a project with the templates.

It lists each file it wrote, `ascribe.toml` last, from the current folder; each type it added, with the fields it declared as the project's other types do; the pages that name no type, up to ten, which `ascribe check` reports until each has its `type:`; each type it kept, with the keys it left as they were; and how to start a page.

With `--format json`, the document has `schema_version` (`2`), `ascribe_version`, `written` (the files, from the folder `ascribe.toml` is in, in the order written), `added`, each with `name`, `summary`, `template`, `guide` (their guide, then Ascribe's notes), and `copied` (the fields of its frontmatter declared as another of the project's types declares them); `kept`, the types `--keep-type` kept, each with `name`, `summary`, `template`, `guide`, and `left` (the type's keys left as they were, in the order `ascribe.toml` has them); `refused`, each with `name`, `reason` (`unknown`, `declared`, `file_in_the_way`, or, with `--keep-type`, `type_not_table`, `type_has_template`, or `frontmatter_differs`), `path` (the file in the way, or null), and `message`; `page_declared`, whether `[types.page]` was written out; and `untyped`, the pages that name no type, from the folder `ascribe.toml` is in. When `refused` isn't empty, `written`, `added`, and `kept` are.

| Code | Meaning |
|---|---|
| `0` | Every template named was written, and its type added, or kept with its `template` and `guide` added |
| `2` | It wrote nothing: a template was refused, there's no `ascribe.toml`, the content model has errors, the folder can't hold templates, `templates.lock` isn't one it can add to, or the types can't be added to `ascribe.toml`. Or a file couldn't be written; the files it wrote before that stay. |

### `ascribe templates status`
@id {permalink=true}: cli-templates-status

Says what's changed in the templates since the project took them: for each one, whether the set this `ascribe` carries has changed it since the copy, and whether the project has changed its copy; then each template of the set the project doesn't have. It compares the project's files and `templates.lock` with the set the binary carries, and reads nothing else: no network, so what it says is as of this `ascribe`, and its first line names the release of theirs the set is from. It changes no file: a team that wants a change copies it by hand.

```sh
ascribe templates status
```

@include: ../_generated/cli-templates-status-options.md

`templates.lock` keeps a hash of each file as `eject` wrote it, and that's what tells the two apart: when the set's text isn't what was written, the set changed it; when the project's file isn't, the project did. A template can be both. A file the set changed and the project didn't is listed with what's different by section: each heading the set added, removed, or changed the text under, with `the start` for what comes before the first heading. A file that's gone is `deleted`. Line endings a checkout changed to CRLF aren't a change.

It finds a template's entry in the `templates.lock` in `templates/`, or beside the template a type names, and finds the templates a project took from the lock's entries and from the types named as the set's templates are. When there's no lock, it's one `eject` didn't write, or it has no entry for a type of the set's, it says so, and says what it can from the files alone: whether each is the same as the set's or different, without telling who changed it.

With `--diff NAME`, it prints only the change to that template, as a unified diff from the project's files to the set's, for each file the set has changed since the copy. When the project hasn't changed its copy, that's the upstream change itself; when it has, or the lock remembers nothing of the file, it says so on standard error, since the diff then undoes the project's changes too.

With `--format json`, the document has `schema_version` (`1`), `ascribe_version`, `set` (the set's `repository`, `tag`, and `commit`), `upstream_changed` (whether the set has changed a template the project took, or no longer has one), `templates`, and `not_taken`, each with `name` and `summary`. Each template has `name`, `record` (`entry`, `no_lock`, `no_entry`, or `invalid_lock`), `lock` (the lock its entry is in, or the one looked in), `lock_problem` (why the lock can't be read, or null), `copied_from` (their release, or null without an entry), `upstream` (`unchanged`, `changed`, `unknown`, or `removed`, when the set no longer has it), `project` (`unchanged`, `changed`, or `unknown`), and `files`, each with `path`, `kind` (`template`, `guide`, or `notes`), `upstream`, `project` (`unchanged`, `changed`, `deleted`, or `unknown`), `matches_set` (whether it's the set's text now, or null), and `sections` (each with `heading`, null for the start, and `change`: `added`, `removed`, or `changed`; null unless the set changed the file and the project didn't). Paths are from the folder `ascribe.toml` is in.

| Code | Meaning |
|---|---|
| `0` | It ran, whatever it found |
| `1` | With `--exit-code`, the set has changed a template the project took, or no longer has one; with `--diff`, the one named. Or `--diff` names a template of the set the project hasn't taken. |
| `2` | It couldn't run: no `ascribe.toml`, one that doesn't load, or `--diff` with a name the set doesn't have |

## `ascribe vale`
@id {permalink=true}: cli-vale
@available: ascribe 0.3.0

Sets up [Vale](https://vale.sh), which checks the prose when `[checks.vale]` in `ascribe.toml` turns it on (see [Prose, through Vale](../checks/vale.md)). Vale itself is run by `ascribe check --vale` and the editor.

### `ascribe vale eject`
@id {permalink=true}: cli-vale-eject

Writes the preset `[checks.vale]` names out as the project's own Vale configuration, for a project that wants more than turning a preset's rules off: `.vale.ini` and a `.vale/` styles folder at the project root, with the rules `off` names turned off in `.vale.ini`. Then it changes `[checks.vale]` in `ascribe.toml` to `config = ".vale.ini"`, in place of `preset` and `off`, keeping its other keys and comments. The project's vocabulary is still Ascribe's to write, under `.ascribe/vale/`.

It takes no options. It lists each file it wrote, `ascribe.toml` last.

| Code | Meaning |
|---|---|
| `0` | The preset is the project's own now |
| `2` | It wrote nothing: no `ascribe.toml`, a content model with errors, a `[checks.vale]` that names no preset, or a `.vale.ini` or `.vale` that's there already. Or a file couldn't be written; the files it wrote before that stay. |
