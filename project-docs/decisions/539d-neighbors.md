---
section: The npm packages
---

**`neighbors(entry, entries, options)` in `@ascribed/astro/content` gives a page's `{ previous?, next? }`, in reading order.** Without `nav`, it reads the build's `_ascribe/nav.json`, in the entry's line for a versioned build, and walks it depth first: a section's own page, then its pages, each nested section after the page beside its folder, the others after the pages. With `nav`, it follows the site's own list, its groups' pages in order. Each neighbor is a sidebar page, with its entry, address, and label. The home page has no previous, and a page not in the order has neither. _Why:_ previous and next are the sidebar read in a line, so they come from the same navigation and the same options. _From:_ the maintainer, on the navigation plan ([phase 1](../navigation/phase-1-decisions.md), decisions 2 and 7; [README](../navigation/README.md) decisions 19 and 24), 16 October 2026; recorded with [#539](https://github.com/ascribed-dev/ascribe/issues/539).
