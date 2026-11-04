---
type: reference
title: "`ascribe init`"
description: "Start a project with its first `ascribe.toml` and `ascribe.addresses`: options, output, and exit codes."
---

`ascribe init` starts a project: it writes its first `ascribe.toml` and `ascribe.addresses`, and makes the folder its pages go in. [Install Ascribe](../install.md#start-a-project) is the task it's used in.

@include: ../_fragments/command-options.md

## `ascribe init`
@id {permalink=true}: cli-init

@summary
Starts a project: writes its first `ascribe.toml` and `ascribe.addresses`, and makes the folder its pages go in, `docs/`. With `--templates`, it takes the page templates too, as [`ascribe templates eject`](../templates/command.md#cli-templates-eject) with no name does. In a folder of pages already there, it declares no page types, checks the pages, and says what to run next.

@include: ../_generated/cli-init-options.md

It writes in the current folder, or in `DIR`, a path from the current folder, which it makes if it isn't there. It asks nothing. Where there's an `ascribe.toml` in the folder or above it, it refuses, names the one it found, and writes nothing: `ascribe templates eject`, run there, adds the templates to that project. To start a second project in a repository that has one, name a folder outside it. It takes no `--config`, since the project is the one it starts.

The file it writes is small, and says what each part is for:

@snippet: code:crates/ascribe-cli/tests/output/init-ascribe.toml

It sets the content root, `docs`, though that's the default, since it's the first thing a project changes; `--content-root` names another, a folder inside the project's. It declares one type, `page`, with a title and a description; since it declares a type, each page names it in its frontmatter, with `type: page`. Its comments name what most projects add next, each with its section of the [`ascribe.toml` reference](../content-model/index.md), and it adds none of them: no site address, no builds, and no dimensions.

Beside `ascribe.toml`, it writes `ascribe.addresses`, with no entries yet, so the project [keeps the addresses it publishes](../publishing/redirects.md) from its first commit: once a page is published, moving it with `git mv` is [`address-lost`](../checks/diagnostics/project-files.md#ASC255) on the next check against the base, and its fix records the move. It does so in a folder that has pages already, too, and leaves an `ascribe.addresses` that's there as it is.

With `--templates`, the six types are added after `page`, and the templates, their guides, Ascribe's notes, and `templates.lock` are written into `templates/`, as `eject` writes them. When a file is where one would be written, it's named, as `eject` names it, and nothing is written.

Like every [command that writes](index.md#commands-that-write), `init` won't change a file git couldn't put back: it names the file and writes nothing, unless you pass `--force`. It adds files, and changes one that's there only to add the templates to it, so it has little for git to refuse. With `--dry-run`, it says what it would write and make, and writes nothing.

It lists, from the current folder:

- Each file it wrote, `ascribe.toml` and then `ascribe.addresses` last.
- The content root, when it made it.
- With `--templates`, each type it added, as `eject` says it.
- The next step: a first page in the content root, and [`ascribe check`](../checks/check.md#cli-check).
- Last, the [Astro guide](../astro/index.md), to publish the pages as a site.

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/init.txt

**In a folder of pages.** When the content root holds pages already, `ascribe.toml` declares no page types, so the pages are of the implicit type `page` and none of them needs `type:`. In place of the type, it declares `description` as a field every page can have. `--templates`, whose types each page would have to name, is refused.

Then it checks the pages and says what it found: how many pages, how many have no `title` and how `ascribe check --fix` gives them one, and what [`ascribe check`](../checks/check.md#cli-check) finds, by code, most first. It ends with what to run next: `--fix`, then the most common code that's left, and [`ascribe baseline write`](../checks/baseline.md) when there's advice to record in a baseline. It exits with 0 whatever the check finds, since the project is there. [Bring existing docs to Ascribe](../existing-docs.md) goes through it, step by step:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/init-existing.txt

With `--format json`, it writes the document its [schema](../contracts/json-reports/model.md#ascribe-init) describes. When `templates.refused` isn't empty, `written` is empty.

.The JSON for this example
@details:
@snippet {lang=json}: code:crates/ascribe-cli/tests/output/init.json
@end

| Code | Meaning |
|---|---|
| `0` | The project is there: `ascribe.toml` and `ascribe.addresses` are written, the content root is there, and with `--templates`, every template and its type |
| `2` | It wrote nothing: there's an `ascribe.toml` in the folder or above it, the folder is a file, `--content-root` isn't a folder inside the project's, `--templates` was given in a folder of pages, a template was refused, it was given `--config`, or, without `--force`, a file git couldn't put back. Or a file couldn't be written; the files it wrote before that stay. |

## See also

- [Getting started](../getting-started.md): a first project, from `npm install` to a built page.
- [Bring existing docs to Ascribe](../existing-docs.md): a project around a folder of Markdown you have.
