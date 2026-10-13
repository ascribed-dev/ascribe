---
section: The language
---

**"Tested with" is in the element library's stylesheet only: the site's HTML holds the environment alone, with no hidden lead-in for agents or parity checks.** The caption's `.ascribe-code-tested` line is a block of its own, and the plain output, which is the page's Markdown version, writes "Tested with" and the same line under the block. _Why:_ the parity check agents' tools run (`afdocs`' `markdown-content-parity`) splits the HTML into lines at block elements and looks for each line of 20 or more characters in the Markdown; the tested line, alone in its block, is always inside the Markdown's sentence, so it passes without a second copy of the words in the markup, which a site would then have to translate twice. _From:_ Kyle, in the project thread for [issue #459](https://github.com/ascribed-dev/ascribe/issues/459), taking the recommendation on its pull request, 13 October 2026.
