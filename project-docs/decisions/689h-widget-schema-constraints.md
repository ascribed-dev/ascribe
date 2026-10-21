---
section: The language
---

**A groupable widget is container-only, a widget with container form has no required primary, and widget names starting with `ascribe-` or reserved by HTML are rejected.** The loader reports each (`model-widget-groupable-form`, `model-widget-container-primary`, `model-widget-reserved-name`). _Why:_ group arms are containers (SPEC §3.6), and a container opener's primary is empty (SPEC §3.5). _From:_ the content model contract's decisions, item 9 ([§26 of its validation rules](https://github.com/ascribed-dev/ascribe/blob/eb7e077/docs/content/contracts/content-model/validation.md#26-decisions)), 28 September 2026, as recommended there; a file of its own since [issue #689](https://github.com/ascribed-dev/ascribe/issues/689).
