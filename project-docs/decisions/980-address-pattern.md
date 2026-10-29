---
section: The language
---

**An API's `reference` that isn't `{ renderer = "redoc", at = "…" }` is an address pattern, not a template: SPEC §5.2, §7.11, and §8.2's Links row say so, and so do the messages of `link-operation-form-unsupported` (ASC274) and `model-api-reference` (ASC260), and the docs.** `ascribe.toml` is unchanged: `reference` is still a string with `{operationId}` and `{tag}`. The code's names, such as `ApiReference::Template`, aren't a user's and stay. _Why:_ "template" also names a content type's page template (`[types.<name>] template`, `template-placeholder-left`) and the `ascribe templates` commands, so a writer reading ASC274 beside ASC195 had to work out that they're unrelated. _From:_ Kyle, [#980](https://github.com/ascribed-dev/ascribe/issues/980), 26 October 2026.
