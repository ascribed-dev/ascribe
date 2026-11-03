---
type: how-to
title: Bring existing docs to Ascribe
description: "Start a project in a folder of Markdown you already have, and take it from `ascribe init` to a clean `ascribe check`."
permalink: existing-docs
---

Ascribe reads plain CommonMark, so a folder of Markdown is most of a project already. This guide starts a project around one, fixes what the first check finds, and shows what to rewrite in pages written for another tool: links, frontmatter, callouts, and heading ids.

The pages here are [`examples/existing-docs`]({repo}/tree/main/examples/existing-docs): four pages in `docs/`, some with frontmatter and some without, and one link written to the `.html` file the old site published.

## Before you start

- Install the command, as [Getting started](getting-started.md#install-the-command) does.
- Commit the pages first, so that `git diff` shows each change Ascribe makes.

## Start the project

In the folder that holds `docs/`:

```sh
npx ascribe init
```

When the pages are in another folder, name it, from the folder you run `init` in: `npx ascribe init --content-root guides`. It's the folder the pages are in, not the project's own, so the build's output stays out of it.

`init` writes `ascribe.toml`, counts the pages it found, and checks them:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/init-existing.txt

The `ascribe.toml` it writes for a folder of pages declares no page types. Every page is then of the one implicit type, `page`, and none of them has to say `type:` in its frontmatter. Each needs a `title`, and can have a `description`:

@snippet: code:crates/ascribe-cli/tests/output/init-existing-ascribe.toml

In an empty folder, `init` declares the type `page` instead, which each page names; [`ascribe init`](install.md#cli-init) has both. `--templates`, which declares six types, is refused in a folder of pages, since each page would have to name one. Take the templates once the pages check clean, with [`ascribe templates eject`](templates/command.md#cli-templates-eject), then give each page `type:`, since the project declares types from then on.

## Fix what the check finds

@steps
1. Apply the safe fixes:

   ```sh
   npx ascribe check --fix --format concise
   ```

   A page with no `title` that opens with a level-1 heading takes the heading as its title: the fix writes it in the frontmatter, and takes the heading out of the page, since the site shows the title in its place. Here that's `index.md` and `alerts.md`. What's left is listed, one line each, as `--format concise` prints it; without it, the check shows each problem with its line and its help:

   @snippet {lang=text}: code:crates/ascribe-cli/tests/output/init-existing-fix.txt

   A fix is safe only when it can't change what the page says. A page whose opening heading something links to, as `#configure-alerts`, isn't given the heading as its title by `--fix`, since the link would lose its target; the fix is offered in the editor, for you to apply and fix the link.
2. Give the other pages a title. `troubleshooting.md` opens with a paragraph, so it gets frontmatter:

   ```markdown
   ---
   title: When a page doesn't load
   ---
   ```

3. Fix the links. A link in Ascribe names the file it goes to, and each output writes its address, so `[Install Lantern](../install.html)` becomes `[Install Lantern](../install.md)`. `ascribe check --code link-target-missing` lists every link that names no file.
4. Put the advice off. Advice never fails the check: here it's the pages with no `description`, and one about `ascribe.addresses`, which step 5 explains and the baseline leaves out. To deal with the rest later, and have the check report only what's new, record it in `ascribe.baseline`:

   ```sh
   npx ascribe baseline write
   ```

   @snippet {lang=text}: code:crates/ascribe-cli/tests/output/init-existing-baseline.txt

   See [The baseline](checks/baseline.md) for what it holds back, and how it shrinks as you fix each.
5. Check again:

   ```sh
   npx ascribe check --format concise
   ```

   @snippet {lang=text}: code:crates/ascribe-cli/tests/output/init-existing-check.txt

   No errors are left. The one advice, [`addresses-base-unreadable`](checks/diagnostics/project-checks.md#ASC257) (ASC257), says the check compared nothing with `ascribe.addresses`: the [base](publishing/redirects.md#addresses-base) it compares with has the pages and no project, so it published no address to keep. The base is where your branch left `main`, or, working on `main`, its last commit. Its help is about CI, where a base can't be read; here nothing is wrong.

   The baseline doesn't hold it back: `baseline write` doesn't compare the addresses, so it never sees this advice, and doesn't record it. The advice goes once the base has the project, as the next step shows.
6. Commit `ascribe.toml`, `ascribe.addresses`, `ascribe.baseline`, and the pages, and add `.ascribe/` to `.gitignore`. Committed on `main`, the base then has the project, and the check is clean:

   ```sh
   npx ascribe check --format concise
   ```

   @snippet {lang=text}: code:crates/ascribe-cli/tests/output/init-existing-committed.txt

   On a branch, the base is still where the branch left `main`, so the advice stays until the branch is merged, as it does in the pull request that brings the project in.

To check every pull request, see [Check in CI](checks/ci.md).

## What to rewrite in pages from another tool

The check finds the links and the frontmatter that need a change, and GitHub's alerts, such as `> [!NOTE]`, which [`github-alert`](checks/diagnostics/content-checks.md#ASC287) reports and `--fix` rewrites as `@note`. It doesn't find other tools' callouts or heading ids: they're text to Ascribe, so the site shows them as they're written. Search for them:

```sh
grep -rnE '^(:::|!!! )|\{#[^}]*\}\s*$|\[\[' docs/
```

| Written as | In Ascribe |
|---|---|
| `[Install](install.html)`, `[Install](/install/)` | `[Install](install.md)`: the file. A published address is reported, and `--fix` rewrites it; an `.html` link is reported as missing, for you to rewrite. |
| `[[Install]]`, `[[install\|Install it]]` | `[Install it](install.md)` |
| `:::note` … `:::`, `!!! note` | [`@note`](writing/directives.md#note), as `@note: One line.`, or `@note {type=warning}:` … `@end` around several blocks |
| `> [!NOTE]`, `> [!WARNING]`, and GitHub's other alerts | The `@note` of the same type. The check reports each, and `--fix` rewrites it, keeping the quote's content |
| `## Setup {#setup}` | `## Setup`, with `@id: setup` on the line below it ([`@id`](writing/directives.md#id)) |
| A frontmatter key for the sidebar's label | `nav-label`, which [every page can have](writing/frontmatter.md#reserved-keys) |
| A frontmatter key for the page's place in the sidebar | [`ascribe.nav`](navigation/order.md), which orders the pages and sections |
| Any other frontmatter key | A field in `[frontmatter]` in `ascribe.toml`, such as `tags = "list(string)?"`, or nothing: a key `ascribe.toml` doesn't declare is reported |

The callout types are `note`, `tip`, `important`, `warning`, and `caution`; `[notes]` in `ascribe.toml` declares others. Only `.md` files are pages: rename a `.mdx` file whose components you've replaced, and keep a file that isn't a page out of the content root, or in a folder whose name starts with `.`.

## See also

- [`ascribe init`](install.md#cli-init): what it writes, and its options.
- [`ascribe check`](checks/check.md): the check, its formats, and `--fix`.
- [Frontmatter](writing/frontmatter.md): the keys every page can have.
- [The `ascribe.toml` reference](content-model/index.md): types, and the fields they share.
