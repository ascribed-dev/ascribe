---
section: The language
---

**Two glossary terms with the same id are an error at the second, which isn't a term, as two with the same text are.** A term's id names it everywhere else (glossary pages plan, decision 2): a link to its heading, `ascribe refs term:<id>`, and a build's map from ids to terms, so two terms can't share one. `glossary-term-duplicate` reports it, with a second wording (`id`), rather than a new diagnostic, since its fix is the same kind: make the terms distinct. SPEC §5.4 gains the rule, and its row in §8.2 the case. The text is checked first, so two terms that share both are reported as sharing their text, as before. _Why:_ `ascribe check` was clean, and a build linked one term's words to the other's heading and definition. _From:_ [#605](https://github.com/ascribed-dev/ascribe/issues/605), found reviewing the glossary pages plan after its phases merged, 18 October 2026.
