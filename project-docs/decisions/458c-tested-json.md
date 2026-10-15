---
section: The binary
---

**The JSON output says whether an example was tested in a `tested` key on its block's `snippet`.** It's absent when the file isn't an example ([decision #458e](458e-what-is-an-example.md)). It holds `status`, one of `current`, `stale`, or `unobserved`, and `observations`, the current ones only, each with its `name` and its `environment` as the observation has it: the kept first line, such as `"node": "v24.9.0"`, not the shown number. Labels aren't in it. _Why:_ a reader of the JSON parses versions its own way, so it gets what was recorded rather than what a page shows. Labels are in the config, and a later key can carry them without changing these. _From:_ Kyle, on [issue #458](https://github.com/ascribed-dev/ascribe/issues/458), taking [phase 1](https://github.com/ascribed-dev/ascribe/blob/180eee83/project-docs/tested-examples/phase-1-decisions.md)'s recommendation, 13 October 2026.
