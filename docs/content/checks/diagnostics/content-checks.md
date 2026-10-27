---
type: reference
title: "Diagnostics: content checks"
description: "The checks of a project's pages, which it can set the level of or turn off."
---

Part of the [diagnostics reference](../diagnostics.md), which says what an entry's severity, level, and next step mean, and has an index of every diagnostic.

Checks of the content's quality: advice unless a project sets another level in [`[checks]`](../../content-model/checks.md#22-checks). A check with settings takes a table, and its entry names them. This page has the checks of pages; the checks of the project as a whole are on [Diagnostics: content checks across the project](project-checks.md), and the checks of outputs, prose, external links, the published site, and tested examples are on [Diagnostics: content checks of outputs and the site](output-checks.md).

```toml
[checks]
code-language-missing = "warning"     # so --deny-warnings fails on it
heading-level-skipped = "off"

[checks.page-size]
level = "warning"
limit = 40_000
```

`recheck-overdue` depends on the day, as [`release-line-ended`](project-checks.md#ASC214) does, so `ascribe check` can report it on one day and not the day before: see [Checking as of another day](../check.md#checking-as-of-another-day).

@include: ../../_generated/diagnostics-content-checks.md
