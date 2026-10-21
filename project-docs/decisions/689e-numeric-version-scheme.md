---
section: The language
---

**The one version scheme is called `numeric`: dotted numbers of any length, compared numerically, with missing components as 0.** `[versions] scheme` accepts it and nothing else in spec 0.1, and the table exists so a later spec can add a scheme without changing the file's shape. *Considered:* calling it `semver` and capping versions at three components, which would need a new document diagnostic. _Why:_ the grammar has no pre-release syntax and allows any number of components, so "semver" would promise what it doesn't do. _From:_ the content model contract's decisions, item 5 ([§26 of its validation rules](https://github.com/ascribed-dev/ascribe/blob/eb7e077/docs/content/contracts/content-model/validation.md#26-decisions)), 28 September 2026, as recommended there; a file of its own since [issue #689](https://github.com/ascribed-dev/ascribe/issues/689).
