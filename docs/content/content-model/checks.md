---
type: reference
title: "`ascribe.toml`: checks and the editor"
description: "How the content model sets the checks' levels and the editor's build, what an absent section means, and how the file is validated."
---

Part of the [`ascribe.toml` reference](index.md), whose [contents](index.md#contents) list every section. The rules for a loader are in the contract's [checks and the editor](../contracts/content-model/checks.md).

## 22. `[checks]`
@id {permalink=true}: 22-checks

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

Advice is shown in the editor and by `ascribe check`, and never fails the check, even with `--deny-warnings`; `off` turns a check off altogether. A check raised to `"error"` fails the check and stops the build, as an error does, though the page is still valid by the specification, and a review check raised this way can still be acknowledged. [Quieting a problem](../checks/check.md#quieting-a-problem) compares `[checks]` with the other ways to quiet one.

A check the diagnostics reference marks off by default, such as `summary-missing`, is off until `[checks]` names it: with a level, it's reported at that level, and as a table without one, such as `[checks.summary-long]` with only a `limit`, at its default level ([Setting a check's level](../checks/diagnostics.md#setting-a-checks-level)).

In VS Code, the sidebar's [This page](../editor/sidebar.md#this-page) shows a page's size against `page-size`'s limit, measured the same way, whether or not the page is over it.

**Rules.** A key that isn't a check is an unknown key, with the closest check suggested (`model-unknown-key`), and so is a setting the check doesn't have. A check that isn't configurable can't be named (`model-check-not-configurable`). A level is one of the four (`model-invalid-value`). `page-size`'s, `memo-long`'s, and `summary-long`'s `limit` is a whole number above 0, and `image-large`'s a size (`model-wrong-type`).

### `[[intended]]`: acknowledgements
@id {permalink=true}: intended-acknowledgements

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

## 23. `[editor]`

@include: ../_fragments/content-model/editor.md

---

## 24. Defaults: what an absent section means

@include: ../_fragments/content-model/defaults.md

---

## 25. Validation

A content model with errors is reported, and nothing else is checked, since every other check depends on it. Its warnings are reported with the rest of the diagnostics. Every rule the loader enforces is listed in the diagnostics reference, on [Diagnostics: the content model](../checks/diagnostics/content-model.md) and [Diagnostics: the content model's builds and site](../checks/diagnostics/content-model-builds.md), with its code and how to fix it. The [contract's §25](../contracts/content-model/validation.md#25-validation-rules) states each rule for someone writing a loader.
