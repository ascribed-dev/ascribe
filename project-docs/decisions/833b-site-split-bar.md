---
section: The look
---

**On the site, a note's bar is the gutter's split bar; the element library's default stays one bar.** `site/src/styles/site.css` draws every note type's start border as two marks in the note's color, with the page showing in the gap between them, as board 11 draws a tip, and the landing page's pillars carry the same mark before their headings. The library's stylesheet, the markup, and the `--ascribe-*` properties don't change. _Why:_ the change marks are the brand's device, and the site is where the brand is; a user's site takes the library's plain bar, and making the split bar a default would need a property to turn it off, a contract addition for a look only Ascribe's own site wants. _From:_ the maintainer, 21 October 2026; [the website plan](https://github.com/ascribed-dev/ascribe/blob/58afc492/project-docs/website/README.md#decisions), decision 9, carried out in #831.
