# Phase 0: Decide the formats

Part of [Redirects](README.md). Needs nothing. A decision by the maintainer; no code.

## Goal

The maintainer settles the shapes phase 1 ships, so none is chosen while building: `ascribe.addresses`'s grammar, `_ascribe/redirects.json`, `[consumer] host` and where the host's files go, the diagnostics' names and levels, and the commands. Each is a contract (README decision 11), and a shape that ships can't change.

The proposal settled what each holds. This settles how each is spelled.

## The recommendation

### `ascribe.addresses`

TOML, beside `ascribe.toml`, read through `FileSystem`:

```toml
# Where addresses the docs published went. Written by `ascribe rename`, the
# editor, `ascribe addresses`, and `ascribe check --fix`; edit only
# [retired]'s reasons.
version = 1

[moved]
"/docs/intro.html" = "getting-started.md"
"guides/install.md#install-with-npm" = "guides/install.md#install-npm"
"guides/keys.md" = "security/keys.md"

[former]
base-path = ["/docs/"]

[retired]
"reference/legacy-sync.md" = "The sync agent was removed in 4.0"
```

- **`version = 1`.** A version it doesn't know is an error naming a newer Ascribe, as `ascribe.permalinks`'s is.
- **`[moved]`:** a key is a source path from the content root, a heading's id as `<file>#<id>` with the file where the `@id` is written (a fragment's, once), or an address from the host's root starting with `/`, from before the project used Ascribe. A value is a source path or `<file>#<id>`. Phase 2 enforces what a value may be.
- **`[former]`:** `base-path`, `permalink-path`, and `agents`, each a list of the values the docs were published under, oldest first, and `builds`, a table from a build's old name to its new one. `trailing-slash` isn't one: the host test showed a rule matches either slash form, so a change of it moves no address a host can't find.
- **`[retired]`:** a key as `[moved]`'s, and a reason, checked for the placeholder as `[permalinks.retired]`'s are.
- **Written whole,** each table sorted, one entry per line, by the commands and the edits; a comment a person adds, but the header, isn't kept, as `ascribe.permalinks` keeps none. A table or key it doesn't know is an error; the later proposal's `[lines.*]` and folder keys are additions an older Ascribe rejects, which before 1.0 is as every file is.

### `_ascribe/redirects.json`

In every build's site output, `generated` in its manifest, written for every build whether or not `ascribe.addresses` exists, as `headers.json` is:

```json
{
  "version": 1,
  "build": "site",
  "redirects": [
    { "from": "/docs/guides/keys/", "to": "/docs/security/keys/", "status": 301, "entry": "guides/keys.md" }
  ],
  "gone": [
    { "path": "/docs/reference/legacy-sync/", "reason": "The sync agent was removed in 4.0", "entry": "reference/legacy-sync.md" }
  ]
}
```

- **Paths from the host's root,** as `headers.json`'s are, in the published case and slash form, sorted by `from` and by `path`.
- **`status`** is 301 in every pair this plan writes; it's in the format so a later one can differ.
- **`entry`** is the `ascribe.addresses` key the pair or the address comes from, for `ascribe addresses list` and This page.
- **A later version adds keys and changes none of these:** the later proposal's `anchors`.

### `[consumer] host`

- **`host = "netlify"`,** its one value, in place of `headers = "netlify"`. `headers` becomes an error whose fix is `host = "netlify"`: before 1.0, nothing keeps the old key.
- **The compiler writes the host's files** into the site output, at `_ascribe/host/_redirects` and `_ascribe/host/_headers`, `generated` in the manifest, only when `host` is set. A site built another way copies them to its root.
- **`@ascribed/astro`** reads those files, not the JSON, and writes the site root's `_redirects` and `_headers`, every project's merged. A site's own `public/_redirects` is merged, Ascribe's rules first, since the first match wins; a site's own `public/_headers` keeps today's rule, failing the build until it has Ascribe's lines. Under an Astro base that isn't `/`, it writes neither and says so, as today.

### The diagnostics

Names and levels; codes are given when each is added.

| Slug | Level | Phase | When |
|---|---|---|---|
| `addresses-file-invalid` | error | 1 | `ascribe.addresses` can't be read, or doesn't have the shape above |
| `address-lost` | warning | 1 | A page's address, or its Markdown version's, that the base published and nothing accounts for; one per source |
| `anchor-lost` | advice | 1 | The same for an explicit `#id`, until anchors reach a reader |
| `addresses-base-unreadable` | warning | 1 | The base can't be read, so nothing was compared |
| `addresses-build-gone` | advice | 1 | A build the base has and the working tree doesn't |
| `addresses-entry-not-a-page` | error | 2 | A value that isn't a page, or a heading on one, in every build that published the key |
| `addresses-entry-live` | advice | 2 | A key whose source is back |
| `addresses-entry-taken` | error | 2 | A key whose address another source publishes now |
| `addresses-entry-unpublished` | advice | 2 | An entry the base's file doesn't have, whose key the base never published |
| `addresses-key-twice` | error | 2 | A key in a table twice, or in two tables |
| `addresses-chain` | advice | 2 | An entry whose value is another entry's key |
| `addresses-anchor-collision` | error | 2 | An `#id` entry from a page that's gone whose id is live on its target |
| `addresses-file-deleted` | error | 2 | `ascribe.addresses` gone while the base has it |
| `addresses-soft-404` | advice | 2 | Many addresses forwarded to one index or the home page |

A settings change (phase 3) is a message of `address-lost`'s, not a slug of its own: one warning for every address a setting moved.

### The commands

- **`ascribe addresses init`:** makes the file, with its header, `version`, and empty tables. A file already there is left alone, and the command says so.
- **`ascribe addresses move <old> <new>`:** records an entry for a move already made, as after `git mv`. `--from-git` (phase 2) records every rename git sees.
- **`ascribe addresses retire <old> --reason "…"`.**
- **`ascribe addresses list`:** every redirect and gone address each build writes, and the entry each comes from; `--build` for one.
- **`ascribe check --base <rev>` and `--base-exact`,** as `ascribe diff`'s.
- **`--format json`** on each, with a schema of its own, as every command's.
- **The MCP tools (phase 2),** `ascribe_addresses_move` and `ascribe_addresses_retire`, take the commands' arguments under the same names, and, as every tool, write no file: each returns the edit to `ascribe.addresses` for the agent to apply.

## Also settled here

- **What's compared, in phase 1:** each page's address, its Markdown version's, and each explicit `@id`, in every build matched by name. Phase 3 adds `go/` permalinks and `llms.txt`.
- **`ascribe rename` records a move** only when `ascribe.addresses` exists, and says it recorded one.

## Output

A decision file in `project-docs/decisions/`, numbered by this phase's issue, with each shape above, as settled, and why. Phase 1 starts from it. Nothing else changes.
