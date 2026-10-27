---
section: The binary
---

**Every page of an archive, and of `ascribe preview`'s folder, is `<html lang="en">`, and starts with a "Skip to content" link to its `<main id="archive-main">`, shown when it has focus.** The archive contract (§2) says so. The content model has no setting for a project's language, so `en` is fixed, as the review report and the permalink pages already write it; once a project can declare its language, the archive takes it from there, and adding that setting is a decision of its own. _Why:_ a page with no `lang` leaves a screen reader to guess its language, the one thing axe found on the archive, and at a phone's width every page opened with the search box and the whole list of pages before its content. _From:_ Kyle, [#1017](https://github.com/ascribed-dev/ascribe/issues/1017), 26 October 2026.
