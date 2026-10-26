---
type: reference
title: "`ascribe.toml` format contract: checks, permalinks, and listing"
description: "The normative contract for the content model's checks, permalinks, crawlers, listing, APIs, and defaults."
---

Part of the [`ascribe.toml` format contract](../content-model.md), whose [contents](../content-model.md#contents) list every section. What these keys are for, with examples, is in the reference's [checks, permalinks, and listing](../../content-model/checks.md).

## 19. `[checks]`

Each key of `[checks]` other than `vale` and `links` is a check's slug, as the [diagnostics reference](../../checks/diagnostics.md) lists it, and its value is a level, `"off"`, `"advice"`, `"warning"`, or `"error"`, or a table, `[checks.<check>]`, with these keys and the check's own settings:

@include: ../../_generated/content-model-checks.md

A check's own settings are listed with the check in the [diagnostics reference](../../checks/diagnostics/content-checks.md). `page-size` and `image-large` take `limit`, and so does `memo-long` from ascribe 0.4.0.

`summary-long` takes `limit` too.

A level applies wherever the diagnostic is reported: by `ascribe check`, by `ascribe build`, and in the editor. `"off"` MUST NOT be reported, and advice MUST NOT fail `ascribe check`, with `--deny-warnings` or without. A check the diagnostics registry marks off by default (`off-by-default`) MUST NOT be reported unless `[checks]` names it: named with a level, it's reported at that level, and named as a table without `level`, at its registry severity.

**Rules** (§25.9): a key MUST be a check's slug, or `vale` or `links`, and an unknown one is reported with a did-you-mean suggestion (`model-unknown-key`). The check MUST be one the diagnostics registry marks configurable (`model-check-not-configurable`), and a level one of the four (`model-invalid-value`). A value MUST be a string or a table, and a table MUST have only `level` and the check's settings (`model-unknown-key`). `page-size`'s, `memo-long`'s, and `summary-long`'s `limit` MUST be an integer above 0, and `image-large`'s an integer number of bytes or a string of a number and `B`, `KB`, or `MB`, with or without a space between, where a kilobyte is 1,000 bytes and a megabyte 1,000,000 (`model-wrong-type`).

### 19.1 `[[intended]]`

Each `[[intended]]` table acknowledges one review check's problem with one content model entry or image (SPEC §4.9). The problem MUST NOT be reported, and MUST be counted and listed with the reason instead, in every build.

@include: ../../_generated/content-model-intended.md

An acknowledgement that covers nothing when every build is checked is advice (`intended-unused`), at its `[[intended]]` header, with a fix that removes the table. A glossary term is acknowledged with `@intended` in its section, and `term` here is `model-unknown-key`.

**Rules** (§25.9): `check` MUST name a check that exists, with a did-you-mean suggestion when it doesn't, whose registry entry has `next = "review"` and `place = "entry"` (`model-intended-check`). The table MUST name exactly one declared phrase, feature, or image (`model-intended-entry`). `check` and `reason` MUST be present (`model-missing-key`), `reason` MUST NOT be empty (`model-empty-text`), and the table MUST have no other key (`model-unknown-key`).

### 19.2 `[checks.vale]`

How the prose is checked with Vale, which the project installs and Ascribe runs, never links or downloads.

@include: ../../_generated/content-model-checks-vale.md

A preset's name MUST NOT change once released, and a rule added to a preset is a **Behavior change** in the changelog. A `command` that's a relative path, or an absolute one into the project's folder, is a program in the project: the language server MUST run it only once the person allows it, and the agents' hook MUST NOT run it.

Ascribe MUST write nothing for Vale outside `.ascribe/vale/` in the project root, except by `ascribe vale eject`: there it writes a preset's configuration and styles, and the vocabulary `Ascribe` of the project's own words (phrase values, glossary terms and aliases, dimension, value, and feature labels), one escaped regular expression to a line.

Each alert is a `prose` diagnostic (`ASC165`) on the source file, with the rule as its JSON `rule`, and at the phrase when it's inside a phrase's text. A Vale that can't be run, fails, runs out of time, or isn't started by its caller is one `prose-not-checked` advice (`ASC166`) at `[checks.vale]`, and never fails the check. It's reported only where the project asked for its prose checked everywhere: `ascribe check` with `--vale`, or anything that checks with `in-check = true`. The language server, which runs Vale whenever there's `[checks.vale]`, MUST NOT report it without `in-check`. The MCP server never starts Vale. Both diagnostics are configurable in `[checks]`, whose level applies after `max-level` caps Vale's.

**Rules** (§25.9): `[checks.vale]` MUST have exactly one of `preset` and `config`, and `off` MUST appear only with `preset` and name only its rules (`model-checks-vale`). `preset` and `max-level` MUST take one of their values (`model-invalid-value`), and the table no other key (`model-unknown-key`).

### 19.3 `[checks.links]`

How `ascribe report links` checks external links: with lychee, which the project installs, run in the project root with the addresses on its standard input. `ascribe check` MUST NOT reach the network.

@include: ../../_generated/content-model-checks-links.md

**The links checked** are those whose destination is an `http` or `https` URL, in every source file, each address once, except a host `ignore` matches, compared without case after a trailing `.` is dropped. An address that answers with a `5xx` status is asked about again, once, after a wait, and only the second answer counts; a link whose address fails again with a `5xx` is reported with the message variant that says the server failed, not that the link is broken for readers. A link whose address answers with an error, doesn't answer in time, or can't be reached is a `link-external-broken` advice (`ASC168`, `next = "review"`, `place = "block"`), and one whose redirects begin with a `301` or `308` a `link-external-moved` advice (`ASC169`, `next = "choose"`), with an unsafe fix that writes the address the permanent redirects end at. Both are at the link, and configurable in `[checks]`. An `@intended {check=link-external-broken}` that covers no broken link is `intended-unused` only when `ascribe report links` ran.

**The cache** is `.ascribe/cache/links/` in the project's folder: one file an address, keyed by the address exactly as lychee is given it and a hash of the project's `lychee.toml`. Only an answer with nothing to report is kept, so every diagnostic in a report comes from an address asked about in that run. An address's answer is reused while it's younger than its own limit: a number of days from half of `reuse-days`, rounded up, through all of it, fixed by a hash of the address. A file that can't be read, has another version, or holds another key counts as no answer.

**The history** is `.ascribe/history/links.json` in the project's folder: one file, version 1, sorted, written whole through a rename after each run, holding for each address the pages link to the last day it answered, the day it started failing, and whether the run reported it. An address no longer linked to is dropped. A run that reads the file again before writing it and finds another run's writing keeps, for each address, the record that confirms its failure later. A link that fails, other than with `404` or `410`, whose address the history records answering before, is unconfirmed, not a diagnostic, until it has failed on every run for `confirm-days` days from its first failure since that answer. A file that can't be read or has another version is no history, and with none, or with `--no-history`, every failure is a diagnostic. A permanent redirect is an answer. An `@intended {check=link-external-broken}` covers an unconfirmed link as it covers a reported one.

`ascribe check`, `build`, `diff`, `drift`, `fmt`, the language server, and the agents' hook MUST NOT read or write the cache or the history.

**Rules** (§25.9): each `ignore` entry MUST be a host name, or `*.` and one (`model-wrong-type`); `command` MUST be a string, and `reuse-days` and `confirm-days` integers of 0 or more (`model-wrong-type`); the table MUST have no other key (`model-unknown-key`).

---

## 20. `[permalinks]`

None. A permalink's record, retired or moved, is an entry of `ascribe.addresses` keyed `go:<name>` (SPEC §7.10), and `[permalinks]` is a replaced key: a loader MUST report it as an error at the key (`model-unknown-key`), whose fix says that `ascribe addresses init` moves its entries.

---

## 21. `[crawlers]`

@include: ../../_fragments/content-model/crawlers.md

**Rules** (§25.11): each of `lines` MUST be a string (`model-wrong-type`) of one line (`model-crawlers-line`), not empty (`model-empty-text`). `training`, `search`, and `on-request` MUST each be `"allow"` or `"disallow"` (`model-invalid-value`). A crawler a `User-agent` line of `lines` names, compared without case, MUST NOT be one a disallowed purpose names, nor one an earlier group of `lines` names (`model-crawlers-named-twice`), where consecutive `User-agent` lines open one group and `*` is never counted. The table MUST have no other key (`model-unknown-key`). A line is otherwise not validated: one that isn't a `User-agent`, `Allow`, or `Disallow` line MUST be written as it is, and `ascribe check` reads only the lines it can.

The crawlers of each purpose are data Ascribe ships, each with its operator, the operator's page about it, and the day it was read; a release MAY add to them. A disallowed purpose's group MUST name every one Ascribe knows of it.

---

## 22. `[listing]`

@include: ../../_fragments/content-model/listing.md

**Rules** (§25.12): each value MUST be one of `listed`, `unquoted`, and `unlisted` (`model-invalid-value`). A key of `[listing.folders]` MUST be a path from the content root ending in `/`, without `.` or `..` parts or a pattern's characters, of a folder that exists by its exact name (`model-listing-folder`, a filesystem rule), and a key of `[listing.types]` a content type the model declares, or `page` when it declares none (`model-listing-type`). A rule MUST NOT be looser than a folder's it's under, or than `default` (`model-listing-looser`), and the table MUST have no other key (`model-unknown-key`). A page looser than a rule it's under is checked with the project, not by the loader: SPEC §8.2's `listing-loosened`.

`older-lines` is held to `default` by `model-listing-looser`, as a folder's and a type's rules are. A page looser than `older-lines`, which only an older line's pages are under, is SPEC §8.2's `listing-loosened-in-line`.

---

## 23. `[apis.<name>]`
@available: ascribe 0.4.4

@include: ../../_fragments/content-model/apis.md

**Rules** (§25.13): `spec` MUST be relative (`model-path-absolute`) and, when the project is in a git repository, inside it (`model-api-outside-repository`); with no repository, the language server and `ascribe mcp` MUST read a spec only inside what the user opened, and report one outside it as `model-api-outside-repository` with the variant `opened`, as a source's folder is (§18). `reference` MUST be a table with exactly `renderer` and `at`, `renderer` one of the choices (`model-missing-key`, `model-unknown-key`, `model-invalid-value`), or a string with `{operationId}` and no placeholder but `{operationId}` and `{tag}` (`model-api-reference`). What needs the spec's file is checked with the project, not by the loader: SPEC §8.2's `api-spec-unreadable`, `api-spec-empty`, `link-operation-missing`, and `link-spec-outside-project`. A compiler MUST percent-encode each value a template's placeholder takes, keeping only RFC 3986's unreserved characters, and MUST join an address that isn't absolute to the site's `[consumer] base-path`, without a line's folder.

---

## 24. Defaults: what an absent section means

@include: ../../_fragments/content-model/defaults.md
