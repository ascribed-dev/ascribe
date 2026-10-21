---
section: The language
---

**A fragment can't use the page-only reserved keys; `@available` inside the fragment does what they would.** `available` and `variant` are defined for pages (SPEC §2.1, §4.3, §4.4), so in spec 0.1 a fragment's frontmatter can't use them, and the keys reserved since (`type`, `permalink`, `listing`, `replaces`, `nav-label`) followed. SPEC §2.1 states it. *Considered:* a fragment's `available` applying to everything the fragment contributes, like a section's spec. _Why:_ `available` and `variant` are defined for pages, and `@available` inside a fragment already marks what it contributes. _From:_ the content model contract's decisions, item 4 ([§26 of its validation rules](https://github.com/ascribed-dev/ascribe/blob/eb7e077/docs/content/contracts/content-model/validation.md#26-decisions)), 28 September 2026, as recommended there; a file of its own since [issue #689](https://github.com/ascribed-dev/ascribe/issues/689).
