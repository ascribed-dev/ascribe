---
type: how-to
title: Bring existing docs to Ascribe
description: "Start a project in a folder of Markdown you already have, and take it from `ascribe init` to a clean `ascribe check`."
permalink: existing-docs
---

Ascribe reads plain CommonMark, so a folder of Markdown is most of a project already. This guide starts a project around one, fixes what the first check finds, and shows what to rewrite in pages written for another tool: links, frontmatter, callouts, heading ids, components, and a copy of the docs for each version.

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

In an empty folder, `init` declares the type `page` instead, which each page names; [`ascribe init`](command/init.md#cli-init) has both. `--templates`, which declares six types, is refused in a folder of pages, since each page would have to name one. Take the templates once the pages check clean, with [`ascribe templates eject`](templates/command.md#cli-templates-eject), then give each page `type:`, since the project declares types from then on.

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

The callout types are `note`, `tip`, `important`, `warning`, and `caution`; `[notes]` in `ascribe.toml` declares others. Only `.md` files are pages: rename a `.mdx` file once its components are replaced, as [the next section](#components-and-versions-from-another-tool) maps them, and keep a file that isn't a page out of the content root, or in a folder whose name starts with `.`.

## Components and versions from another tool
@id: components-and-versions-from-another-tool

The check reports none of these either: an MDX component or a shortcode is text or raw HTML to Ascribe. Each one becomes a directive, a phrase, or a widget the project declares. [`examples/comparison`]({repo}/tree/main/examples/comparison) has one page written four ways, in Ascribe, Starlight MDX, Elastic's docs-builder, and Hugo, with a table that compares them construct by construct.

### Tabs, steps, partials, and components

| Docusaurus or Starlight (MDX) | MkDocs | Hugo | In Ascribe |
|---|---|---|---|
| `<Tabs>` with a `<TabItem>` for each tab; Starlight's `<Tabs syncKey>`, `<PackageManagerTabs>` | `=== "npm"` content tabs | `{{< tabs >}}` with a `{{< tab >}}` for each | A [`@variant`](writing/directives.md#variant) group: an arm for each tab, `@variant {pm=npm}:`, and one `@end`. Tabs that are always the same choice, such as the package manager, are a dimension in `ascribe.toml`, so a build can keep one arm; a one-off set of tabs is labeled arms, a title line such as `.Using Docker` above each `@variant:` |
| `<Steps>` around an ordered list | A plain ordered list | A plain ordered list | [`@steps`](writing/directives.md#steps) on the line above the list, with no wrapper to close |
| `import Partial from './_partial.mdx'` and `<Partial />` | `--8<-- "partial.md"` | `{{% include "partial.md" %}}` | [`@include: _partial.md`](writing/directives.md#include). A file whose name starts with `_` is a fragment, not a page, so Docusaurus's partials keep their names; rename the `.mdx` to `.md` |
| `:::tip[Title]`, `:::tip Title`, or Starlight's `<Aside type="tip" title="Title">` | `!!! tip "Title"` | `> [!TIP]`, which the check reports, and `--fix` rewrites | A title line, then the note: `.Title` on the line above `@note {type=tip}`. Without a title, `@note {type=tip}: …` alone |
| `<details>` with a `<summary>` | `??? note "Title"` | `{{< details >}}` | [`@details`](writing/directives.md#details), with the summary as its title line |
| A value written into the page, such as a version | A `{{ variable }}` from `mkdocs-macros` | `{{% param "version" %}}` | A [phrase](glossary.md#phrase), `{version}`, declared once in `[phrases]`. In a code block, the fence opts in with `phrases=true` |
| `<Badge>`, or a label saying where a section applies | | | [`@available`](writing/directives.md#available) under the heading, such as `@available: self-managed 3.4`, naming a target `ascribe.toml` declares |
| Any other component the project wrote, such as `<Video>` | A Markdown extension or a macro | A shortcode of the project's own | A [project widget](writing/directives.md#project-widgets): a directive declared in `[widgets.<name>]`, such as `@quill-video {id=intro}`. The site writes it as a custom element with its name, for the site to define, and plain Markdown writes its fallback text |

Markdown that isn't a directive stays as it's written: an ordered list stays an ordered list. A heading's id is `@id` on the line below it, as the table above rewrites it.

### A copy of the docs for each version

Docusaurus keeps a copy of every page for each version in `versioned_docs/`, and mike publishes a copy of a whole MkDocs site for each. Ascribe keeps one copy, and builds a site for each version from it:

@steps
1. Start from the newest copy, the one in `docs/`, and leave the others out of the content root.
2. Where an older version's page differs, mark what's in some versions and not others with [`@available`](writing/directives.md#available): `@available: self-managed 3.5` under a heading that's new in 3.5, or `available:` in a page's frontmatter for a page that's new.
3. List the versions a reader can be on in [`[releases]`](content-model/variation.md#121-releasestarget), as `live = ["3.4", "3.5"]`, and name it on the site's build, as `versions = "self-managed"`. Each live line gets its own output: the current line keeps the site's addresses, so turning versions on moves none, and each older one is under its number, as `/docs/3.4/`.
4. For a version too old to rebuild, keep the site the old tool published, and list it under `archived` in `[releases]` with its address, so the live site lists it among the versions.

[Publish a site for each version](versions/index.md) has each step, from listing the lines to moving them on when a version ships.

## See also

- [`ascribe init`](command/init.md#cli-init): what it writes, and its options.
- [`ascribe check`](checks/check.md): the check, its formats, and `--fix`.
- [Frontmatter](writing/frontmatter.md): the keys every page can have.
- [The `ascribe.toml` reference](content-model/index.md): types, and the fields they share.
