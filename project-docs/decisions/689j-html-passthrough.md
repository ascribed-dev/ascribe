---
section: The language
---

**`[consumer] html` stays a key, and the `astro` profile accepts only `true`.** A profile that needs `false` defines what its output does then. _Why:_ the site output depends on raw HTML for its custom elements (SPEC §9.5), so a site output without it would be a degraded one defined for no consumer that needs it. _From:_ the content model contract's decisions, item 11 ([§26 of its validation rules](https://github.com/ascribed-dev/ascribe/blob/eb7e077/docs/content/contracts/content-model/validation.md#26-decisions)), 28 September 2026, as recommended there; a file of its own since [issue #689](https://github.com/ascribed-dev/ascribe/issues/689).
