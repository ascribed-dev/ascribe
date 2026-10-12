---
title: "`ascribe.toml`: checks, permalinks, and listing"
description: "How the content model sets the checks' levels, permalinks, crawlers, and listing, its defaults, and how it's validated."
---

Part of the [`ascribe.toml` reference](../content-model.md), whose [contents](../content-model.md#contents) list every section.

## 19. `[checks]`
@available: next

How loudly each check about your content's quality speaks: whether it's reported as advice, a warning, or an error, or not at all. A diagnostic about whether the project is valid, such as a broken link, can't be set here: it's always reported at its own severity.

```toml
[checks]
code-language-missing = "warning"

[checks.page-size]
level = "warning"
limit = 40_000
```

| Key | Type | Default | Description |
|---|---|---|---|
| `<check>` | string or table | the check's own level | A check's name, as the [diagnostics reference](../diagnostics.md) lists it. The value is its level: `"off"`, `"advice"`, `"warning"`, or `"error"`. Or it's a table with `level` and the check's own settings, which its entry in the reference lists; without `level`, the check keeps its own. |

Only a check the [diagnostics reference](../diagnostics.md#setting-a-checks-level) marks configurable can be named; that section lists them, with their settings. Two have one, `limit`: for `page-size`, a whole number of characters above 0, `50_000` unless set; for `image-large`, a size, `"500 KB"` unless set. `[checks.vale]` and `[checks.links]` are reserved for the settings of tools Ascribe runs.

For example, a site whose sidebar lists every page doesn't need to hear about pages nothing links to, and a team that wants smaller images can lower the limit:

```toml
[checks]
page-orphan = "off"
title-duplicate = "warning"

[checks.image-large]
limit = "200 KB"
```

| Setting | Of | Type | Default | Description |
|---|---|---|---|---|
| `limit` | `page-size` | integer | `50_000` | The size, in characters of a page's plain Markdown in a build, at which the page is reported. Above 0. |
| `limit` | `image-large` | size | `"500 KB"` | The size over which an image is reported: a number of bytes, or a string with `B`, `KB`, or `MB`, such as `"1.5 MB"`. A kilobyte is 1,000 bytes. |

Advice is shown in the editor and by `ascribe check`, and never fails the check, even with `--deny-warnings`. A check set to `off` isn't reported at all.

**Rules.** A key that isn't a check is an unknown key, with the closest check suggested (`model-unknown-key`), and so is a setting the check doesn't have. A check that isn't configurable can't be named (`model-check-not-configurable`). A level is one of the four (`model-invalid-value`). `page-size`'s `limit` is a whole number above 0, and `image-large`'s a size (`model-wrong-type`).

### `[[intended]]`: acknowledgements
@id: intended-acknowledgements
@available: next

Says that a check's problem with a content model entry or an image is intended, as [`@intended`](../directives.md#intended) does for a block and the `intended` frontmatter key for a page. Each table names one check, one entry, and why.

@include: ../../_fragments/content-model/intended-example.md

| Key | Type | Default | Description |
|---|---|---|---|
| `check` | string | required | The check, by its name. It must be a review check that reports its problems at an entry, as the [diagnostics reference](../diagnostics.md) says. |
| `reason` | string | required | Why the problem is intended. Not empty. |
| `phrase`, `feature`, `term`, or `image` | string | required | The entry: a phrase's key, a feature's key, a glossary term's id, or an image's path under the content root. Exactly one. |

A glossary term whose only occurrences are inside a phrase's value is reported by `glossary-term-unused`, since the search for uses doesn't read phrase values. Acknowledge it here with `term`.

**Rules.** A check that doesn't exist, isn't a review check, or reports its problems somewhere other than an entry is an error (`model-intended-check`). A table without exactly one entry key, or naming an entry that isn't declared, is an error (`model-intended-entry`).

### `[checks.vale]`
@available: next

Checks the prose with [Vale](https://vale.sh), a program the project installs: Ascribe gives it each page's prose, with phrases replaced by their text and without frontmatter, directive lines, attributes, or code, and reports its alerts as `prose` diagnostics at their place in the source. The editor runs it on a page when it's opened and saved; `ascribe check` runs it with `--vale`, or always with `in-check = true`. [Prose, through Vale](../../guides/vale.md) has the steps.

```toml
[checks.vale]
preset = "quiet"
off = ["Ascribe.Repeated"]
```

| Key | Type | Default | Description |
|---|---|---|---|
| `preset` | string | | Ascribe's own selection of Vale rules: `quiet`, rules that are almost never wrong. Nothing is downloaded: the preset is written under `.ascribe/vale/`. |
| `config` | string | | Instead of a preset, the path of the project's own `.vale.ini`, from the project root. |
| `off` | array of strings | `[]` | Rules of the preset to turn off, by the names alerts give them, such as `Ascribe.Repeated`. Only with `preset`. |
| `command` | string | `"vale"` | The command that runs Vale. A path with a separator is from the project root; a bare name is looked up as the shell would. A path is a program in the project: the editor asks before it runs one, and the agents' hooks don't run one ([Prose, through Vale](../../guides/vale.md#where-vale-runs)). |
| `in-check` | boolean | `false` | Whether `ascribe check`, and the agents' hook, check the prose without `--vale`. |
| `max-level` | string | `"error"` | The most an alert can be: `"advice"`, `"warning"`, or `"error"`. Vale's `suggestion`, `warning`, and `error` are advice, a warning, and an error up to it; with `"advice"`, Vale's alerts never fail `--deny-warnings`. |

Ascribe writes the project's own words (phrase values, glossary terms and their aliases, and the labels of dimensions, their values, and features) to a Vale vocabulary, `Ascribe`, under `.ascribe/vale/`, so Vale doesn't take them for misspellings. It's turned on for a preset and for the project's own configuration alike; nothing the project wrote is changed.

The level of every Vale alert can be set at once with `prose` in `[checks]`, and the advice that Vale couldn't be run with `prose-not-checked`.

**Rules.** `[checks.vale]` has exactly one of `preset` and `config`, and `off` goes with `preset` and names its rules (`model-checks-vale`). `preset` is one of Ascribe's presets, and `max-level` one of the three levels (`model-invalid-value`).

### `[checks.links]`
@available: next

How `ascribe report links` checks the project's external links, with [lychee](https://lychee.cli.rs), a program the project installs. `ascribe check` never checks external links. [The project report](../../guides/report.md#external-links) has the steps.

```toml
[checks.links]
ignore = ["localhost", "*.internal.example.com"]
```

| Key | Type | Default | Description |
|---|---|---|---|
| `command` | string | `"lychee"` | The command that runs lychee. A path with a separator is from the project root; a bare name is looked up as the shell would. |
| `ignore` | array of strings | `[]` | The hosts whose links aren't checked: a host name, such as `example.com`, or `*.` and a host name, which covers every host under it but not the name itself. Case doesn't matter. |
| `reuse-days` | whole number | `14` | How many days an address that answered isn't asked about again. Each address's own limit falls between half of this and all of it, so addresses first checked together don't all come due on one day. `0` asks about every address on every run. |
| `confirm-days` | whole number | `3` | How many days a link that answered before and fails now is listed as unconfirmed, and not reported, before it's reported. A 404 or a 410 is reported at once. `0` reports every failure at once. |

An answer is reused only when it had nothing to report: a link that moved, failed, or timed out is asked about on every run, so every problem in a report was found in that run. The answers are kept in `.ascribe/cache/links/`; deleting it costs the next run time, and nothing else. `ascribe report links --recheck` asks about every address.

A link is held as unconfirmed only when an earlier run got an answer from its address: what each address last did is kept in `.ascribe/history/links.json`. Deleting it costs one noisier report, which reports every failure as a first run does, and never a missed problem. `ascribe report links --no-history` reports every failure too.

A broken link is `link-external-broken` and a moved one `link-external-moved`; both are configurable in `[checks]`, and a broken one is acknowledged with `@intended` above its block.

**Rules.** `ignore` names hosts, without a scheme, path, or port, and `reuse-days` and `confirm-days` are whole numbers, 0 or more (`model-wrong-type`).

---

## 20. `[permalinks]`
@available: next

Names for something outside the docs to link to that no heading or page declares: a [permalink](../directives.md#permalinks) ended on purpose, and a second name for one. Each is published at an address, as a permalink is, and recorded in `ascribe.permalinks` when the project has it (SPEC §7.6). The [guide](../../guides/permalinks.md#moving-aliasing-and-retiring) says when to use each.

```toml
[permalinks.retired]
rotate-keys = { to = "guides/security.md", reason = "Merged into the security guide" }
legacy-sync = { reason = "The sync agent was removed in 4.0" }

[permalinks.aliases]
agent-settings = "settings"
cli-settings = "settings"
```

`[permalinks.retired]` has an entry for each retired permalink, keyed by its name:

| Key | Type | Default | Description |
|---|---|---|---|
| `reason` | string | required | Why it was retired. A reader whose link leads there sees it, as text. Not empty. |
| `to` | string | | Where its address leads now: a page, by its path from the content root, and `#` and a heading's id, as `guides/security.md#rotate-keys`. Without it, or in a build that doesn't publish the page, the address shows that what was there was removed, the reason, and a link to the docs' home. |

`[permalinks.aliases]` has an entry for each alias, keyed by its name, whose value is the name it leads to: a permalink a heading or page declares, or another alias that leads to one. An alias's address leads where its permalink's does, in every build that publishes it, so repointing the alias changes where it leads and nothing else.

[`ascribe permalinks retire`](../cli/sources.md#cli-permalinks-retire) writes `[permalinks.retired]` for you, and the editor's quick fix on a removed permalink writes an entry with a placeholder reason for you to replace. To bring a retired permalink back, delete its entry and declare the name again.

**Rules.** A name has the characters of an `@id` (`model-invalid-name`). A retired permalink has a `reason`, not empty and not the quick fix's placeholder, and no keys but `reason` and `to` (`model-missing-key`, `model-empty-text`, `model-permalinks`, `model-unknown-key`). A name is one thing: retired, an alias, or declared by the content, never two (`model-permalinks`, `permalink-name-conflict`). An alias leads, through aliases, to a permalink the content declares, never back to itself or to a retired permalink (`model-permalinks`, `permalink-alias-target-missing`). A `to` leads to a page, not a fragment, and to a heading on it when it names one (`permalink-retired-target-missing`).

---

## 21. `[crawlers]`
@available: next

@include: ../../_fragments/content-model/crawlers.md

`robots.txt` is a request: a crawler may ignore it, and some bots document that they may. A crawler that follows it and is told not to fetch a page never reads that page's head. Keeping a page out of search results is the page's [`listing`](../content-model.md#listing-a-page), not a `Disallow` line: `ascribe check` says so when a page that isn't listed is at an address the project's own lines disallow for every crawler (`listing-disallowed`), since the crawler can't read the rule then, and a search engine can list the address alone.

`ascribe check` reads each search crawler the way it reads the file: the one group that applies to it. A page that isn't listed is reported when a search crawler may not fetch it, whether the lines for every crawler say so, a group of `lines` that names that crawler, or `search = "disallow"`.

### Crawlers by purpose

Each operator below runs a bot for each purpose, or a token that says what it may do with what its other bots fetch, and says on its own page which is which. A purpose's group names each of them. A bot whose operator states no single purpose isn't here, such as Common Crawl's `CCBot`, whose archive many train on, or Meta's `meta-externalagent`, which gathers for training and for indexing both; name such a bot in `lines`, in a group of its own. Each row says what its operator said on the day in its last column, linked to the page.

@include: ../../_generated/crawlers.md

What each choice costs, in a sentence each:

- **Disallowing `training`** asks those operators not to train models on the site. It doesn't take the site out of any search or answer, and it stops nothing already gathered. Google's and Apple's are tokens, not crawlers: Googlebot and Applebot still fetch every page for search, and the token says only what may be done with it.
- **Disallowing `search`** takes the site out of those AI search products' answers, which is the point of being crawled by them: a site opted out of OpenAI's search crawler isn't shown in ChatGPT's search answers, for one ([OpenAI](https://developers.openai.com/api/docs/bots), read 2026-10-12). It doesn't touch Google's or Bing's search, whose AI answers are drawn from their ordinary index, which only a page's `listing` keeps a page out of. `ascribe check` says what it costs (`crawlers-disallowed`).
- **Disallowing `on-request`** asks assistants not to fetch a page when a person asks about it, so they answer from what they already have, or from another site. It's a request several operators say their fetcher may not follow, since a person asked: OpenAI's, Perplexity's, Amazon's, and Meta's say so (see the table), and Google's own fetchers for its agents generally ignore `robots.txt` and have no token, so they aren't listed ([Google](https://developers.google.com/search/docs/crawling-indexing/google-user-triggered-fetchers), read 2026-10-12). `ascribe check` says what it costs (`crawlers-disallowed`).

A host's or a CDN's own settings for bots can block a crawler before it reads `robots.txt`, whatever the file says, and can allow it in spite of it. Those are the host's, not Ascribe's: check them when a crawler you allowed can't reach the site.

The list changes, as operators add bots and rename them. Ascribe's release process rereads every source, and a newer Ascribe may name more crawlers for a purpose than an older one did.

**Rules.** Each line is one line (`model-crawlers-line`) and not empty (`model-empty-text`). A purpose is `allow` or `disallow` (`model-invalid-value`). A crawler is named in one group: not in a group of `lines` that a disallowed purpose names, and not in two groups of `lines` (`model-crawlers-named-twice`). Projects that build one site must agree on each purpose, since it has one `robots.txt`: one that allows a purpose and one that disallows it fail the site's build, naming both. No other key is allowed in `[crawlers]` (`model-unknown-key`).

---

## 22. `[listing]`
@available: next

@include: ../../_fragments/content-model/listing.md

`unlisted` pages are out of the sitemap and `llms.txt`, and their head asks not to be listed; `unquoted` pages stay in both, and their head asks not to be quoted (`nosnippet`). That limits what a search result, or an AI answer inside a search product, may quote from the page. It asks nothing of a crawler gathering training data. So a whole folder can be kept out of search without a change to any of its pages:

```toml
[listing.folders]
"internal/" = "unlisted"
```

In the editor, `listing:` in a page's frontmatter completes its three values, and hovering it says which rule the page is under. A page looser than its rule has a quick fix that removes its `listing`.

**Rules.** Each value is one of the three (`model-invalid-value`). A folder is a path from the content root ending in `/`, with no `.` or `..` part or pattern characters, of a folder that's there, with its name's case as on disk (`model-listing-folder`). A type is one the project declares, or `page` when it declares none (`model-listing-type`). A rule isn't looser than a folder's it's under, or than `default` (`model-listing-looser`). A page isn't looser than a rule it's under (`listing-loosened`). No other key is allowed in `[listing]` (`model-unknown-key`).

---

## 23. Defaults: what an absent section means

@include: ../../_fragments/content-model/defaults.md

---

## 24. Validation

A content model with errors is reported, and nothing else is checked, since every other check depends on it. Its warnings are reported with the rest of the diagnostics. Every rule the loader enforces is listed in the [diagnostics reference](../diagnostics/content-model.md), with its code and how to fix it.
