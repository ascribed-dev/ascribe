---
section: The language
---

**A feature's spec can't name another feature.** The loader reports it (`model-feature-nested`). _Why:_ features that refer to features would allow chains and cycles, and a reader of one feature would have to follow them. _From:_ the content model contract's decisions, item 19 ([§26 of its validation rules](https://github.com/ascribed-dev/ascribe/blob/eb7e077/docs/content/contracts/content-model/validation.md#26-decisions)), 28 September 2026, as recommended there; a file of its own since [issue #689](https://github.com/ascribed-dev/ascribe/issues/689).
