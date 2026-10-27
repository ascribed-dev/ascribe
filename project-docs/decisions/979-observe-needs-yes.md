---
section: The binary
---

**Without a terminal, `ascribe observe` lists the config's commands and runs nothing unless given `--yes`.** It exits 2, not the 1 `--check` gives for a stale observation, and its message names `--yes`. `--check` runs nothing, so it needs no `--yes`, and it gives no consent to a run after it. CI's runs of `observe`, `pnpm bless`, AGENTS.md, and CONTRIBUTING.md pass `--yes`, after the commands `ascribe.examples.toml` names have been read. _Why:_ the question exists for the case where nobody is watching, such as an agent in a checkout whose `ascribe.examples.toml` someone else wrote, and running silently there is the wrong default. Before 1.0, a behavior change whose fix is one flag the error names is worth it. _From:_ the maintainer, in [issue #979](https://github.com/ascribed-dev/ascribe/issues/979), 26 October 2026.
