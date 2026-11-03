---
section: The docs and the site
---

**The docs' sidebar orders its guide sections along the path a team takes to adopt Ascribe:** Writing pages, Templates, The content model, The editor, Checks and reports, Review, Navigation, Publishing with Astro, Publishing, Versions, Permalinks, Examples, OpenAPI, Agents, and The command, after the pages at the root and before the contracts, which keep their place at the end. `docs/ascribe.nav` holds the order, and `site/test/fixtures/sidebar.json` the sidebar it makes. The feature folders stay as they are, and a section is only moved, never renamed, by this decision. _Why:_ the sections were in the order they were written, so Templates, which a writer needs on their first page, came 14th of 15, and Navigation, which every site needs, came after Permalinks. _From:_ the maintainer, in [issue #1277](https://github.com/ascribed-dev/ascribe/issues/1277), 3 November 2026.
