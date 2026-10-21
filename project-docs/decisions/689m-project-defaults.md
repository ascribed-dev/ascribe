---
section: The language
---

**`[project]` defaults to `content-root = "docs"` and `output-dir = ".ascribe/build"`, with paths relative to `ascribe.toml`, `..` allowed, and absolute paths rejected.** _Why:_ relative paths keep a project portable, and the content root can't default to `"."`, since the output directory couldn't then sit outside it. _From:_ the content model contract's decisions, item 14 ([§26 of its validation rules](https://github.com/ascribed-dev/ascribe/blob/eb7e077/docs/content/contracts/content-model/validation.md#26-decisions)), 28 September 2026, as recommended there; a file of its own since [issue #689](https://github.com/ascribed-dev/ascribe/issues/689).
