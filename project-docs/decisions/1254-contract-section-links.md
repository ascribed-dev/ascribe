---
section: The docs and the site
---

**A contract page that cites a numbered section of another contract links to its heading, so a renumbering that leaves the citation stale fails `ascribe check`; outside the docs, where a link can't go, the number is kept right by hand.** Correcting such a citation changes no contract's meaning: it's recorded by this decision, and needs none of its own. _Why:_ commit 0b5ec798 renumbered the content-model contract and left plain-text `§` numbers in other contracts pointing at the wrong sections, which no check could catch. _From:_ Kyle, in [#1254](https://github.com/ascribed-dev/ascribe/issues/1254) and on its pull request, 3 November 2026, from the docs review of that day; [#1319](https://github.com/ascribed-dev/ascribe/issues/1319) linked the citations #1254 left as text.
