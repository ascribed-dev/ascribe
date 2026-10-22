---
section: The website
---

**The landing page is an Ascribe project, written with widgets, and every link on it is a permalink the site's tests check.** `site/home/` holds it, and its layout is said in widgets in `site/home/ascribe.toml`, drawn by `site/src/styles/site.css` from the tokens. A layout a widget can't say is noted in the brainstorm's section on widgets, and the page stays in Ascribe. Every example on it is a file read with `@snippet`, a command's tested output, or a screenshot the script takes. _Why:_ the landing page is the first marketing page built with Ascribe, so what it needs and can't say is evidence for the product; real examples and checked permalinks keep it true when the code and the docs move. _From:_ the maintainer, 21 and 22 October 2026; [the website plan](https://github.com/ascribed-dev/ascribe/blob/58afc492/project-docs/website/README.md#decisions), decisions 3 and 15, carried out in #817.
