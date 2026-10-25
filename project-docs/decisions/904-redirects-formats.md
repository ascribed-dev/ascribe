---
section: The binary
---

**Redirects ship five shapes, each a contract from the phase that ships it: `ascribe.addresses`, a TOML file beside `ascribe.toml` with `version`, `[moved]`, `[former]`, and `[retired]`; `_ascribe/redirects.json` in every build's site output, with `version`, `build`, `redirects`, and `gone`; `[consumer] host = "netlify"` in place of `headers`, with the compiler writing the host's files and `@ascribed/astro` placing them; fourteen diagnostics, `address-lost` a warning and `anchor-lost` advice beside it; and the commands `ascribe addresses init`, `move`, `retire`, and `list`, `ascribe check --base` and `--base-exact`, and MCP tools that return the edit and write nothing.**

**`ascribe.addresses`** is read through `FileSystem`:

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

A `version` it doesn't know is an error naming a newer Ascribe, as `ascribe.permalinks`'s is. A `[moved]` key is a source path from the content root, a heading's id as `<file>#<id>` with the file where the `@id` is written (a fragment's, once), or an address from the host's root starting with `/`, from before the project used Ascribe; a value is a source path or `<file>#<id>`, and phase 2 enforces what a value may be. `[former]` has `base-path`, `permalink-path`, and `agents`, each a list of the values the docs were published under, oldest first, and `builds`, a table from a build's old name to its new one; it has no `trailing-slash`. A `[retired]` key is as `[moved]`'s, and its value a reason, checked for the placeholder as `[permalinks.retired]`'s are. The file is written whole, each table sorted, one entry per line, by the commands and the edits; a comment a person adds isn't kept, but the header is, and a table or key it doesn't know is an error.

**`_ascribe/redirects.json`** is written for every build, whether or not `ascribe.addresses` exists, as `headers.json` is, and is `generated` in the manifest:

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

Its paths are from the host's root, as `headers.json`'s are, in the published case and slash form, sorted by `from` and by `path`. `status` is 301 in every pair the plan writes, and is in the format so a later one can differ. `entry` is the `ascribe.addresses` key the pair or the address comes from, for `ascribe addresses list` and This page. A later version adds keys and changes none of these.

**`[consumer] host = "netlify"`** is the key's one value. `headers = "netlify"` becomes an error whose fix is `host = "netlify"`. Only when `host` is set, the compiler writes `_ascribe/host/_redirects` and `_ascribe/host/_headers` into the site output, `generated` in the manifest, and a site built another way copies them to its root. `@ascribed/astro` reads those files, not the JSON, and writes the site root's `_redirects` and `_headers`, every project's merged. A site's own `public/_redirects` is merged after Ascribe's rules, since the first match wins; a site's own `public/_headers` keeps today's rule, failing the build until it has Ascribe's lines. Under an Astro base that isn't `/`, the integration writes neither and says so, as today.

**The diagnostics,** whose codes are given as each is added:

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

**The commands.** `ascribe addresses init` makes the file, with its header, `version`, and empty tables, and leaves a file already there alone, saying so. `ascribe addresses move <old> <new>` records an entry for a move already made, as after `git mv`; `--from-git` (phase 2) records every rename git sees. `ascribe addresses retire <old> --reason "…"` records a retired address. `ascribe addresses list` lists every redirect and gone address each build writes, and the entry each comes from, with `--build` for one. `ascribe check` takes `--base <rev>` and `--base-exact`, as `ascribe diff` does. Each takes `--format json`, with a schema of its own, as every command does. The MCP tools (phase 2), `ascribe_addresses_move` and `ascribe_addresses_retire`, take the commands' arguments under the same names and, as every tool, write no file: each returns the edit to `ascribe.addresses` for the agent to apply.

**Also settled:** phase 1 compares each page's address, its Markdown version's, and each explicit `@id`, in every build matched by name, and phase 3 adds `go/` permalinks and `llms.txt`. `ascribe rename` records a move only when `ascribe.addresses` exists, and says it recorded one.

_Why:_ each shape is a contract (the [redirects plan](https://github.com/ascribed-dev/ascribe/blob/638865426913dd00580d23b24a897775b80c096d/project-docs/redirects/README.md)'s decision 11), and one that ships can't change, so none is chosen while building. The file follows `ascribe.permalinks`, the pattern the plan builds on: a version, tables written whole and sorted, a reason checked for the placeholder, and unknown keys an error, since before 1.0 an addition an older Ascribe rejects is as every file is. `trailing-slash` isn't in `[former]` because the host test showed a rule matches either slash form, so a change of it moves no address a host can't find. `redirects.json` is written for every build and its integration reads the host's files rather than the JSON, as `headers.json` and `_headers` work, so a site built without Astro gets the same files to copy. Ascribe's `_redirects` rules go first because Netlify uses the first rule that matches. `headers` becomes an error rather than an alias because before 1.0 nothing keeps an old key. `anchor-lost` is advice beside `address-lost` until anchors reach a reader (the plan's decision 7), and a lost address is one warning per source (its decision 6). The MCP tools write nothing because no tool does: the agent applies the edit. _From:_ Kyle, 25 October 2026, accepting every recommendation of [phase 0 of the redirects plan](https://github.com/ascribed-dev/ascribe/blob/638865426913dd00580d23b24a897775b80c096d/project-docs/redirects/phase-0-formats.md) as written, on [issue #904](https://github.com/ascribed-dev/ascribe/issues/904); phase 1 builds from it.
