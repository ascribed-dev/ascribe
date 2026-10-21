---
section: The language
---

**A widget's plain output is a static CommonMark string with phrases substituted, and the content a widget wraps is kept unless it sets `plain-content = "drop"`.** The string takes no attribute values (no behavior). SPEC §6 and §9.4's "or nothing" applies to the widget itself, not to its content. _Why:_ silently losing content is worse than showing it. _From:_ the content model contract's decisions, item 8 ([§26 of its validation rules](https://github.com/ascribed-dev/ascribe/blob/eb7e077/docs/content/contracts/content-model/validation.md#26-decisions)), 28 September 2026, as recommended there; a file of its own since [issue #689](https://github.com/ascribed-dev/ascribe/issues/689).
