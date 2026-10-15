---
section: The npm packages
---

**`<ascribe-availability-target>` takes a boolean `planned` attribute, written `planned=""`, for a target whose version is in a planned line; the library styles it as the badge of no state with a dashed outline, and its text puts the planned label in place of the state.** A single version reads `Self-managed (planned, 3.6)`; in a history, the planned step reads `planned 3.6` and the rest as before: `Self-managed (preview 3.5, planned 3.6)`. Frontmatter for Astro carries `planned: true` on the target. A site restyles it with `ascribe-availability-target[planned]`. _Why:_ planned is a fact about the version, not one of the project's lifecycle states, so it's an attribute beside `states` rather than a state, and it stops when the line moves to `live` with no edit to the page. _From:_ the implementor of [#496](https://github.com/ascribed-dev/ascribe/issues/496), in its pull request, 15 October 2026, for the maintainer's review.
