---
type: reference
title: "`ascribe.toml`: checks, permalinks, and listing"
description: "How the content model sets the checks' levels, permalinks, crawlers, and listing, its defaults, and how it's validated."
---

Part of the [`ascribe.toml` reference](index.md), whose [contents](index.md#contents) list every section. The rules for a loader are in the contract's [checks, permalinks, and listing](../contracts/content-model/checks.md).

## 19. `[checks]`

How loudly each check about your content's quality speaks: whether it's reported as advice, a warning, or an error, or not at all. A diagnostic about whether the project is valid, such as a broken link, can't be set here: it's always reported at its own severity.

```toml
[checks]
code-language-missing = "warning"

[checks.page-size]
level = "warning"
limit = 40_000
```

Each key is a check's name, as the [diagnostics reference](../checks/diagnostics.md) lists it. Its value is the check's level, or a table, `[checks.<check>]`, with the check's own settings, which its entry in the reference lists, and these keys:

@include: ../_generated/content-model-checks.md

Only a check the [diagnostics reference](../checks/diagnostics.md#setting-a-checks-level) marks configurable can be named; that section lists them, with their settings. Four have one, `limit`: for `page-size`, a whole number of characters above 0, `50_000` unless set; for `image-large`, a size, as a number of bytes or a string such as `"1.5 MB"` (a kilobyte is 1,000 bytes), `"500 KB"` unless set; for `memo-long`, a whole number of characters above 0, `2_000` unless set; and for `summary-long`, a whole number of characters above 0, `300` unless set. `[checks.vale]` and `[checks.links]` are reserved for the settings of tools Ascribe runs.

For example, a site whose sidebar lists every page doesn't need to hear about pages nothing links to, and a team that wants smaller images can lower the limit:

```toml
[checks]
page-orphan = "off"
title-duplicate = "warning"

[checks.image-large]
limit = "200 KB"
```

Advice is shown in the editor and by `ascribe check`, and never fails the check, even with `--deny-warnings`; `off` turns a check off altogether.

A check the diagnostics reference marks off by default, such as `summary-missing`, is off until `[checks]` names it: with a level, it's reported at that level, and as a table without one, such as `[checks.summary-long]` with only a `limit`, at its default level ([Setting a check's level](../checks/diagnostics.md#setting-a-checks-level)).

In VS Code, the sidebar's [This page](../editor/sidebar.md#this-page) shows a page's size against `page-size`'s limit, measured the same way, whether or not the page is over it.

**Rules.** A key that isn't a check is an unknown key, with the closest check suggested (`model-unknown-key`), and so is a setting the check doesn't have. A check that isn't configurable can't be named (`model-check-not-configurable`). A level is one of the four (`model-invalid-value`). `page-size`'s, `memo-long`'s, and `summary-long`'s `limit` is a whole number above 0, and `image-large`'s a size (`model-wrong-type`).

### `[[intended]]`: acknowledgements
@id: intended-acknowledgements

Says that a check's problem with a content model entry or an image is intended, as [`@intended`](../writing/directives.md#intended) does for a block and the `intended` frontmatter key for a page. Each table names one check, one entry, and why.

```toml
[[intended]]
check = "phrase-unused"
phrase = "old-product-name"
reason = "Kept for the 2.x pages restored in the next release."
```

@include: ../_generated/content-model-intended.md

A glossary term is a heading, so it's acknowledged on its page instead, with [`@intended`](../writing/directives.md#intended) under its heading; `term` here is an error that says so.

**Rules.** A check that doesn't exist, isn't a review check, or reports its problems somewhere other than an entry is an error (`model-intended-check`). A table without exactly one entry key, or naming an entry that isn't declared, is an error (`model-intended-entry`).

### `[checks.vale]`

Checks the prose with [Vale](https://vale.sh), a program the project installs: Ascribe gives it each page's prose, with phrases replaced by their text and without frontmatter, directive lines, attributes, or code, and reports its alerts as `prose` diagnostics at their place in the source. The editor runs it on a page when it's opened and saved; `ascribe check` runs it with `--vale`, or always with `in-check = true`. [Prose, through Vale](../checks/vale.md) has the steps.

```toml
[checks.vale]
preset = "quiet"
off = ["Ascribe.Repeated"]
```

@include: ../_generated/content-model-checks-vale.md

Ascribe writes the project's own words (phrase values, glossary terms and their aliases, and the labels of dimensions, their values, and features) to a Vale vocabulary, `Ascribe`, under `.ascribe/vale/`, so Vale doesn't take them for misspellings. It's turned on for a preset and for the project's own configuration alike; nothing the project wrote is changed.

The level of every Vale alert can be set at once with `prose` in `[checks]`, and the advice that Vale couldn't be run with `prose-not-checked`.

**Rules.** `[checks.vale]` has exactly one of `preset` and `config`, and `off` goes with `preset` and names its rules (`model-checks-vale`). `preset` is one of Ascribe's presets, and `max-level` one of the three levels (`model-invalid-value`).

### `[checks.links]`

How `ascribe report links` checks the project's external links, with [lychee](https://lychee.cli.rs), a program the project installs. `ascribe check` never checks external links. [Check external links](../checks/external-links.md) has the steps.

```toml
[checks.links]
ignore = ["localhost", "*.internal.example.com"]
```

@include: ../_generated/content-model-checks-links.md

The answers it reuses and the links it holds as unconfirmed are kept in `.ascribe/`: [What it remembers](../checks/external-links.md#what-it-remembers).

A broken link is `link-external-broken` and a moved one `link-external-moved`; both are configurable in `[checks]`, and a broken one is acknowledged with `@intended` above its block.

**Rules.** `ignore` names hosts, without a scheme, path, or port, and `reuse-days` and `confirm-days` are whole numbers, 0 or more (`model-wrong-type`).

---

## 20. `[permalinks]`

Where a project used to record what became of a [permalink](../writing/directives.md#permalinks) no heading or page declares: one ended on purpose, and another name for one. They're kept in [`ascribe.addresses`](../publishing/redirects.md#ascribeaddresses) now, under the key `permalink:` and the name, with every other address the docs have published; the [guide](../permalinks/index.md#moving-aliasing-and-retiring) says when to use each. A project with these tables still has them read, as they were, and is warned of each (`model-permalinks-moved`) until [`ascribe addresses init`](../publishing/redirects.md#cli-addresses-init) moves them there.

```toml
[permalinks.retired]
rotate-keys = { to = "guides/security.md", reason = "Merged into the security guide" }
legacy-sync = { reason = "The sync agent was removed in 4.0" }

[permalinks.aliases]
agent-settings = "settings"
cli-settings = "settings"
```

`[permalinks.retired]` has an entry for each retired permalink, keyed by its name:

@include: ../_generated/content-model-permalinks-retired.md

`[permalinks.aliases]` has an entry for each alias, keyed by its name, whose value is the name it leads to: a permalink a heading or page declares, or another alias that leads to one.

Moved, a retired permalink with a `to` is a `[moved]` entry, which keeps no reason, one without is a `[retired]` entry, and an alias is a `[moved]` entry whose value is `permalink:` and the name it leads to. Ascribe writes neither table any more: [`ascribe permalinks retire`](../permalinks/command.md#cli-permalinks-retire) and the editor's quick fix write `ascribe.addresses`.

**Rules.** A name has the characters of an `@id` (`model-invalid-name`). A retired permalink has a `reason`, not empty and not the quick fix's placeholder, and no keys but `reason` and `to` (`model-missing-key`, `model-empty-text`, `model-permalinks`, `model-unknown-key`). A name is one thing: retired, an alias, or declared by the content, never two (`model-permalinks`, `permalink-name-conflict`). An alias leads, through aliases, to a permalink the content declares, never back to itself or to a retired permalink (`model-permalinks`, `permalink-alias-target-missing`). A `to` leads to a page, not a fragment, and to a heading on it when it names one (`permalink-retired-target-missing`). A name `ascribe.addresses` records too is its entry there.

---

## 21. `[crawlers]`

@include: ../_fragments/content-model/crawlers.md

What each purpose's choice costs, and why a page is kept out of search by its listing and not by `robots.txt`, is in [Publishing](../publishing/index.md#what-crawlers-are-told).

`ascribe check` reads each search crawler the way it reads the file: the one group that applies to it. A page that isn't listed is reported when a search crawler may not fetch it, whether the lines for every crawler say so, a group of `lines` that names that crawler, or `search = "disallow"` (`listing-disallowed`).

### Crawlers by purpose

Each operator below runs a bot for each purpose, or a token that says what it may do with what its other bots fetch, and says on its own page which is which. A purpose's group names each of them. A bot whose operator states no single purpose isn't here, such as Common Crawl's `CCBot`, whose archive many train on, or Meta's `meta-externalagent`, which gathers for training and for indexing both; name such a bot in `lines`, in a group of its own. Each row says what its operator said on the day in its last column, linked to the page.

@include: ../_generated/crawlers.md

The list changes, as operators add bots and rename them. Ascribe's release process rereads every source, and a newer Ascribe may name more crawlers for a purpose than an older one did.

**Rules.** Each line is one line (`model-crawlers-line`) and not empty (`model-empty-text`). A purpose is `allow` or `disallow` (`model-invalid-value`). A crawler is named in one group: not in a group of `lines` that a disallowed purpose names, and not in two groups of `lines` (`model-crawlers-named-twice`). Projects that build one site must agree on each purpose, since it has one `robots.txt`: one that allows a purpose and one that disallows it fail the site's build, naming both. No other key is allowed in `[crawlers]` (`model-unknown-key`).

---

## 22. `[listing]`

@include: ../_fragments/content-model/listing.md

What each value tells search engines and agents is in [Publishing](../publishing/index.md#how-a-page-is-listed).

In VS Code, the sidebar's [This page](../editor/sidebar.md#this-page) says whether a page is listed, and which rule says so; choosing the row opens the rule.

In the editor, `listing:` in a page's frontmatter completes its three values, and hovering it says which rule the page is under. A page looser than its rule has a quick fix that removes its `listing`.

**Rules.** Each value is one of the three (`model-invalid-value`). A folder is a path from the content root ending in `/`, with no `.` or `..` part or pattern characters, of a folder that's there, with its name's case as on disk (`model-listing-folder`). A type is one the project declares, or `page` when it declares none (`model-listing-type`). A rule isn't looser than a folder's it's under, or than `default` (`model-listing-looser`). A page isn't looser than a rule it's under (`listing-loosened`). No other key is allowed in `[listing]` (`model-unknown-key`).

In an older line of a versioned build, a page isn't looser than `older-lines` either (`listing-loosened-in-line`).

---

## 23. `[apis.<name>]`
@available: ascribe 0.4.4

@include: ../_fragments/content-model/apis.md

How to publish a reference and link to it is in [Link to your API reference](../openapi/index.md).

**Rules.** `spec` is a path from the project root, relative (`model-path-absolute`), inside the git repository the project is in, when it's in one (`model-api-outside-repository`). A file that isn't there, that can't be read, or that isn't YAML or JSON is an error at `spec` (`api-spec-unreadable`), and one with no operations is advice (`api-spec-empty`). `reference` is a table with `renderer` and `at`, or a template with `{operationId}` and no placeholder but `{operationId}` and `{tag}` (`model-api-reference`). A link to an operation the spec doesn't have is an error at the link, with the closest ids as its fixes (`link-operation-missing`). An operation no link names, unless the spec marks it `deprecated`, is advice at `spec` (`api-operation-unreferenced`): see [Find the operations no guide links to](../openapi/coverage.md). No other key is allowed (`model-unknown-key`).

---

## 24. Defaults: what an absent section means

@include: ../_fragments/content-model/defaults.md

---

## 25. Validation

A content model with errors is reported, and nothing else is checked, since every other check depends on it. Its warnings are reported with the rest of the diagnostics. Every rule the loader enforces is listed in the diagnostics reference, on [Diagnostics: the content model](../checks/diagnostics/content-model.md) and [Diagnostics: the content model's builds and site](../checks/diagnostics/content-model-builds.md), with its code and how to fix it. The [contract's §25](../contracts/content-model/validation.md#25-validation-rules) states each rule for someone writing a loader.
