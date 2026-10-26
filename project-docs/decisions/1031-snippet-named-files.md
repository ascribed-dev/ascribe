---
section: The language
---

**SPEC §4.8 matches a code file by its whole name before its extension, and takes tags in one-line block comments.** A table of names gives `Dockerfile`, `Containerfile`, `Makefile`, `GNUmakefile`, `Justfile`, `.env`, `.env.*`, and `.gitignore` `#` comments and the language a snippet of one defaults to (`dockerfile`, `makefile`, `just`, `dotenv`, `gitignore`); names and extensions are compared without regard to case. `graphql`, `gql`, `properties`, `mk`, and `dockerfile` join the `#` extensions. `css` takes tags in `/* … */` on a line of its own, and `scss` and `less` in that or `//`. A test holds `crates/ascribe-resolve/src/snippet/tags.rs` to the spec's tables. _Why:_ it's additive, the spec is a draft until 1.0, and these are files engineers quote every day, which could only be shown whole. _From:_ Kyle, [#1031](https://github.com/ascribed-dev/ascribe/issues/1031), 26 October 2026.
