---
type: reference
title: "`ascribe addresses`"
description: "Keep `ascribe.addresses`: make it, record a move or a retired address, and list the redirects each build writes, with each command's options and output."
---

`ascribe addresses` keeps `ascribe.addresses`, the record of where the addresses the docs published went: it makes the file, records a move or a retired address, and lists the redirects each build writes. [Keep the addresses you've published](redirects.md) is the task the commands are used in.

@include: ../_fragments/command-options.md

## `ascribe addresses`
@id {permalink=true}: cli-addresses
@summary: Keeps the project's record of where the addresses it published went, `ascribe.addresses`: makes it, records a move or a retired address, and lists the redirects each build writes.

Keeps `ascribe.addresses`. Each takes `--format json`, and writes the document its [schema](../contracts/json-reports/addresses.md#ascribe-addresses-init) describes. Each exits with `0` when it's done, and `2` when it couldn't run: the project doesn't load, or the file can't be read or written.

### `ascribe addresses init`
@id {permalink=true}: cli-addresses-init
@summary: Makes `ascribe.addresses`, so moves are recorded and the check reports an address that's lost. A file already there is left alone.

Makes `ascribe.addresses`, with its header, its `version`, and empty tables. A file already there is left as it is, and it says so.

In a project whose `ascribe.toml` still has `[permalinks]`, from before `ascribe.addresses` recorded permalinks, it moves the table's entries into the file, made or already there, and takes the table out of `ascribe.toml`, keeping the rest of the file as written. Each of `[permalinks.retired]` goes in `[retired]` as `go:<name>`, or in `[moved]` to its `to` when it has one. Each of `[permalinks.aliases]` goes in `[moved]` as `go:<name>`, leading to `go:` and the name it leads to. Run `ascribe permalinks sync` after, when the project has `ascribe.permalinks`. A base whose `ascribe.toml` still has the table is read as if it had been moved, so the move itself loses no address.

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

## See also

- [Keep the addresses you've published](redirects.md): what the check reports, how to fix a lost address, and what the build writes.
