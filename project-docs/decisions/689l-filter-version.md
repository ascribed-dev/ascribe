---
section: The language
---

**A filter build names a version for a versioned target, and none for a versionless one.** SPEC §9.3 gives a filter "a target and, for versioned targets, a version"; the loader requires the version where the target is versioned and rejects it where it isn't (`model-build-filter-version`). *Considered:* a versioned target with no version meaning "at every version". _Why:_ without a version, the "state in effect" a filter decides by is undefined. _From:_ the content model contract's decisions, item 13 ([§26 of its validation rules](https://github.com/ascribed-dev/ascribe/blob/eb7e077/docs/content/contracts/content-model/validation.md#26-decisions)), 28 September 2026, as recommended there; a file of its own since [issue #689](https://github.com/ascribed-dev/ascribe/issues/689).
