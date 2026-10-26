---
section: The binary
---

**`ascribe init` writes an empty `ascribe.addresses` beside `ascribe.toml`, in an empty folder and in one with pages, and leaves one already there as it is; and with no `--base`, a project outside a git repository, or in one with no commit yet, compares no addresses and isn't told so.** A project with history and no default branch, or a base named that can't be read, is still `addresses-base-unreadable`, so a check in CI that compared nothing never passes silently. _Why:_ renaming a page is routine, so a new project is protected without knowing to ask; and before the first commit nothing can have been published, so the warning would greet every new project, Getting started's included, with nothing to act on. _From:_ [#1016](https://github.com/ascribed-dev/ascribe/issues/1016), the technical writer review's finding 4; the quiet first check proposed on the issue, 26 October 2026.
