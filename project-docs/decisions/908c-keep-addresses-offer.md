---
section: The binary
---

**After the editor moves a page of a project without `ascribe.addresses`, it offers Keep the addresses you've published once for the project, and accepted, runs `ascribe addresses init` and then `ascribe addresses move` for each page the move took.** The offer is remembered per folder in the workspace's state, whether it was accepted or dismissed, and isn't made without a binary to run. _Why:_ `init` alone records nothing, so the move that prompted the offer would be lost the moment the file exists; asking more than once would nag a project that decided not to keep its addresses. _From:_ [#908](https://github.com/ascribed-dev/ascribe/issues/908), [redirects, phase 4](https://github.com/ascribed-dev/ascribe/blob/9162c25a/project-docs/redirects/phase-4-editor.md), 25 October 2026.
