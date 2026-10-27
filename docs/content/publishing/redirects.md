---
type: how-to
title: Keep the addresses you've published
description: "Record where a page went when it moves or goes, so a link to its old address still leads somewhere: `ascribe.addresses`, the check that compares with the base, and the redirects the build writes."
---

A link to a page outlives the page's place in the docs: a bookmark, a search result, a product's help menu, an answer on a forum. Rename a folder or merge two pages, and every one of those links stops working unless someone decided where it should lead. `ascribe.addresses` is where that decision is written. With it, a move records itself, `ascribe check` reports an [address](../glossary.md#address) the docs published that nothing accounts for, and the build writes the redirects the host needs.

## Turn it on

A project [`ascribe init`](../install.md#cli-init) started has it already: init writes `ascribe.addresses` beside `ascribe.toml`, so commit it with the docs, and its addresses are kept from the first commit.

A project started before that, or without `init`, makes the file beside `ascribe.toml`, and commits it with the docs:

```console
$ ascribe addresses init
Wrote ascribe.addresses. Moves are recorded from now on.
```

A project without the file records nothing, and the check compares nothing, so deleting it is the decision to stop keeping the addresses.

The editor offers it too. After you move a page of a project without the file, in the Explorer or in Pages, VS Code asks once for the project whether to **Keep the addresses you've published**. Choose it, and the extension runs `ascribe addresses init`, then `ascribe addresses move` for each page that moved, so the move that prompted it is recorded. Dismissed, it isn't asked again for the project.

## `ascribe.addresses`

The file says where each address the docs published went. Ascribe writes it, each table sorted, one entry per line; you edit only a retired entry's reason:

```toml
# Where addresses the docs published went. Written by `ascribe rename`, the
# editor, `ascribe addresses`, and `ascribe check --fix`; edit only
# [retired]'s reasons.
version = 1

[moved]
"/docs/intro.html" = "getting-started.md"
"guides/install.md#install-with-npm" = "guides/install.md#install-npm"
"guides/keys.md" = "security/keys.md"
"go:agent-settings" = "go:settings"
"go:rotate-keys" = "security/keys.md#rotate-keys"

[retired]
"go:legacy-sync" = "The sync agent was removed in 4.0"
"reference/legacy-sync.md" = "The sync agent was removed in 4.0"
```

- **`[moved]`:** where each key's addresses lead now. A key is a page's path from the content root, which stands for every address the page was published at, its Markdown version's too; a file's path, `#`, and a heading's id, with the file the `@id` is written in, a fragment's once for every page that includes it; `go:` and a [permalink](../permalinks/index.md)'s name, for one no heading or page declares any more; or an address from the host's root, starting with `/`, from before the project used Ascribe. A value is a page's path, and optionally `#` and a heading's id, or, for a `go:` key, `go:` and another permalink's name, which makes the key an alias of it ([when a section moves or goes away](../permalinks/index.md#moving-aliasing-and-retiring)).
- **`[retired]`:** the addresses removed on purpose, each key as `[moved]`'s, with why. A reader who follows an old link gets [a page that says what was removed and why](#a-retired-addresss-page), answering 404; a retired permalink's address shows the reason.
- **`[former]`:** the routing settings the docs were published under before, each oldest first: `base-path`, `permalink-path`, and `agents`, each a list of values, and `builds`, a table from a build's old name to its new one. [`ascribe check --fix` writes it](#a-settings-change) when a setting changes.

Its format is in [SPEC §7.10]({repo}/blob/main/SPEC.md#710-the-addresses-file).

## What a move records

`ascribe rename`, and moving a page in the editor, record the move in the same change as the links they update:

```console
$ ascribe rename content/guides/my-setup.md content/guides/setup.md
moved content/guides/my-setup.md to content/guides/setup.md
recorded the move in ascribe.addresses
```

```toml
[moved]
"guides/my-setup.md" = "guides/setup.md"
```

Moving a page back to where an entry says it was removes the entry. A page moved another way, as with `git mv`, is recorded with [`ascribe addresses move`](#cli-addresses-move), and a page removed on purpose with [`ascribe addresses retire`](#cli-addresses-retire).

## What the check reports

With the file, `ascribe check` lists every address the base published, in each build, and every address the working tree publishes, and reports what the base had that nothing accounts for. A build's addresses are each page's, its Markdown version's, each heading's explicit `@id`, each [permalink](../permalinks/index.md)'s under the permalinks' path, and each `llms.txt` file's. An address is accounted for when the working tree still publishes it, when an entry of `[moved]` leads it to a page a build publishes, when `[retired]` has it, or when an availability mark takes its page out of one build while another build publishes it.

- **[`address-lost`](../checks/diagnostics/project-files.md#ASC255)**, a warning: a page's address, or its Markdown version's, once for each page, listing its addresses and, when it isn't every build, the builds. Forward it with `ascribe addresses move`, or retire it with `ascribe addresses retire`.
- **[`anchor-lost`](../checks/diagnostics/project-files.md#ASC256)**, advice: a heading's explicit `@id` the base published on a page still published, which the page doesn't have now. A link to it lands at the top of the page, so it's advice for now. A generated id isn't kept: it changes with the heading's text.
- **[`addresses-build-gone`](../checks/diagnostics/project-files.md#ASC258)**, advice: a build the base has and `ascribe.toml` doesn't, whose addresses aren't compared. A line's build isn't reported once `[releases]` archives the line: it left on purpose, and [`ascribe versions retire`](../versions/archive.md#retire-a-line) records where its addresses go.

```console
$ git rm content/guides/teams/share-a-loom.md && ascribe check --base main --deny-warnings
[ASC255] Warning: guides/teams/share-a-loom.md was published at /docs/guides/teams/share-a-loom and /docs/guides/teams/share-a-loom.md, and nothing publishes them now
```

Each is reported at `ascribe.addresses`. The comparison runs when `ascribe check` checks the whole project, not for a page or a file named. In the editor, see [below](#in-the-editor).

## In the editor

The language server compares the addresses with the base's between checks, not as you type in a page: when the project loads, after a save, after a change on disk, as a page deleted in the Explorer or a `git checkout` is, and as you edit `ascribe.addresses`. Delete a page, and within seconds its lost addresses are in the Problems panel at `ascribe.addresses`, as [Check Every Build](../editor/problems.md#the-problems-panel) would put them, with their quick fixes, and the Project view's Health counts them under [Against the base](../editor/sidebar.md#addresses-lost). Check Every Build replaces them with its own when it runs. The server keeps the base's addresses for each base commit and version of Ascribe, so a comparison after the first doesn't read the base again.

Moving a page in the editor puts its entry in the same edit as the links it updates, unsaved in `ascribe.addresses` until you save: an entry you haven't saved counts, and **Undo** before you save takes back the move and its entry together. A page moved back after its entry is saved, as `git mv` would, leaves the entry, which [`addresses-entry-live`](../checks/diagnostics/project-files.md#ASC266) reports. On a page that old addresses lead to, This page's About lists each under [Also answers at](../editor/sidebar.md#old-addresses).

## Fix a lost address

Each lost address has a fix, an edit to `ascribe.addresses`, offered by the check, by `ascribe check --fix` where it's safe, and as quick fixes on the problem in the editor:

- **Record git's renames,** when git sees the page renamed, as `git mv` leaves it: the move is recorded as `ascribe addresses move --from-git` records it. It's safe, since git's rename is the decision, so `ascribe check --fix` applies it.
- **Forward it to…** the page most like it, by its headings and words as the base had them, and the next two most like it: `ascribe addresses move <old> <new>`. For a heading's id, the same id on another page. It's a choice, so it's never applied for you.
- **Retire it, and write why:** the entry, with a placeholder for its reason, which is reported until you replace it.

```console
$ git mv content/guides/setup.md content/start/setup.md && git rm -q content/reference/options.md
$ ascribe check --base main
[ASC255] Warning: guides/setup.md was published at /docs/guides/setup and /docs/guides/setup.md, and nothing publishes them now
   │ Help: git sees it renamed to start/setup.md: record git's renames, as `ascribe addresses move --from-git` does
[ASC255] Warning: reference/options.md was published at /docs/reference/options and /docs/reference/options.md, and nothing publishes them now
   │ Help: Forward it to the page most like it, guides/configure.md: `ascribe addresses move reference/options.md guides/configure.md`
$ ascribe addresses move --from-git
Recorded 1 move git sees: guides/setup.md → start/setup.md.
```

`--format json` lists every fix of each problem, with its edits.

## The file's checks

With the base read, `ascribe check` also checks that each entry of `ascribe.addresses` still does what it says. Each is reported at its entry:

| Diagnostic | Level | When |
|---|---|---|
| [`addresses-entry-not-a-page`](../checks/diagnostics/project-files.md#ASC265) | Error | A `[moved]` value isn't a page, or a heading on one, or a build that published the key doesn't publish it |
| [`addresses-entry-live`](../checks/diagnostics/project-files.md#ASC266) | Advice | The key's source is back, as after a page is restored; the fix removes the entry |
| [`addresses-entry-taken`](../checks/diagnostics/project-files.md#ASC267) | Error | Another page publishes an address the key had, so an old link lands on it |
| [`addresses-entry-unpublished`](../checks/diagnostics/project-files.md#ASC268) | Advice | The base never published the key, and its file doesn't have the entry, as moving a page made on the same branch leaves; the fix removes it |
| [`addresses-key-twice`](../checks/diagnostics/project-files.md#ASC269) | Error | A key is in the file twice, as a merge can leave; the file isn't used until one goes |
| [`addresses-chain`](../checks/diagnostics/project-files.md#ASC270) | Advice | A `[moved]` value is another entry's key; the build writes where the chain ends, and the fix forwards it there |
| [`addresses-anchor-collision`](../checks/diagnostics/project-files.md#ASC271) | Error | An id from a page that's gone is live on the page its page's addresses lead to, so a link to it lands there |
| [`addresses-file-deleted`](../checks/diagnostics/project-files.md#ASC272) | Error | `ascribe.addresses` is gone, and the base has it; read only when `--base` names the base, as CI does |
| [`addresses-soft-404`](../checks/diagnostics/project-files.md#ASC273) | Advice | Three addresses or more are forwarded to one index page or the home page |

Retiring a page that entries lead to retires them with it, for the same reason, and moving a page takes the entries that lead to its headings with it.

To stop keeping the addresses on purpose, delete the file in a pull request of its own, whose check compares with its own head, `ascribe check --base HEAD`.

## A settings change

Changing a setting that every address is made with moves every address at once: `base-path`, `permalink-path`, or `agents` in `[consumer]`, or a build's name. The check says so once, as `address-lost`'s settings message, naming the setting, its old and new values, and how many addresses it moved, in place of a warning for each page. Its fix records the old value in `[former]`. It's safe, since changing the setting is the decision, so `ascribe check --fix` applies it:

```console
$ ascribe check --base main
[ASC255] Warning: base-path changed from "/docs/" to "/handbook/", so 14 addresses moved, 5 of them permalinks
   │ Help: Record the settings change: `ascribe check --fix`
$ ascribe check --fix && ascribe build
```

```toml
[former]
base-path = ["/docs/"]
```

The build then forwards every address under each former value to the same source's address today, from `ascribe.addresses` alone, so it keeps doing so once the base has moved past the change:

- **`base-path`:** each page's address, its Markdown version's, each `llms.txt` file's, and each permalink's while `permalink-path` is its default, `go/` under the base path. A `[moved]` entry's old addresses are forwarded under each former base path too.
- **`permalink-path`:** each permalink's address.
- **`agents`,** turned off: `agents = [true]` forwards each Markdown version to its page, and each `llms.txt` file to the page at the base path.
- **`builds`:** the build of the new name is compared with the base's build of the old one. A build is taken for renamed when the base's build is gone and a new build publishes more than half of its pages.

A second change composes with the first: under `base-path = ["/docs/", "/guide/"]`, an address under either forwards straight to today's, in one step. A page lost in the same change, as one deleted with it, is still a warning of its own. `trailing-slash` isn't kept: a host matches an address with or without its final `/`.

## The base

The base is what the docs published before: the merge base of the repository's default branch and `HEAD`, as a pull request shows its changes, or the revision `--base` names. `--base-exact` compares with that revision itself. It's read as [`ascribe diff`](../review/diff.md) reads it, so the checkout needs the history: in CI, `fetch-depth: 0`, as [Checks in CI](../checks/ci.md#keep-the-addresses-youve-published) sets up.

When the base can't be read, because the revision isn't there, as in a shallow clone, or there's history but no default branch to compare with, or `--base` names a base outside a repository, the rest of the check runs and [`addresses-base-unreadable`](../checks/diagnostics/project-checks.md#ASC257), a warning, says why, so a check that compared nothing never passes silently. A base with no project, as a project's first pull request has, published nothing to keep, and that's advice. Before the first commit, with no `--base`, nothing can have been published: a project outside a git repository, or in one with no commit yet, compares nothing, and the check doesn't say so, so a project `ascribe init` just started checks clean.

## What the build writes

Each build writes `_ascribe/redirects.json` in its site output, from `ascribe.addresses` alone: for each `[moved]` entry, a `301` from each address the key had, the page's and its Markdown version's, to where the value is now; for each `[former]` value, a `301` from each address under it to today's, as [a settings change](#a-settings-change) says; and for each `[retired]` entry, its addresses and the reason. The base isn't read, so a redirect stays once the base has moved past the move. Its format is in the [output-layout contract](../contracts/output-layout.md#18-the-redirects).

```console
$ ascribe addresses list
site:
  /docs/guides/my-setup -> /docs/guides/setup (301, guides/my-setup.md)
  /docs/guides/my-setup.md -> /docs/guides/setup.md (301, guides/my-setup.md)
```

**On Netlify,** with `host = "netlify"` in `[consumer]`, each build writes Netlify's `_redirects` too, and `@ascribed/astro` places it at the root of the built site, ahead of a site's own `public/_redirects` ([the host's files](site.md#the-hosts-files)):

```text
/docs/guides/my-setup /docs/guides/setup 301
/docs/guides/my-setup.md /docs/guides/setup.md 301
```

The same file forwards each [permalink](../permalinks/index.md)'s address, as [below](#permalinks-on-netlify).

**On another host,** read `_ascribe/redirects.json` after the build and give each pair to the host's own configuration.

## A retired address's page

A reader who follows a link to a retired address gets a page that says what was there was removed, and why, with a link to the docs' home, answering 404, so search engines drop it. `@ascribed/astro` adds one for each address in `redirects.json`'s `gone`, marked `noindex`, in `_ascribe/gone/` under the build's base path, at the address from there, and the build's Netlify `_redirects` serves it at the old address:

```text
/docs/reference/legacy-sync /docs/_ascribe/gone/reference/legacy-sync 404
```

Another site generator reads `gone` from `_ascribe/redirects.json` and makes its own page for each.

## Permalinks on Netlify
@id: permalinks-on-netlify
@available: ascribe planned

With `host = "netlify"`, the `_redirects` the build writes forwards each permalink's address with a `301`, straight to its page and heading, so a fetcher that doesn't follow the forwarding page's refresh, as `curl` and most agents' tools don't, still lands there in one step. Netlify keeps the heading's anchor. The forwarding page stays, for a host without redirects:

```text
/docs/go/connect/ pm=:pm /docs/go/connect/ 200!
/docs/go/connect/ /docs/install-agent/#connect 301!
/docs/go/legacy-sync/ /docs/go/legacy-sync/ 410!
```

- **An address with a [dimension](../content-model/variation.md#dimensions)'s parameter,** such as `?pm=pnpm`, gets the forwarding page instead, whose script sets the reader's choice before it forwards: a line for each dimension, ahead of the `301`, since Netlify uses the first rule that matches.
- **A retired permalink's address** serves its page, which says why it was removed, with a `410`, which tells a fetcher it's gone on purpose.
- **`!`** forces each rule past the forwarding page at the address, which Netlify would serve otherwise.

The lines come after the moves and the retired addresses, and the [output-layout contract](../contracts/output-layout.md#19-the-hosts-files) has their format. On another host, read `_ascribe/permalinks.json` and write the same rules for it.

## `ascribe addresses`
@id {permalink=true}: cli-addresses
@summary: Keeps the project's record of where the addresses it published went, `ascribe.addresses`: makes it, records a move or a retired address, and lists the redirects each build writes.

Keeps `ascribe.addresses`. Each takes `--format json`, and writes the document its [schema](../contracts/json-reports/addresses.md#ascribe-addresses-init) describes. Each exits with `0` when it's done, and `2` when it couldn't run: the project doesn't load, or the file can't be read or written.

### `ascribe addresses init`
@id {permalink=true}: cli-addresses-init
@summary: Makes `ascribe.addresses`, so moves are recorded and the check reports an address that's lost. A file already there is left alone.

Makes `ascribe.addresses`, with its header, its `version`, and empty tables. A file already there is left as it is, and it says so.

In a project whose `ascribe.toml` still has `[permalinks]`, from before `ascribe.addresses` recorded permalinks, it moves the table's entries into the file, made or already there, and takes the table out of `ascribe.toml`, keeping the rest of the file as written: each of `[permalinks.retired]` as `go:<name>` in `[retired]`, or in `[moved]` to its `to` when it has one, and each of `[permalinks.aliases]` as `go:<name>` in `[moved]` to `go:` and the name it leads to. Run `ascribe permalinks sync` after, when the project has `ascribe.permalinks`. A base whose `ascribe.toml` still has the table is read as if it had been moved, so the move itself loses no address.

@include: ../_generated/cli-addresses-init-options.md

### `ascribe addresses move`
@id {permalink=true}: cli-addresses-move
@summary: Records that a page, or a heading's id, moved, after a move made without Ascribe, so the build forwards its old addresses.

Records that a page, or a heading's id, moved, after a move made without Ascribe, as with `git mv`. The old path is gone; the new one is a page of the project, or a heading on one. A permalink no heading declares any more, `go:<name>`, moves to a page or a heading, or to another permalink, `go:<name>`.

```console
$ ascribe addresses move guides/keys.md security/keys.md
Recorded in ascribe.addresses: guides/keys.md moved to security/keys.md.
```

Each page's path is from the current folder or from the content root, and the entry is from the content root. So, from a repository's root whose project is in `docs/` with its content in `docs/content/`, `ascribe addresses move --config docs docs/content/guides/keys.md docs/content/security/keys.md` records the same entry, and so does `ascribe addresses retire` given such a path.

**`--from-git`** records every move git sees between the base and the working tree whose old path is gone, one entry for each page, so a reorganization made with `git mv` is recorded at once. The base is chosen as `ascribe check` chooses it, with `--base` and `--base-exact`. Git sees a file that's renamed and added, as `git mv` leaves it, and only when the old and new files are mostly the same: a page rewritten as it moved is forwarded with `ascribe addresses move <old> <new>`.

```console
$ ascribe addresses move --from-git
Recorded 1 move git sees: guides/setup.md → start/setup.md.
```

@include: ../_generated/cli-addresses-move-options.md

### `ascribe addresses retire`
@id {permalink=true}: cli-addresses-retire
@summary: Records that a page, or a heading's id, was removed on purpose, and why.

Records that a page, or a heading's id, or a permalink, `go:<name>`, was removed on purpose, with the reason, which is for whoever reads the file later.

```console
$ ascribe addresses retire reference/legacy-sync.md --reason "The sync agent was removed in 4.0"
Recorded in ascribe.addresses: reference/legacy-sync.md was retired.
```

@include: ../_generated/cli-addresses-retire-options.md

### `ascribe addresses list`
@id {permalink=true}: cli-addresses-list
@summary: Shows the redirects and retired addresses each build writes, and the entry each comes from. It changes nothing.

Shows the redirects and retired addresses each build writes, as its `_ascribe/redirects.json` has them, and the entry of `ascribe.addresses` each comes from. It changes nothing.

@include: ../_generated/cli-addresses-list-options.md
