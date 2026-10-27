---
section: The language
---

**A check that reads a page's description or review date finds the field by its `role`, which the content type sets on it.** A type without the role opts out of the check (SPEC §7.2). *Considered:* a `field` setting in each check's `[checks]` table, and fixed names such as `description`. _Why:_ a project names its own fields: a `field` setting names one field for every type, so two types with different names for their description can't both be checked, and a type without the field is reported as missing it; fixed names take a name from every project. _From:_ the content model contract's decisions, item 23 ([§26 of its validation rules](https://github.com/ascribed-dev/ascribe/blob/eb7e077/docs/content/contracts/content-model/validation.md#26-decisions)), with release 0.3.0, 11 October 2026; a file of its own since [issue #689](https://github.com/ascribed-dev/ascribe/issues/689).

The role `review-date` is `recheck-date`, and `review-overdue` is `recheck-overdue`, since decision [#953](953-one-meaning-per-word.md); `review-date` still names the role for a release.
