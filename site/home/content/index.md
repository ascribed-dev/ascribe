---
type: page
title: Ascribe
description: Documentation written as code, checked like code, and built into a website, plain Markdown, and JSON.
intended:
  - check: title-heading-differs
    reason: The home page's title is the site's name, and its top heading is the tagline in the hero.
---

@hero-banner {status=early}:
# Documentation written as code

Ascribe is Markdown with a small set of directives for the structure documentation needs, a content model that says what a page may use, and one compiler that checks the pages and builds them into a website, plain Markdown, and JSON.
@end

## A page, and what readers see

Here is a page's source, read from its file in Ascribe's repository, and the same file as this site builds and draws it. The note, the tabs, and the steps are Ascribe's.

@source-and-result {page=install}:
@snippet {lang=markdown, title=install.md}: code:site/demo/content/install.md
@end

## Treated like code

@claim-example:
### Checked

A broken link, a missing heading, or a name the content model doesn't declare is an error, in the editor as you type and in CI. This is what [`ascribe check`]({site}/go/cli-check/) prints for the page that getting started writes, which links to a page it doesn't have yet:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/getting-started-check.txt

An AI coding agent runs the same check after each edit: [`ascribe agents sync`]({site}/go/agents-guide/) gives it the project's rules.
@end

@claim-example:
### One source, several outputs

Content that differs by platform, product, or package manager is marked once, with [`@variant`]({site}/go/directives/). The site draws it as tabs, and plain Markdown and JSON label each arm. A build can keep only one, and a page can say which releases have what it describes.

@variant {pm=npm}:
```sh
npx ascribe build
```
@variant {pm=pnpm}:
```sh
pnpm exec ascribe build
```
@end
@end

@claim-example:
### Examples from real code

A code example is read from the code's own file with `@snippet`, every time a page is checked or built, so it changes when the code does. On a pull request, [`ascribe drift`]({site}/go/cli-drift/) lists the pages whose examples changed, so someone rereads the words around them:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/drift.txt
@end

@claim-example:
### Reviewed as pages

A pull request's changes are read the way readers will see them: in the site's own preview, or in VS Code's, with what changed marked and each review comment beside the paragraph it's about. [Review]({site}/go/review-guide/) shows how.

![A page of these docs in the site's preview during review: a changed paragraph is marked, the pull request's two comments on it are beside it, and the review panel below names the pull request and its one change.](screenshots/review-light.png){scheme=light}
![A page of these docs in the site's preview during review: a changed paragraph is marked, the pull request's two comments on it are beside it, and the review panel below names the pull request and its one change.](screenshots/review-dark.png){scheme=dark}
@end

[Why Ascribe](why.md) says which problems these answer, from what teams that keep docs as code report, and what Ascribe doesn't do.

## Start
@memo: Getting started's first two sections install these two (decision 86 in project-docs/decisions.md), and scripts/docs-site/facts.test.ts fails when the commands here differ from theirs.

Install the command in your project's directory:

```sh
npm install --save-dev @ascribed/cli
```

And the editor, for VS Code:

```sh
code --install-extension Ascribe.ascribe-vscode
```

Then [read getting started]({site}/go/getting-started/): it sets up a project, writes a first page, checks it, and publishes it.

## Where it's going

- **Search.** Find where the pages say something as readers see it, through phrases, fragments, and variants, and edit it at its source.

## Early, and open source

Ascribe is before 1.0, so what it writes and what it accepts can still change between releases. The latest release is {version}. The code, the docs, and the plans are on [GitHub](https://github.com/ascribed-dev/ascribe), under the Mozilla Public License 2.0. [Credits](credits.md) says who builds it, and what it's built on.
