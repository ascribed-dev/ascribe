---
type: how-to
title: Bring existing docs to Ascribe
description: "Start a project in a folder of Markdown you already have, and take it from `ascribe init` to a clean `ascribe check`."
permalink: existing-docs
---

Ascribe reads plain CommonMark, so a folder of Markdown is most of a project already. This guide starts a project around one, fixes what the first check finds, and shows what to rewrite in pages written for another tool: links, frontmatter, callouts, and heading ids.

The pages here are [`examples/existing-docs`]({repo}/tree/main/examples/existing-docs): four pages in `docs/`, some with frontmatter and some without, and one link written as the address the old site published.

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
   npx ascribe check --fix
   ```

   A page with no `title` that opens with a level-1 heading takes the heading as its title: the fix writes it in the frontmatter, and takes the heading out of the page, since the site shows the title in its place. A link written as a page's published address, such as `/guides/alerts/`, is rewritten to the page's file. What's left is listed:

   @snippet {lang=text}: code:crates/ascribe-cli/tests/output/init-existing-fix.txt

   A fix is safe only when it can't change what the page says. A page whose opening heading something links to, as `#configure-alerts`, isn't given the heading as its title by `--fix`, since the link would lose its target; the fix is offered in the editor, for you to apply and fix the link.
2. Give the other pages a title. `troubleshooting.md` opens with a paragraph, so it gets frontmatter:

   ```markdown
   ---
   title: When a page doesn't load
   ---
   ```

3. Fix the links. A link in Ascribe names the file it goes to, and each output writes its address, so `[Install Lantern](../install.html)` becomes `[Install Lantern](../install.md)`. `ascribe check --code link-target-missing` lists every link that names no file.
4. Put the advice off. Advice never fails the check: here it's the pages with no `description`. To deal with it later, and have the check report only what's new, record it in `ascribe.baseline`:

   ```sh
   npx ascribe baseline write
   ```

   @snippet {lang=text}: code:crates/ascribe-cli/tests/output/init-existing-baseline.txt

   See [The baseline](checks/baseline.md) for what it holds back, and how it shrinks as you fix each.
5. Check again:

   ```sh
   npx ascribe check
   ```

   @snippet {lang=text}: code:crates/ascribe-cli/tests/output/init-existing-check.txt

Commit `ascribe.toml`, `ascribe.baseline`, and the pages, and add `.ascribe/` to `.gitignore`. To check every pull request, see [Check in CI](checks/ci.md).

## What to rewrite in pages from another tool

The check finds the links and the frontmatter that need a change. It doesn't find callouts or heading ids written in another tool's syntax: they're text to Ascribe, so the site shows them as they're written. Search for them:

```sh
grep -rnE '^(:::|!!! |> \[!)|\{#[^}]*\}\s*$|\[\[' docs/
```

| Written as | In Ascribe |
|---|---|
| `[Install](install.html)`, `[Install](/install/)` | `[Install](install.md)`: the file. A published address is reported, and `--fix` rewrites it; an `.html` link is reported as missing, for you to rewrite. |
| `[[Install]]`, `[[install\|Install it]]` | `[Install it](install.md)` |
| `:::note` … `:::`, `!!! note`, `> [!NOTE]` | [`@note`](writing/directives.md#note), as `@note: One line.`, or `@note {type=warning}:` … `@end` around several blocks |
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
