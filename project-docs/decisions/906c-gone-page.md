---
section: The npm packages
---

**A retired address gets a page saying why, at `/_ascribe/gone` and its address, which the host serves at the address with a 404.** `redirects.json`'s `gone[]` gains `page`, the same for a page's address and its Markdown version's; Netlify's `_redirects` gains `<path> <page> 404` for each, after the pairs. `@ascribed/astro` adds one route, `/_ascribe/gone/[...address]`, when any build lists one, building a page for each with the reason as text, a link to its project's root, and `noindex`, and leaves the pages out of the sitemap. The markup is the integration's; another site generator reads `gone` and writes its own. _Why:_ a 404 tells a search engine to drop the address, and a page saying why tells the reader; serving it with a rewrite keeps the address the reader asked for. _From:_ [#906](https://github.com/ascribed-dev/ascribe/issues/906), [redirects, phase 2](../redirects/phase-2-decisions.md), task 5, 25 October 2026.
