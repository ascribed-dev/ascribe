---
type: how-to
title: Keep the addresses you've published
description: "Record where a page went when it moves or goes, so a link to its old address still leads somewhere: ascribe.addresses, the check that compares with the base, and the redirects the build writes."
available: ascribe planned
---

A link to a page outlives the page's place in the docs: a bookmark, a search result, a product's help menu, an answer on a forum. Rename a folder or merge two pages, and every one of those links stops working unless someone decided where it should lead. `ascribe.addresses` is where that decision is written. With it, a move records itself, `ascribe check` reports an address the docs published that nothing accounts for, and the build writes the redirects the host needs.

## Turn it on

Make the file beside `ascribe.toml`, and commit it with the docs:

```console
$ ascribe addresses init
Wrote ascribe.addresses. Moves are recorded from now on.
```

A project without the file records nothing, and the check compares nothing, so turning it on is the decision to keep the addresses.

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

[retired]
"reference/legacy-sync.md" = "The sync agent was removed in 4.0"
```

- **`[moved]`:** where each key's addresses lead now. A key is a page's path from the content root, which stands for every address the page was published at, its Markdown version's too; a file's path, `#`, and a heading's id, with the file the `@id` is written in, a fragment's once for every page that includes it; or an address from the host's root, starting with `/`, from before the project used Ascribe. A value is a page's path, and optionally `#` and a heading's id.
- **`[retired]`:** the addresses removed on purpose, each key as `[moved]`'s, with why. A reader who follows an old link gets the host's not-found page.

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

With the file, `ascribe check` lists every address the base published, in each build, and every address the working tree publishes, and reports what the base had that nothing accounts for. An address is accounted for when the working tree still publishes it, when an entry of `[moved]` leads it to a page a build publishes, when `[retired]` has it, or when an availability mark takes its page out of one build while another build publishes it.

- **[`address-lost`](../checks/diagnostics/pages.md#ASC255)**, a warning: a page's address, or its Markdown version's, once for each page, listing its addresses and, when it isn't every build, the builds. Forward it with `ascribe addresses move`, or retire it with `ascribe addresses retire`.
- **[`anchor-lost`](../checks/diagnostics/pages.md#ASC256)**, advice: a heading's explicit `@id` the base published on a page still published, which the page doesn't have now. A link to it lands at the top of the page, so it's advice for now. A generated id isn't kept: it changes with the heading's text.
- **[`addresses-build-gone`](../checks/diagnostics/pages.md#ASC258)**, advice: a build the base has and `ascribe.toml` doesn't, whose addresses aren't compared.

```console
$ git rm content/guides/teams/share-a-loom.md && ascribe check --base main --deny-warnings
[ASC255] Warning: guides/teams/share-a-loom.md was published at /docs/guides/teams/share-a-loom and /docs/guides/teams/share-a-loom.md, and nothing publishes them now
```

Each is reported at `ascribe.addresses`. The comparison runs when `ascribe check` checks the whole project, not for a page or a file named, and not in the editor.

## The base

The base is what the docs published before: the merge base of the repository's default branch and `HEAD`, as a pull request shows its changes, or the revision `--base` names. `--base-exact` compares with that revision itself. It's read as [`ascribe diff`](../review/diff.md) reads it, so the checkout needs the history: in CI, `fetch-depth: 0`, as [Checks in CI](../checks/ci.md#keep-the-addresses-youve-published) sets up.

When the base can't be read, because there's no repository, or the revision isn't there, as in a shallow clone, the rest of the check runs and [`addresses-base-unreadable`](../checks/diagnostics/content-checks.md#ASC257), a warning, says why, so a check that compared nothing never passes silently. A base with no project, as a project's first pull request has, published nothing to keep, and that's advice.

## What the build writes

Each build writes `_ascribe/redirects.json` in its site output, from `ascribe.addresses` alone: for each `[moved]` entry, a `301` from each address the key had, the page's and its Markdown version's, to where the value is now, and for each `[retired]` entry, its addresses and the reason. The base isn't read, so a redirect stays once the base has moved past the move. Its format is in the [output-layout contract](../contracts/output-layout.md#18-the-redirects).

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

**On another host,** read `_ascribe/redirects.json` after the build and give each pair to the host's own configuration.

## `ascribe addresses`
@id {permalink=true}: cli-addresses
@summary: Keeps the project's record of where the addresses it published went, `ascribe.addresses`: makes it, records a move or a retired address, and lists the redirects each build writes.

Keeps `ascribe.addresses`. Each takes `--format json`, and writes the document its [schema](../contracts/json-reports/sources.md#ascribe-addresses-init) describes. Each exits with `0` when it's done, and `2` when it couldn't run: the project doesn't load, or the file can't be read or written.

### `ascribe addresses init`
@id {permalink=true}: cli-addresses-init
@summary: Makes `ascribe.addresses`, so moves are recorded and the check reports an address that's lost. A file already there is left alone.

Makes `ascribe.addresses`, with its header, its `version`, and empty tables. A file already there is left as it is, and it says so.

@include: ../_generated/cli-addresses-init-options.md

### `ascribe addresses move`
@id {permalink=true}: cli-addresses-move
@summary: Records that a page, or a heading's id, moved, after a move made without Ascribe, so the build forwards its old addresses.

Records that a page, or a heading's id, moved, after a move made without Ascribe, as with `git mv`. The old path is gone; the new one is a page of the project, or a heading on one.

```console
$ ascribe addresses move guides/keys.md security/keys.md
Recorded in ascribe.addresses: guides/keys.md moved to security/keys.md.
```

@include: ../_generated/cli-addresses-move-options.md

### `ascribe addresses retire`
@id {permalink=true}: cli-addresses-retire
@summary: Records that a page, or a heading's id, was removed on purpose, and why.

Records that a page, or a heading's id, was removed on purpose, with the reason, which is for whoever reads the file later.

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
