# Unreleased changes

Each change a user would notice adds its changelog entry here, as a file of its own, instead of editing [CHANGELOG.md](../CHANGELOG.md): pull requests then never change the same lines of the changelog, so they don't conflict there. A release gathers the files into the changelog's unreleased section and deletes them (decision 123 in [project-docs/decisions.md](../project-docs/decisions.md)).

**Name** the file for the change, in lowercase words joined by `-`, such as `check-fix.md`. Any name that isn't taken will do; it only sets the order the entries are gathered in.

**Write** the entry as CHANGELOG.md writes one: a list item that starts with `- ` and a bold sentence saying what changed, then the rest. Lines under the item are indented by two spaces. A link is written relative to the repository's root, as the changelog's are: a docs page is linked by its source, such as `docs/content/guides/permalinks.md`, and the site's changelog page points it at the page.

```markdown
- **`ascribe check --fix` applies the safe fixes.** Every fix `ascribe check` marks `safe` is applied, then the check reports what's left.
```

A change that breaks something a user relies on says **Behavior change:** in its bold sentence.

`pnpm test` checks each file (`scripts/release/changes.test.ts`): its name, that it's one or more list items with nothing that would end one early, and that its links lead to files in the repository. `node scripts/release/changes.ts check` checks them alone. The site's changelog page shows them in its unreleased section, as `main` has them.

At a release, `node scripts/release/changes.ts gather` moves them into CHANGELOG.md, at the top of the unreleased section, and deletes their files ([RELEASING.md](../RELEASING.md)).
