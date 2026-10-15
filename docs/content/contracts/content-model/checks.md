---
type: reference
title: "`ascribe.toml` format contract: checks, permalinks, and listing"
description: "The normative contract for the content model's checks, permalinks, crawlers, listing, and defaults."
---

Part of the [`ascribe.toml` format contract](../content-model.md), whose [contents](../content-model.md#contents) list every section.

## 19. `[checks]`
@available: ascribe 0.3.0

How loudly each check about the content's quality speaks. A diagnostic about whether the project is valid can't be named here: its severity is the registry's. The two forms a key's value takes are:

```toml
[checks]
code-language-missing = "warning"     # a level

[checks.page-size]                    # or a table: the level, and the check's own settings
level = "off"
limit = 40_000
```

| Key | Type | Default | Description |
|---|---|---|---|
| `<check>` | string or table | the check's own level | A check's slug, as the [diagnostics reference](../../checks/diagnostics.md) lists it. The value is a level: `"off"` (not reported), `"advice"`, `"warning"`, or `"error"`. Or it's a table with an optional `level` and the check's own settings, which the reference lists with the check; a table without `level` keeps the check's own level. |

Only a check the diagnostics registry marks configurable can be named; the [diagnostics reference](../../checks/diagnostics/content-checks.md) lists them. The settings a check's table takes:

| Check | Setting | Type | Default | Description |
|---|---|---|---|---|
| `page-size` | `limit` | integer | `50000` | The size, in characters of a page's plain Markdown in a build, at which the page is reported. It MUST be above 0. |
| `image-large` | `limit` | integer or string | `"500 KB"` | The size over which an image file is reported: a whole number of bytes, or a number followed by `B`, `KB`, or `MB` (a kilobyte is 1,000 bytes, a megabyte 1,000,000), with or without a space. |

The names `vale` and `links` are reserved: they're tables of settings for the tools Ascribe runs, not checks, and no check has either name. `vale` is §19.2, and `links` §19.3.

A level changes how a diagnostic is reported everywhere: by `ascribe check` and `ascribe build`, and in the editor. Advice is shown, and never fails `ascribe check`, even with `--deny-warnings`; a check set to `off` isn't reported at all.

**Rules** (§24.9): every key is a check's slug, with a did-you-mean suggestion for one that isn't (`model-unknown-key`); the check is configurable (`model-check-not-configurable`); a level is one of the four (`model-invalid-value`); a table has only `level` and the check's settings (`model-unknown-key`); a value is a string or a table, `page-size`'s `limit` a whole number above 0, and `image-large`'s a size (`model-wrong-type`).

### 19.1 `[[intended]]`
@available: ascribe 0.3.0

Acknowledges a review check's problem with a content model entry or an image as intended (SPEC §4.9): the problem isn't reported, and is counted and listed with the reason instead. Each table is one acknowledgement.

@include: ../../_fragments/content-model/intended-example.md

| Key | Type | Default | Description |
|---|---|---|---|
| `check` | string | required | A check's slug. The registry gives it `next = "review"` and `place = "entry"`. |
| `reason` | string | required | Why the problem is intended. Not empty. |
| `phrase` | string | | A phrase's key, declared in `[phrases]`. |
| `feature` | string | | A feature's key, declared in `[features]`. |
| `term` | string | | A glossary term's id, declared in `[glossary]`. |
| `image` | string | | An image's path under the content root. |

Exactly one of `phrase`, `feature`, `term`, and `image`. The acknowledgement covers the check's problems that name that entry, in every build. One that covers nothing when every build is checked is advice (`intended-unused`), reported at its `[[intended]]` header with a fix that removes the table.

**Rules** (§24.9): the check exists, with a did-you-mean suggestion for one that doesn't, is a review check, and is reported at an entry (`model-intended-check`); the table names exactly one declared entry (`model-intended-entry`); `check` and `reason` are present (`model-missing-key`) and `reason` isn't empty (`model-empty-text`); and the table has no other key (`model-unknown-key`).

### 19.2 `[checks.vale]`
@available: ascribe 0.3.0

How the prose is checked with Vale, a program the project installs. Ascribe runs it; it isn't linked or downloaded.

```toml
[checks.vale]
preset = "quiet"              # or: config = ".vale.ini"
off = ["Ascribe.Repeated"]    # rules of the preset to turn off
command = "vale"
in-check = false
max-level = "error"
```

| Key | Type | Default | Description |
|---|---|---|---|
| `preset` | string | | One of Ascribe's presets: `quiet`. A preset's name never changes once released; a rule added to it is a **Behavior change** in the changelog. |
| `config` | string | | The project's own `.vale.ini`, a path from the project root. |
| `off` | array of strings | `[]` | Rules of the preset to turn off, as `<Style>.<Rule>`. Only with `preset`. |
| `command` | string | `"vale"` | The command run. A value with a path separator is a path from the project root; a bare name is looked up on the `PATH`. A relative path, or an absolute one into the project's folder, is a program in the project, which the language server runs only once the person allows it, and the agents' hook never runs (decision 76). |
| `in-check` | boolean | `false` | Whether `ascribe check` checks the prose without `--vale`. |
| `max-level` | string | `"error"` | `"advice"`, `"warning"`, or `"error"`: the most a Vale alert is reported as. Vale's `suggestion`, `warning`, and `error` map to advice, warning, and error, capped at this. |

What Ascribe writes for Vale is under `.ascribe/vale/` in the project root: a preset's configuration and styles, and the vocabulary `Ascribe` of the project's own words (phrase values, glossary terms and aliases, dimension, value, and feature labels), one escaped regular expression to a line. Nothing outside it is written, except by `ascribe vale eject`.

Each alert is a `prose` diagnostic (`ASC165`) on the source file, with the rule as its JSON `rule`; one inside a phrase's text is at the phrase. A Vale that can't be run, fails, or runs out of time is one `prose-not-checked` advice (`ASC166`) at `[checks.vale]`, never a failed check, and so is one the caller doesn't start: the MCP server never starts Vale, the agents' hook never starts a program in the project, and the language server starts one only once the person allows it. Both are configurable in `[checks]`, and `[checks]` sets their level after `max-level` caps Vale's.

**Rules** (§24.9): exactly one of `preset` and `config` (`model-checks-vale`); `off` only with `preset`, and only naming its rules (`model-checks-vale`); `preset` and `max-level` take one of their values (`model-invalid-value`); no other key (`model-unknown-key`).

### 19.3 `[checks.links]`
@available: ascribe 0.3.0

How `ascribe report links` checks external links: with lychee, a program the project installs, which Ascribe runs with the addresses on its standard input. `ascribe check` never reaches the network.

```toml
[checks.links]
command = "lychee"
ignore = ["localhost", "*.internal.example.com"]
```

| Key | Type | Default | Description |
|---|---|---|---|
| `command` | string | `"lychee"` | The command run, in the project root. A value with a path separator is a path from the project root; a bare name is looked up on the `PATH`. |
| `ignore` | array of strings | `[]` | Hosts not checked. `name` matches that host; `*.name` matches every host that ends in `.name`, and not `name` itself. Matched without regard to case, after a trailing `.` is dropped. |
| `reuse-days` | integer | `14` | How many days an answer is reused. An address that answered with nothing to report isn't asked about again while its answer is younger than its own limit: a number of days from half of `reuse-days`, rounded up, through all of it, fixed by a hash of the address. `0` reuses nothing. |
| `confirm-days` | integer | `3` | How many days a failure is held. A link that fails, other than with `404` or `410`, whose address the history records answering before, is unconfirmed, not a diagnostic, until it has failed on every run for `confirm-days` days from its first failure since that answer. `0` holds nothing. |

Answers are kept in `.ascribe/cache/links/` in the project's folder, one file an address, keyed by the address exactly as lychee is given it and a hash of the project's `lychee.toml`. Only an answer with nothing to report is kept, so every diagnostic in a report comes from an address asked about in that run. A file that can't be read, has another version, or holds another key counts as no answer. `ascribe check`, `build`, `diff`, `drift`, `fmt`, the language server, and the agents' hook never read or write the folder.

The history is `.ascribe/history/links.json` in the project's folder: one file, version 1, sorted, written whole through a rename after each run, holding for each address the pages link to the last day it answered, the day it started failing, and whether the run reported it. An address no longer linked to is dropped. A run that reads the file again before writing it and finds another run's writing keeps, for each address, the record that confirms its failure later. A file that can't be read or has another version is no history, and with none, or with `--no-history`, every failure is a diagnostic. A permanent redirect is an answer. An `@intended {check=link-external-broken}` covers an unconfirmed link as it covers a reported one. `ascribe check`, `build`, `diff`, `drift`, `fmt`, the language server, and the agents' hook never read or write it.

The links checked are every link whose destination is an `http` or `https` URL, in every source file, each address once. A link whose address answers with an error, doesn't answer in time, or can't be reached is a `link-external-broken` advice (`ASC168`, `next = "review"`, `place = "block"`); one whose redirects begin with a `301` or `308` is a `link-external-moved` advice (`ASC169`, `next = "choose"`), with an unsafe fix that writes the address the permanent redirects end at. Both are at the link, and configurable in `[checks]`. An `@intended {check=link-external-broken}` that covers no broken link is `intended-unused` only when `ascribe report links` ran; `ascribe check` doesn't report it.

**Rules** (§24.9): each `ignore` entry is a host name, or `*.` and one (`model-wrong-type`); `command` is a string, and `reuse-days` and `confirm-days` integers of 0 or more (`model-wrong-type`); no other key (`model-unknown-key`).

---

## 20. `[permalinks]`
@available: ascribe 0.3.0

Names for something outside the docs to link to that no heading or page declares (SPEC §7.6): a permalink ended on purpose, and a second name for one.

```toml
[permalinks.retired]
rotate-keys = { to = "guides/security.md#keys", reason = "Merged into the security guide" }
legacy-sync = { reason = "The sync agent was removed in 4.0" }

[permalinks.aliases]
agent-settings = "settings"
```

| Key | Type | Default | Description |
|---|---|---|---|
| `retired.<name>` | table | | A retired permalink. `reason` (string, required, not empty) is why, which a reader whose link leads there sees. `to` (string) is where its address leads now: a page's path from the content root, and optionally `#` and the id of a heading on it. |
| `aliases.<name>` | string | | An alias: the name it leads to, a permalink a heading or page declares, or another alias that leads to one. |

No other key is allowed in `[permalinks]` or in a retired permalink's table. A name has the characters of an `@id`. `ascribe permalinks retire`, and the editor's quick fix on `permalink-removed`, write `[permalinks.retired]` in place; the quick fix's reason is the placeholder `why this permalink was retired`, which is an error until it's replaced.

**Rules** (§24.10): each name is valid (`model-invalid-name`); a retired permalink has a `reason` (`model-missing-key`), not empty (`model-empty-text`) and not the placeholder (`model-permalinks`), and no other key (`model-unknown-key`); no name is both retired and an alias, and no alias leads back to itself (`model-permalinks`). What needs the content is checked with the project, not by the loader: SPEC §8.2's `permalink-name-conflict`, `permalink-alias-target-missing`, and `permalink-retired-target-missing`.

---

## 21. `[crawlers]`
@available: ascribe 0.3.0

@include: ../../_fragments/content-model/crawlers.md

**Rules** (§24.11): each line is a string (`model-wrong-type`), one line (`model-crawlers-line`), and not empty (`model-empty-text`); `training`, `search`, and `on-request` are each `"allow"` or `"disallow"` (`model-invalid-value`); a crawler a `User-agent` line of `lines` names, compared without case, isn't one a disallowed purpose names, nor one an earlier group of `lines` names (`model-crawlers-named-twice`), where consecutive `User-agent` lines open one group and `*` is never counted; no other key (`model-unknown-key`). What a line says isn't validated otherwise: a line that isn't a `User-agent`, `Allow`, or `Disallow` line is written as it is, and `ascribe check` reads only the lines it can.

The crawlers of each purpose are data Ascribe ships, each with its operator, the operator's page about it, and the day it was read; a release may add to them. A disallowed purpose's group names every one Ascribe knows of it.

---

## 22. `[listing]`
@available: ascribe 0.3.0

@include: ../../_fragments/content-model/listing.md

**Rules** (§24.12): each value is one of `listed`, `unquoted`, and `unlisted` (`model-invalid-value`); a folder is a path from the content root ending in `/`, without `.` or `..` parts or a pattern's characters, of a folder that exists by its exact name (`model-listing-folder`, a filesystem rule); a type is a content type the model declares, or `page` when it declares none (`model-listing-type`); no rule is looser than a folder's it's under, or than `default` (`model-listing-looser`); no other key (`model-unknown-key`). A page that's looser than a rule it's under is checked with the project, not by the loader: SPEC §8.2's `listing-loosened`, and `listing-loosened-in-line` for `older-lines`, which only an older line's pages are under.

---

## 23. Defaults: what an absent section means

@include: ../../_fragments/content-model/defaults.md
