---
type: how-to
title: Keep pages current
description: "Give each page an owner and a date it's next due to be checked, see the overdue pages in `ascribe check` and the editor, count pages by owner with `ascribe report`, and run the report on a schedule in CI."
---

"Who owns this page, and when was it last checked?" Ascribe answers both from two fields of a page's frontmatter: its owner, such as a team, and its recheck date, the day it's next due to be read against what it describes. `ascribe check` and the editor report a page whose recheck date has passed, and `ascribe report` counts the pages by owner and lists the overdue ones.

This guide adds the two fields, shows where an overdue page is reported, and runs the report every week in CI. It's for a project that has `ascribe.toml` already, as [Getting started](../getting-started.md) sets one up.

## Add the two fields

@steps
1. In `ascribe.toml`, add an `owner` field and a `recheck-date` field to the frontmatter of the page types you want to keep current. A field named for its [role](index.md#54-field-roles) has it, so these names need nothing more:

   ```toml
   [types.page.frontmatter]
   title = "string"
   description = "string"
   owner = "enum(docs, platform)?"
   recheck-date = "date?"
   ```

   An `enum` lists the owners a page can name, so a misspelled team is an error; a `string` takes any name. Leave out the `?` to make every page of the type say it. To give the fields to every type at once, add them to `[frontmatter]` instead.
2. On each page, write who owns it and when it's next due, as a date, `YYYY-MM-DD`:

   ```yaml
   ---
   type: page
   title: Install the agent
   description: Install the agent on a host.
   owner: platform
   recheck-date: 2027-03-01
   ---
   ```

A field of another name, such as one your pages already have, takes the role with `role`: `team = { type = "string?", role = "owner" }`. [Field roles](index.md#54-field-roles) has the rules.

## See the overdue pages

Once a page's recheck date is before today, [`recheck-overdue`](../checks/diagnostics/content-checks.md#ASC153) reports it as advice, at the date:

```text
$ ascribe check --code recheck-overdue --format concise
docs/guides/install.md:6: [ASC153] this page was due for a recheck on 2026-09-01; check it still holds, then move `recheck-date` to the next recheck
checked 3 files for every build, 1 with problems for code `ASC153`: 0 errors, 0 warnings, 1 advice
```

The check compares the date with today's, in UTC, so a page becomes overdue without a change to the project. To see what falls due by another day, such as the end of next month, set `ASCRIBE_TODAY`:

```sh
ASCRIBE_TODAY=2026-12-31 ascribe check --code recheck-overdue --format concise
```

In VS Code, the sidebar shows the same thing in two places:

- **This page**'s Health section has a **Review** row: when the page is next due, in the warning color once it's overdue ([This page's rows](../editor/sidebar.md#this-pages-rows)).
- **The Project view**'s Health section counts **Rechecks overdue** across the project, after **Check Every Build**, and opens the first of them ([The Project view](../editor/sidebar.md#the-project-view)).

## Recheck a page

Read the page against what it describes, and correct what's changed. Then move its `recheck-date` to when it's next due, such as six months on. When the page is right as it is, moving the date is the whole fix.

Advice doesn't fail `ascribe check`, so an overdue page never fails a pull request on its own. To make it louder, set its level in [`[checks]`](checks.md#22-checks), `recheck-overdue = "warning"`; a CI job that runs `ascribe check --deny-warnings`, as [Checks in CI](../checks/ci.md) sets one up, then fails every pull request on the day a page falls due, whatever the pull request changes. The schedule below tells the owners without blocking anyone.

## Count pages by owner

`ascribe report inventory` counts the project's pages by type and by owner, and lists the overdue rechecks:

```text
$ ascribe report inventory
Inventory
  3 pages.
  By type: page 3.
  By owner: docs 1, platform 1, no owner 1.
  Overdue rechecks (1):
    docs/guides/install.md:6: [ASC153] this page was due for a recheck on 2026-09-01; check it still holds, then move `recheck-date` to the next recheck
  Pages nothing links to: none.
  Entries nothing uses: none.
```

"No owner" counts the pages that leave the field out, and the pages of types without one. The list shows 20 at most, and says how many it left out; `--limit 500` shows up to 500. With `--format json`, the counts are `inventory.by_owner`, and the overdue pages `inventory.overdue`, as [`ascribe report`](../checks/report-command.md#cli-report) has them. [The project report](../checks/report.md#inventory) says what the rest of the inventory counts.

## Run the report on a schedule

This GitHub Actions job runs the inventory every Monday and writes it to the run's summary. When a page is overdue, it opens an issue, or updates the one that's open, so the owners see the list without anyone running the report. Change `--config docs` to your project's folder, or drop it when that's the repository's root.

```yaml
name: Rechecks

on:
  schedule:
    - cron: "0 6 * * 1"
  workflow_dispatch:

permissions:
  contents: read
  issues: write

jobs:
  rechecks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
      - uses: actions/setup-node@949feb2413d6458794dcd2491c4babbbce0c15c1 # v7.1.0
        with:
          node-version: {node}
      - run: npm install --global @ascribed/cli
      - run: ascribe report inventory --config docs --limit 500 --format markdown > report.md
      - run: cat report.md >> "$GITHUB_STEP_SUMMARY"
      - name: Open or update the issue
        env:
          GH_TOKEN: ${{ github.token }}
          TITLE: "Docs: pages due for a recheck"
        run: |
          overdue=$(ascribe report inventory --config docs --format json | jq '.inventory.overdue.total')
          if [ "$overdue" -gt 0 ]; then
            number=$(gh issue list --state open --search "in:title \"$TITLE\"" --json number --jq '.[0].number')
            if [ -n "$number" ]; then
              gh issue edit "$number" --body-file report.md
            else
              gh issue create --title "$TITLE" --body-file report.md
            fi
          fi
```

- **The count** comes from the report's JSON, not its exit code: `--exit-code` fails on the `problems` section's findings, and the inventory's overdue list isn't one of them.
- **The issue** is found by its title, so one stays open until someone closes it, and each run replaces its body with the latest list. Closing it once every page is rechecked is up to you; the next overdue page opens a new one.
- **The project's report** runs other sections every day, the external links and the published site among them: [On a schedule](../checks/report.md#on-a-schedule) has that job, and this one's step can go in it.

## See also

- [Field roles](index.md#54-field-roles): the rules for `owner`, `recheck-date`, and `description`.
- [`recheck-overdue`](../checks/diagnostics/content-checks.md#ASC153): the check, and acknowledging it on a page that's meant to stay as it is.
- [The project report](../checks/report.md): every section of `ascribe report`, and acting on each.
