---
type: page
title: Ascribe
description: "Ascribe is a toolchain for documentation written as code: Markdown with the structure docs need, checked as you write, and built into a website, plain Markdown, and JSON."
intended:
  - check: title-heading-differs
    reason: The home page's title is the site's name, and its top heading is the headline in the hero.
---

@hero-banner {status=early}:
# Docs as code, with the tools docs need.

Ascribe is a toolchain for documentation written as code: Markdown with the structure docs need, checked as you write, and built into a website, plain Markdown, and JSON.

@tool-list:
- [Reusable content](#tools)
- [Variants](#tools)
- [Versions](#tools)
- [A glossary](#tools)
- [Preview](#tools)
@end

@call-to-action:
@memo: Start installs the same command, and scripts/docs-site/facts.test.ts fails when either differs from getting started's.
```sh
npm install --save-dev @ascribed/cli
```

[Read getting started]({site}/go/getting-started/)
@end

![The mark: a heavy letter a, with the two change marks of a gutter beside it, on a grid.](images/hero-mark-light.svg){scheme=light}
![The mark: a heavy letter a, with the two change marks of a gutter beside it, on a grid.](images/hero-mark-dark.svg){scheme=dark}
@end

## A page, and what readers see

Here is a page's source, read from its file in Ascribe's repository, and the same file as this site builds and draws it. The note, the tabs, and the steps are Ascribe's.

@source-and-result {page=install}:
@snippet {lang=markdown, title=install.md}: code:site/demo/content/install.md
@end

## The tools of a docs team
@id: tools
@story-pillar

@pillar-proof:
- [Reuse]({site}/go/directives/) with `@include` and phrases.
- [Variants]({site}/go/variants/) by platform, product, or package manager with `@variant`, shown as tabs.
- [Availability and versions]({site}/go/versions/): one source, a build for each release line.
- A [glossary]({site}/go/glossary/) that links terms to their definitions.
- [Page types and templates]({site}/go/content-model/), from a content model in `ascribe.toml`.
- [Notes, steps, and details]({site}/go/directives/).
- A [page preview and a site preview]({site}/go/preview/) in VS Code.
- [Permalinks]({site}/go/permalinks/) that keep working when a section moves.
@end

@proof-example:
@variant {pm=npm}:
```sh
npx ascribe build
```
@variant {pm=pnpm}:
```sh
pnpm exec ascribe build
```
@end

Content that differs by platform, product, or package manager is marked once, with `@variant`, as this page marks the command above. The site draws it as tabs, and plain Markdown and JSON label each arm.
@end

## Checked like code
@id: checks
@story-pillar

@pillar-proof:
### Checked

A broken link, a missing heading, or a name the content model doesn't declare is an error, in the editor as you type and in CI. This is what [`ascribe check`]({site}/go/cli-check/) prints for the page that getting started writes, which links to a page it doesn't have yet.

[Vale]({site}/go/cli-vale/) style checks run on the prose.
@end

@proof-example:
@snippet {lang=text}: code:crates/ascribe-cli/tests/output/getting-started-check.txt
@end

@pillar-proof:
### Examples from real code

A code example is read from the code's own file with `@snippet`, every time a page is checked or built, so it changes when the code does. On a pull request, [`ascribe drift`]({site}/go/cli-drift/) lists the pages whose examples changed, so someone rereads the words around them.
@end

@proof-example:
@snippet {lang=text}: code:crates/ascribe-cli/tests/output/drift.txt
@end

@pillar-proof:
### Reviewed as pages

A pull request's changes are read the way readers will see them: in the site's own preview, or in VS Code's, with what changed marked and each review comment beside the paragraph it's about. [Review]({site}/go/review-guide/) shows how.
@end

@proof-example:
![A page of these docs in the site's preview during review: a changed paragraph is marked, the pull request's two comments on it are beside it, and the review panel below names the pull request and its one change.](screenshots/review-light.png){scheme=light}
![A page of these docs in the site's preview during review: a changed paragraph is marked, the pull request's two comments on it are beside it, and the review panel below names the pull request and its one change.](screenshots/review-dark.png){scheme=dark}
@end

## Built for AI agents, writing and reading
@id: for-agents
@story-pillar

@pillar-proof:
### As writers

- The project's rules from [`ascribe agents sync`]({site}/go/cli-agents-sync/).
- The model and the checks through the [MCP server]({site}/go/cli-mcp/).
- A [hook]({site}/go/cli-agents-hook/) that checks after each edit.
- [Memos]({site}/go/memos/) keep the reason a page is written its way, for the next person or agent who changes it.

### As readers

- [`llms.txt`]({site}/go/publish-for-agents/).
- A [Markdown version]({site}/go/publish-for-agents/) of every page.
- Each permalink's [help as JSON]({site}/go/astro-reference/).
- An [index of the docs]({site}/go/cli-agents-index/) written into the product's `AGENTS.md`.
@end

@proof-example:
@snippet {lang=markdown, title=AGENTS.md}: code:crates/ascribe-cli/tests/output/agents/quill/AGENTS.md.snap

What [`ascribe agents sync`]({site}/go/cli-agents-sync/) writes into an example project's `AGENTS.md`.
@end

@audience-list:
## For engineers, writers, and agents

The pillars are the same for everyone. What leads changes with the reader.

@audience-entry:
### Engineers

Leads with [checked like code](#checks) and examples from real code.

They get writing tools without leaving the editor, the repository, or CI.

[The path for engineers]({site}/go/for-engineers/)
@end

@audience-entry:
### Writers

Leads with [the tools of a docs team](#tools).

They get reuse, variants, versions, a glossary, and preview, without leaving the repository where the engineers who review them work.

[The path for writers]({site}/go/for-writers/)
@end

@audience-entry:
### Agents

Leads with [the rules, the checks, and the outputs](#for-agents).

An agent writing pages has a deterministic check to run. An agent reading them gets Markdown and an index, not a page of HTML.

[The path for agent builders]({site}/go/for-agent-builders/)
@end
@end

## Start
@memo: Getting started's first two sections install these two (decision 86 in project-docs/decisions.md), and scripts/docs-site/facts.test.ts fails when the commands here, or the hero's, differ from theirs.

@call-to-action:
@steps
1. Install the command in your project's directory:

   ```sh
   npm install --save-dev @ascribed/cli
   ```

2. And the editor, for VS Code:

   ```sh
   code --install-extension Ascribe.ascribe-vscode
   ```

Then [read getting started]({site}/go/getting-started/): it sets up a project, writes a first page, checks it, and publishes it.

[Read getting started]({site}/go/getting-started/)
@end

@quiet-pillar:
## Plain files you can leave

- A page reads as Markdown without Ascribe.
- The [plain output]({site}/go/cli-build/) is CommonMark.
- Ascribe hosts nothing and calls no model.
- It's open source under the MPL.

[Why Ascribe](why.md) says which problems these answer, from what teams that keep docs as code report, and what Ascribe doesn't do.

## Early, and open source

Ascribe is before 1.0, so what it writes and what it accepts can still change between releases. The latest release is {version}. The code, the docs, and the plans are on [GitHub](https://github.com/ascribed-dev/ascribe), under the Mozilla Public License 2.0.

Ascribe is made by [Bryo Studio](https://bryostudio.com). [Credits](credits.md) says what it's built on.
@end
