---
section: The language
---

**With no `[types]` there's one implicit type, `page`, with a required `title`; with no `[builds]` there's one build, `site` (`switch`, `badge`); declaring any type or build removes the implicit one.** The implicit type's pages name no type ([decision #450](450-page-type-key.md)). _Why:_ none was recorded with it; it's what makes a file with only `spec = "0.1"` a valid model. _From:_ the content model contract's decisions, item 18 ([§26 of its validation rules](https://github.com/ascribed-dev/ascribe/blob/eb7e077/docs/content/contracts/content-model/validation.md#26-decisions)), 28 September 2026, as recommended there; a file of its own since [issue #689](https://github.com/ascribed-dev/ascribe/issues/689).
