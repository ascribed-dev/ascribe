---
section: The language
---

**A pattern's `{a,b}` groups expand to at most 256 patterns.** The count is the product of each group's alternatives, so `{a,b}/{x,y,z}` is 6 and eight two-way groups are 256; a pattern past it isn't valid, and `model-pattern-syntax` says why, wherever patterns are compiled: `[sources]`, `[sections]`, `[fragments]`, and `ascribe.examples.toml`. The content model's conventions say so in the `{a,b}` row. _Why:_ a pattern compiles to every combination of its alternatives, so each two-way group doubles the work, and the nightly fuzz run found a 1 KB `include` pattern with sixteen groups that didn't finish loading. Matching each group lazily would accept every pattern, but options can hold `/` and `**`, so it would mean a second matcher for no pattern anyone writes; 256 is far past the few groups real patterns have. _From:_ triage, at the maintainer's request, in [issue #1401](https://github.com/ascribed-dev/ascribe/issues/1401), 22 November 2026.
