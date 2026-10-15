---
section: Process
---

**A changelog entry declares its area by its folder: `changes/<area>/<name>.md`.** The areas are `AREAS` in `scripts/release/changes.ts`, each a folder and the `###` heading its entries are gathered under, in order: the language, the `ascribe` command, the editor, the language server, Astro and the elements, agents, review, the docs and the site, installing, and fixes. `gather` puts each entry at the top of its area's heading in the unreleased section, adding the heading where the order puts it, and the check refuses an entry in no area's folder. The entries `CHANGELOG.md`'s unreleased section already had were sorted under the headings once, by hand, with no entry's words changed. _Why:_ the changelog page groups a release's entries by area, so each entry has to say its area; a folder keeps every entry file a plain list item, where frontmatter would change what each one looks like. _From:_ issue #382, recommended there, 12 October 2026.
