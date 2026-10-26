---
section: The language
---

**A line shaped like a directive whose name is a known keyword in another case, such as `@Note:`, draws `directive-unknown` with the keyword as its suggestion (SPEC §3.2).** Names stay case-sensitive, so the line is still text; only the warning is new. A capitalized word that isn't a keyword in lowercase, such as `@Warning:`, still draws none. _Why:_ `@notes:` was warned about and `@Note:` wasn't, so a writer's capital letter made a callout plain text without a word. _From:_ Kyle, [#1034](https://github.com/ascribed-dev/ascribe/issues/1034), 26 October 2026.
