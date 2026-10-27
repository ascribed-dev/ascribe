---
section: The language
---

**`example-stale` is a warning by default, which `ascribe check --deny-warnings` fails on; `example-unobserved` stays advice.** It's still configurable in `[checks]`, so a project can set it back to `"advice"`, and it's reported only for a snippet whose file has an observation, so only a project that set up tested examples sees it. _Why:_ a project that ran `ascribe observe` asked to know when an example changes after its test run, and CI's recipe passed anyway: the page quietly lost its "Tested with" line, and an expected output beside it could be wrong. Ascribe's own docs raised it to an error for this reason. An unobserved example lost nothing it had, so it stays advice. _From:_ Kyle, [#1010](https://github.com/ascribed-dev/ascribe/issues/1010), 26 October 2026.
