---
section: The language
---

**The plain-markdown output's absolute links take the site's origin from an optional `[consumer] site`; without it, links are root-relative and `ascribe build` warns.** SPEC §9.4 says the plain output's links are absolute URLs, which needs an origin the source doesn't have. _Why:_ SPEC §9.4's absolute URLs need the site's origin, which nothing else in a project gives. _From:_ the content model contract's decisions, item 10 ([§26 of its validation rules](https://github.com/ascribed-dev/ascribe/blob/eb7e077/docs/content/contracts/content-model/validation.md#26-decisions)), 28 September 2026, as recommended there; a file of its own since [issue #689](https://github.com/ascribed-dev/ascribe/issues/689).
