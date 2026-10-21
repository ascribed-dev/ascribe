---
section: The language
---

**A page's title is its frontmatter `title`, which every page type declares as a required string.** It's what SPEC §5.2's "the page title" is, as the text of an empty link to a page, and SPEC §7.2 states it; the loader reports a type without it (`model-type-title`). *Considered:* falling back to the page's first level-1 heading. _Why:_ empty link text uses "the page title" (SPEC §5.2), and the spec didn't say where it comes from. _From:_ the content model contract's decisions, item 3 ([§26 of its validation rules](https://github.com/ascribed-dev/ascribe/blob/eb7e077/docs/content/contracts/content-model/validation.md#26-decisions)), 28 September 2026, as recommended there; a file of its own since [issue #689](https://github.com/ascribed-dev/ascribe/issues/689).
