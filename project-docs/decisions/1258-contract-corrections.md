---
section: The docs and the site
---

**The contract pages say what the code already does where #1258 found them wrong, and the code doesn't change to match the old words.** The agents' `llms.txt` links are absolute URLs with no `../`; `nav.json`'s `source` is a field of a page record, and `pages.json` is keyed by route; the page beside a top folder is listed twice, as one beside a nested folder is; `headers.json` is empty only when a build publishes no Markdown versions and no permalinks; retired and moved permalinks live in `ascribe.addresses`, not `[permalinks]`; `ascribe-operation` carries an anchor; and the JSON reports' lists name `ascribe new` and `ascribe rename`. _Why:_ each statement disagreed with the binary, which consumers already rely on, and the subpages and SPEC already said what the code does, so the pages were the ones in error. _From:_ Kyle, in [#1258](https://github.com/ascribed-dev/ascribe/issues/1258), 3 November 2026, from the docs review of that day.
