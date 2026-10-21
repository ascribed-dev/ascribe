---
section: The language
---

**A frontmatter field accepts phrases only when its table form sets `phrases = true`, on `string` and `list(string)` fields; none does by default.** SPEC §5.1 leaves the choice to the content model. _Why:_ the setting lives on the field, not in `[phrases]`, so that a phrase key can never collide with a setting's name. _From:_ the content model contract's decisions, item 20 ([§26 of its validation rules](https://github.com/ascribed-dev/ascribe/blob/eb7e077/docs/content/contracts/content-model/validation.md#26-decisions)), 28 September 2026, as recommended there; a file of its own since [issue #689](https://github.com/ascribed-dev/ascribe/issues/689).
