# Decisions, one file each

Each decision made since the numbered ones in decisions.md is a file of its own here, instead of an entry in [decisions.md](../decisions.md): pull requests then never change the same lines, and never claim the same number (decision #381, [381-decision-files.md](381-decision-files.md)). The numbered decisions stay in decisions.md, with their numbers, and it takes no new ones.

**A pull request opened before this folder,** which adds a numbered entry to decisions.md, moves that entry into a file here, named by its issue, when it merges `main`: the check fails on a number in decisions.md above the last it had.

**Name** the file for the issue the decision was made for: its number, then a few lowercase words joined by `-`, such as `381-decision-files.md`. A decision with no issue takes its pull request's number. GitHub numbers issues and pull requests in one sequence, so no other branch takes the same one. An issue that makes more than one decision gives each a letter after the number: `381a-…`, `381b-…`.

**Cite** it as "decision #381" (or "decision #381a"), with a link to the file where a link fits. A decision in decisions.md is still cited by its number alone, as "decision 74".

**Write** it as decisions.md writes an entry, after frontmatter naming its section, which is one of decisions.md's `##` headings:

```markdown
---
section: The language
---

**The bold sentence says what was decided.** Then what it means in practice. _Why:_ the reason. _From:_ who made it and where, with the date.
```

A link is relative to the file, as in any page of `project-docs/`.

**A decision that's replaced** stays: strike its bold sentence through (`~~**...**~~`) and say which decision replaced it, with a link. Its file isn't deleted, and its number isn't reused.

**A plan's own decisions** stay in its `README.md` while it runs, numbered within the plan. Those that still bind the work after the plan finishes become files here when it closes, each numbered by the plan's issue or the pull request that closes it.

**Every decision about one section** is in two places: that section of decisions.md, and the files here that name it. `node scripts/repo-docs/decisions.ts list` prints the files' decisions under their sections, and `node scripts/repo-docs/decisions.ts list "The language"` one section's. The list isn't written into decisions.md, since a list every pull request rewrites would bring back the conflict these files remove.

`pnpm test` checks each file (`scripts/repo-docs/decisions.test.ts`): its name, that its number is above decisions.md's last, that decisions.md has no number above that, that no two files share a number without letters, that its section is a known one, that it has the bold sentence, a _Why:_, and a _From:_, and that its links lead to files in the repository. `node scripts/repo-docs/decisions.ts check` checks them alone.
