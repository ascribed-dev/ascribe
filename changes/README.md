# Unreleased changes

Each change a user would notice adds its changelog entry here, as a file of its own, instead of editing [CHANGELOG.md](../CHANGELOG.md): pull requests then never change the same lines of the changelog, so they don't conflict there. A release gathers the files into the changelog's unreleased section and deletes them (decision 123 in [project-docs/decisions.md](../project-docs/decisions.md)).

**Put** the file in the folder of the change's area (decision 129). A release gathers it under that area's heading, and the site's changelog page groups the entries by them:

| Folder | Heading | For |
| --- | --- | --- |
| `language/` | The language | directives, `ascribe.toml`, the checks and what they report, the outputs' contents |
| `command/` | The `ascribe` command | the commands, their options, text, and JSON |
| `editor/` | The editor | VS Code: the preview, the sidebar, actions, settings |
| `language-server/` | The language server | what the server answers, for any editor |
| `astro/` | Astro and the elements | `@ascribed/astro` and `@ascribed/elements` |
| `agents/` | Agents | agent instructions, the skill, the MCP server, hooks, what agents read |
| `review/` | Review | `ascribe diff`'s report, review in the preview and the site, `@ascribed/review` |
| `docs/` | The docs and the site | a guide or reference page, the site at ascribed-dev.com |
| `installing/` | Installing | the binaries, the npm packages' installation and exports, the canary |
| `fixes/` | Fixes | an entry whose bold text is **Fixed:** |

A change that spans areas goes where a reader would look for it first. A new area is a decision: it's added to `AREAS` in `scripts/release/changes.ts`, which a test holds this table to.

**Name** the file for the change, in lowercase words joined by `-`, such as `check-fix.md`. Any name that isn't taken will do; it only sets the order the entries are gathered in, within their area.

**Write** the entry as CHANGELOG.md writes one: a list item that starts with `- ` and a bold sentence saying what changed, then the rest. Lines under the item are indented by two spaces. A link is written relative to the repository's root, as the changelog's are: a docs page is linked by its source, such as `docs/content/guides/permalinks.md`, and the site's changelog page points it at the page.

```markdown
- **`ascribe check --fix` applies the safe fixes.** Every fix `ascribe check` marks `safe` is applied, then the check reports what's left.
```

**Keep the bold sentence to 100 characters** as it reads, without the Markdown (decision 130). The changelog page shows each entry closed, as its bold sentence alone, so that sentence says what changed, and the rest goes after it. A fix's bold text is **Fixed:**, and the page shows it with the sentence after it, which the limit counts too (decision 131):

```markdown
- **Fixed:** `ascribe fmt` no longer follows a symbolic link out of the content root. A file that's a link…
```

A change that breaks something a user relies on says **Behavior change:** in its bold sentence, and the page marks it.

`pnpm test` checks each file (`scripts/release/changes.test.ts`): its name, that it's in an area's folder, that it's one or more list items with nothing that would end one early, that each starts with a bold sentence within the limit, and that its links lead to files in the repository. `node scripts/release/changes.ts check` checks them alone. The site's changelog page shows them in its unreleased section, as `main` has them.

At a release, `node scripts/release/changes.ts gather` moves them into CHANGELOG.md, each at the top of its area's heading in the unreleased section, adding the heading when there isn't one yet, and deletes their files ([RELEASING.md](../RELEASING.md)).
