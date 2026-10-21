---
section: The language
---

**Built-in lifecycle states and note types can be relabeled, and states' `available` flags changed, but none removed.** `ga` stays available; a new state sets `available` explicitly; a new note type needs a `label`; and built-in states take part in the one-role rule. _Why:_ content with no state is `ga` (SPEC §4.4), so `ga` has to count as available; none was recorded for the rest. _From:_ the content model contract's decisions, item 17 ([§26 of its validation rules](https://github.com/ascribed-dev/ascribe/blob/eb7e077/docs/content/contracts/content-model/validation.md#26-decisions)), 28 September 2026, as recommended there; a file of its own since [issue #689](https://github.com/ascribed-dev/ascribe/issues/689).
