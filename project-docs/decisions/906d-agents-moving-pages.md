---
section: The binary
---

**A project with `ascribe.addresses` has its agents' rules say how to move, forward, and retire a page, the skill says so for every project, and the MCP server's `ascribe_addresses_move` and `ascribe_addresses_retire` return the edit to apply.** The rules' "Moving pages" section, after the links', names `ascribe rename`, `ascribe addresses move --from-git` after `git mv`, and `move` or `retire` for a page deleted on purpose. The tools take the commands' arguments, write nothing, and return the commands' JSON with `written: false` and `edits`, so every tool stays read-only. _Why:_ an agent reorganizing pages with `git mv` is the likeliest to lose addresses, and one without a shell needs the edit rather than a command (decision #904). _From:_ [#906](https://github.com/ascribed-dev/ascribe/issues/906), [redirects, phase 2](https://github.com/ascribed-dev/ascribe/blob/9162c25a/project-docs/redirects/phase-2-decisions.md), task 6, 25 October 2026.
