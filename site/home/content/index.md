---
title: Ascribe
description: Documentation written as code, checked like code, and built into a website, plain Markdown, and JSON.
---

@hero-banner {status=early}:
# Documentation written as code

Ascribe is Markdown with a small set of directives for the structure documentation needs, a content model that says what a page may use, and one compiler that checks the pages and builds them into a website, plain Markdown, and JSON.
@end

## Early, and open source

Ascribe is before 1.0, so what it writes and what it accepts can still change between releases. The latest release is {version}. The code, the docs, and the plans are on [GitHub](https://github.com/ascribed-dev/ascribe), under the Mozilla Public License 2.0.

## Start

Install the command in your project's directory:

```sh
npm install --save-dev @ascribed/cli
```

And the editor, for VS Code:

```sh
code --install-extension Ascribe.ascribe-vscode
```

Then [read getting started]({site}/go/getting-started/): it sets up a project, writes a first page, checks it, and publishes it.
