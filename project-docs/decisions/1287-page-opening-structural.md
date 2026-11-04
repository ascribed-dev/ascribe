---
section: The language
---

**`page-opening-missing` is structural: it reports a page whose first block after its title isn't a paragraph written in the page, such as an `@include`, a heading, a list, a table, or code, and never judges the paragraph's words.** A level-1 heading that repeats the title, a memo, an HTML comment, and a directive bound to the block below it, such as `@summary` or `@available`, are passed over; fragments aren't checked, and `[checks]` sets its level. Openings that are only a pointer, such as "Part of …", are left to a project's prose rules, as this repository's Vale rules hold them. _Why:_ telling a sentence from a fragment with no main verb needs parsing English, and a check every project's pages get, in any language, can't guess at it without being noisy. _From:_ Kyle, [#1287](https://github.com/ascribed-dev/ascribe/issues/1287), 3 November 2026.
