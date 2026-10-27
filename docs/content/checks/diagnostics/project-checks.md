---
type: reference
title: "Diagnostics: content checks across the project"
description: "The checks of a project as a whole, such as orphan pages, what's unused, and release lines, which it can set the level of or turn off."
---

The content checks of the project as a whole: what nothing uses or links to, what two pages share, and what a release line marks. The checks of pages are on [Diagnostics: content checks](content-checks.md), which says how a project sets a check's level or turns it off in [`[checks]`](../../content-model/checks.md#22-checks), and the checks of outputs, prose, external links, the published site, and tested examples are on [Diagnostics: content checks of outputs and the site](output-checks.md). Part of the [diagnostics reference](../diagnostics.md), which says what an entry's severity, level, and next step mean, and has an index of every diagnostic.

These checks need every page, so the editor reports a page's orphan and duplicate title, and what's unused, when you save, and the checks of image files run only in `ascribe check`. `release-line-ended` depends on the day, so `ascribe check` can report it on one day and not the day before: see [Checking as of another day](../check.md#checking-as-of-another-day).

@include: ../../_generated/diagnostics-project-checks.md
