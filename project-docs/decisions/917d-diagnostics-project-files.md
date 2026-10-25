---
section: The docs and the site
---

**The diagnostics reference has a seventh page, "the addresses file and API specs", for the source-file diagnostics from the "Addresses file" construct on, so "links, pages, and sources" stays under the 50,000 characters an agent reads in one piece.** The split is a construct in `tests/conformance/tests/docs.rs`, `FIRST_ON_PROJECT_FILES`, as the others are. The addresses file's diagnostics moved with their permalinks, which `ascribe permalinks sync` recorded. _Why:_ the OpenAPI diagnostics took the page over the limit, and `page-size` reported it. _From:_ [#917](https://github.com/ascribed-dev/ascribe/issues/917), 25 October 2026.
