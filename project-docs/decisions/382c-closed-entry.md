---
section: Process
---

**A closed entry shows its bold text when that's a sentence of its own, and otherwise its first sentence.** So an entry whose bold text is only **Fixed:** shows `Fixed:` and the sentence after it, as does one whose bold text starts a sentence ("**`@snippet`** puts…") or that has none; an entry that's all summary isn't collapsed. Fixes go in the `fixes/` area and keep writing **Fixed:** as their bold text, and the limit of decision #382b counts the sentence after it. `summaryOf` in `scripts/release/changes.ts` and the page's `collapseEntries` in `site/src/changelog.ts` apply the rule, to the Markdown and the rendered page. _Why:_ fourteen unreleased fixes would otherwise all collapse to "Fixed:", and their first sentence already says what was fixed, so no entry's words had to change. _From:_ issue #382, 12 October 2026.
