---
section: Process
---

**A changelog entry is one line: a bold headline, then one or two sentences, 40 words at most, ending with a link to the page with the detail.** A fix is **Fixed:** and a sentence or two within the same 40 words, and links to a page when one describes it. No entry's headline, or a fix's first sentence, has a link, since the changelog page puts it in the `<summary>` that opens the entry. `changes.test.ts` holds `changes/` and CHANGELOG.md's entries from 0.4.4 on to the shape, and every entry in CHANGELOG.md to a headline with no link; older releases keep their longer entries. _Why:_ the changelog had grown to about 36,000 words, longer than the spec, because entries restated their pages; a reader skims it to learn what changed, and a link inside a `<summary>` is a nested control that axe reports. _From:_ [#970](https://github.com/ascribed-dev/ascribe/issues/970), from the clarity and noise review, 26 October 2026.
