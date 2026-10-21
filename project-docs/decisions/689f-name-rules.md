---
section: The language
---

**Names follow three rules beyond the one-role rule: a dimension value belongs to one dimension, dimension names follow the `key` rule, and names that differ only in case are a warning.** The first two are SPEC §7.2's (`model-dimension-value-shared`, and the `key` grammar for dimension names). The third is the loader's. _Why:_ `cloud` in an availability spec must mean one thing; dimension names are written as attribute keys in `@variant {deployment=cloud}`; and names are case-sensitive, so two that differ only in case are easy to confuse (`model-name-case`). _From:_ the content model contract's decisions, item 6 ([§26 of its validation rules](https://github.com/ascribed-dev/ascribe/blob/eb7e077/docs/content/contracts/content-model/validation.md#26-decisions)), 28 September 2026, as recommended there; a file of its own since [issue #689](https://github.com/ascribed-dev/ascribe/issues/689).
