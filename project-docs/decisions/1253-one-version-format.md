---
section: The binary
---

**`ascribe_version` in every command's JSON is the plain release, such as `0.4.5`, with no field for the build.** `init`, `new`, `templates`, and `observe` wrote the full `0.4.5+dev (0655d2f50120)` from a binary built from a checkout, and now write the release, as every other command already did. Each schema describes the field as "The release of Ascribe that wrote it, as `0.4.5`." The build's details stay where a person asks which binary they have: `ascribe --version`, the MCP server's `serverInfo`, and the language server's. _Why:_ the release is what a reader installs and what a tool compares, and a field with two formats can't be compared at all; nobody reading the JSON has needed the commit, which exists only on a binary built from a checkout. _From:_ Kyle, in [#1253](https://github.com/ascribed-dev/ascribe/issues/1253), 3 November 2026, from the docs review of that day.
