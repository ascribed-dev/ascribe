---
title: "Diagnostics: content checks"
description: "The checks of a project's quality, which it can set the level of or turn off."
---

Part of the [diagnostics reference](../diagnostics.md), which says what an entry's severity, level, and next step mean, and has an index of every diagnostic.

@available: ascribe 0.3.0
Checks of the content's quality: advice unless a project sets another level in [`[checks]`](../content-model/checks.md#19-checks). A check with settings takes a table, and its entry names them.

```toml
[checks]
code-language-missing = "warning"     # so --deny-warnings fails on it
heading-level-skipped = "off"

[checks.page-size]
level = "warning"
limit = 40_000
```

`review-overdue` depends on the day, so `ascribe check` can report it on one day and not the day before. Set `ASCRIBE_TODAY=YYYY-MM-DD` to check as of another day; Ascribe otherwise uses today's date in UTC.

The checks across the project need every page, so the editor reports a page's orphan and duplicate title, and what's unused, when you save, and the checks of image files run only in `ascribe check`.

@include: ../../_generated/diagnostics-content-checks.md
