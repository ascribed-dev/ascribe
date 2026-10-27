---
type: reference
title: Files Ascribe writes
description: "Every file Ascribe puts in a project, what writes it, and whether to commit it or leave it to `.gitignore`."
permalink: project-files
---

Ascribe reads your pages and `ascribe.toml`, and writes a few files of its own beside them. This page lists every one, so you can tell a file you didn't write from one you did, and decide what to commit.

The short version: add `.ascribe/` to your `.gitignore`, and commit the rest of what Ascribe writes, apart from its copy inside Astro's own `.astro/` folder.

```text
.ascribe/
```

## The files

Paths are from the folder that holds `ascribe.toml`, except where a row says the repository's root or the Astro root. In a repository with one project at its root, these are the same folder.

| File | What writes it | Commit it |
|---|---|---|
| `.ascribe/build/` | [`ascribe build`](../checks/check.md#cli-build), and `@ascribed/astro` when it builds. It's the default output directory; `[project] output-dir` puts it elsewhere ([output layout](../contracts/output-layout.md)). | No |
| `.ascribe/vale/` | `ascribe check` and the editor, when they run Vale: a preset's configuration, and the vocabulary of the project's own words ([Prose, through Vale](../checks/vale.md#the-projects-own-words)). | No |
| `.ascribe/cache/` | [`ascribe report links`](../checks/report-command.md#cli-report), which keeps the answers it reuses in `.ascribe/cache/links/` ([Answers it reuses](../checks/external-links.md#answers-it-reuses)). | No |
| `.ascribe/history/` | `ascribe report links`, which keeps in `.ascribe/history/links.json` the last day each address answered, the day it started failing, and whether it was reported ([Links that answered before](../checks/external-links.md#links-that-answered-before)), and [`ascribe report`](../checks/report-command.md#cli-report), which keeps in `.ascribe/history/report.json` what its `problems` and `delivery` sections found, to say what's new next time ([What's new](../checks/report.md#whats-new-since-the-last-report)). | No |
| `.ascribe/dev.json` | `astro dev` with `@ascribed/astro`, for the editor's site preview. It's removed when `astro dev` stops ([In `astro dev`](../astro/reference.md#in-astro-dev)). | No |
| `.ascribe/memos/` | You do: [personal memos](../writing/memo-files.md#personal-memos), memo files kept on your machine only. Ascribe reads them, and writes one only when you ask: the editor's **Ascribe: Add a personal memo** creates one, and a rename (`ascribe rename`, `F2`, or moving a file in the editor) rewrites the `about` of each that names what it renames. | No |
| `ascribe.lock` | [`ascribe sources fetch`](../examples/sources.md#cli-sources-fetch) and [`ascribe sources update`](../examples/sources.md#cli-sources-update): the commit each source in another repository is pinned to ([Keep the docs in another repository](../examples/docs-repository.md)). | Yes |
| `sources/` | `ascribe sources fetch` and `ascribe sources update`: copies of the files your snippets use from another repository, in `sources/<name>/`. | Yes |
| `examples.observations/` | [`ascribe observe`](../examples/sources.md#cli-observe), beside `ascribe.examples.toml` in the code's folder: what its tested examples passed with, in `default.toml`, or `<name>.toml` with `--name`. `ascribe observe --check` fails without it ([Tested examples](../examples/tested.md#2-run-the-test-and-commit-what-it-records)). | Yes |
| `ascribe.permalinks` | [`ascribe permalinks sync`](../permalinks/command.md#cli-permalinks-sync) and [`ascribe permalinks retire`](../permalinks/command.md#cli-permalinks-retire): the list of the docs' permalinks a product checks its links against ([The permalinks list](../contracts/permalinks-list.md)). | Yes |
| `ascribe.addresses` | [`ascribe init`](../install.md#cli-init) writes it with a new project, or [`ascribe addresses init`](../publishing/redirects.md#cli-addresses-init) makes it, and from then on [`ascribe addresses move`](../publishing/redirects.md#cli-addresses-move) and [`ascribe addresses retire`](../publishing/redirects.md#cli-addresses-retire), `ascribe rename`, moving a page in the editor, and `ascribe check --fix` write to it: where each address the docs published went, which the build turns into redirects ([Keep the addresses you've published](../publishing/redirects.md)). | Yes |
| `ascribe.baseline` | [`ascribe baseline write`](../checks/baseline.md#cli-baseline-write) and [`ascribe baseline prune`](../checks/baseline.md#cli-baseline-prune): the advice `ascribe check` holds back for now ([The baseline](../checks/baseline.md)). | Yes |
| `ascribe.toml` | You do: the content model ([`ascribe.toml` reference](../content-model/index.md)). [`ascribe init`](../install.md#cli-init) writes the first one, and a few commands change it in place ([Files of yours that Ascribe changes](#files-of-yours-that-ascribe-changes)). | Yes |
| `ascribe.examples.toml` | You do, in the code's folder: the files the docs show as examples, and the test that proves them ([Tested examples](../examples/tested.md#1-say-which-files-are-examples)). `ascribe observe` reads it. | Yes |
| `ascribe.nav` | You do: the order of the project's sections and pages, and their labels, beside `ascribe.toml` ([Navigation](../navigation/index.md)). Ascribe reads it; with none, a project has the default order. | Yes |
| `memos/` | You do: [memo files](../writing/memo-files.md), which say why a page is written the way it is. Ascribe reads them, and writes one only when you ask: the editor's **Ascribe: Add a memo** creates one, and a rename (`ascribe rename`, `F2`, or moving a file in the editor) rewrites the `about` of each that names what it renames. They're in no output. | Yes |
| `.vale.ini` and `.vale/` | [`ascribe vale eject`](../checks/vale.md#cli-vale-eject), once. From then on they're yours, and Ascribe doesn't write them again. | Yes |
| `templates/` | [`ascribe templates eject`](../templates/command.md#cli-templates-eject), once for each template: the template, its guide, and Ascribe's notes on the type, in `templates/` or the folder `--to` names, and `templates.lock` beside them, which records what each was copied from. From then on the templates are yours, and Ascribe doesn't write them again ([Page templates](../templates/index.md)). | Yes |
| `AGENTS.md` | [`ascribe agents sync`](../agents/commands.md#cli-agents-sync), between its markers, beside `ascribe.toml`, and at the repository's root when that's another folder. The rest of the file is yours ([Agent instructions](../agents/instructions.md)). | Yes |
| `CLAUDE.md` | `ascribe agents sync`, an import of `AGENTS.md` between its markers, in each `CLAUDE.md` there is; it creates one only with `--target claude`. | Yes |
| `.agents/skills/ascribe/` and `.claude/skills/ascribe/` | `ascribe agents sync`, at the repository's root: the skill, the second with `--target claude` ([The skill](../agents/instructions.md#the-skill)). | Yes |
| `.claude/rules/ascribe*.md` and `.github/instructions/ascribe*.instructions.md` | `ascribe agents sync`, at the repository's root, with `--target claude-rules` and `--target copilot` ([Other agents' files](../agents/instructions.md#other-agents-files)). | Yes |
| `.claude/settings.json`, `.mcp.json`, `.codex/hooks.json`, and `.github/hooks/ascribe.json` | `ascribe agents sync --with-hook`, at the repository's root: the hooks' entries and the MCP server, beside your own ([Hooks](../agents/hooks.md)). | Yes |
| `.github/workflows/copilot-setup-steps.yml` and `.github/agents/ascribe-docs.md` | `ascribe agents sync --cloud`, at the repository's root: steps between markers in the workflow, and with `--agent`, the custom agent ([GitHub Copilot](../agents/copilot.md)). | Yes |
| The files you name with `--into` and `--pages` | [`ascribe agents index`](../agents/commands.md#cli-agents-index): the index of the docs between its markers in the `--into` file, such as your product's `AGENTS.md`, and the pages' Markdown versions in the `--pages` folder ([Index your docs for your product's agents](../agents/product-index.md)). | Yes |
| `src/content.config.ts`, `src/pages/[...slug].astro`, `src/layouts/Docs.astro`, and `src/styles/docs.css` | `npx @ascribed/astro init`, once, in the Astro root. From then on they're yours ([Set up a site](../astro/index.md#set-up-a-site)). | Yes |
| `.astro/integrations/_ascribed_astro/` | `@ascribed/astro`, in the Astro root: a copy of the collection's schema, inside Astro's own folder, which Astro's starters leave out of git. | No |
| `dist/_redirects`, `dist/_headers`, and `dist/_ascribe/gone/` | `@ascribed/astro`, when it builds, in the Astro root's output folder: with `host` in `[consumer]`, the host's files, whose `_redirects` has a line for each address [`ascribe.addresses`](../publishing/redirects.md#what-the-build-writes) records a move or a retirement of ([the host's files](../publishing/site.md#the-hosts-files)), and in `_ascribe/gone/` under the base path, a page for each retired address, saying what was removed and why ([A retired address's page](../publishing/redirects.md#a-retired-addresss-page)). Astro's starters leave `dist/` out of git. | No |

## What deleting them costs

Everything under `.ascribe/` is written again when it's needed: deleting it costs the next build, check, or report time, and nothing else, apart from `.ascribe/history/` and `.ascribe/memos/`. Without `.ascribe/history/`, the next `ascribe report links` reports every failing link, as a first run does, including one that would have been held as unconfirmed: one noisier report, and never a missed problem. Your personal memos are yours alone, so deleting `.ascribe/memos/` loses them.

A file you commit is a record, not a cache. Without `ascribe.lock` and `sources/`, `ascribe check` reports each snippet from another repository until `ascribe sources fetch` writes both again. Without `ascribe.permalinks`, a product's `ascribe permalinks verify` has no list to check against, and `ascribe check` in the docs can't warn when one is removed. Without `ascribe.addresses`, a move is no longer recorded, the build writes no redirects for the pages that moved before, and `ascribe check` can't report an address that's lost.

## Files of yours that Ascribe changes

A few commands change files you wrote, in place, and only when you run them: `ascribe vale eject` changes `ascribe.toml`, keeping its comments, and `ascribe addresses init` takes its `[permalinks]` out, and `ascribe templates eject` adds to its end; [`ascribe fmt`](../checks/check.md#cli-fmt), [`ascribe rename`](questions.md#cli-rename), and `ascribe check --fix` change your pages, and `ascribe rename` changes `ascribe.toml` and `ascribe.nav` too, keeping their comments; the editor's **Pages** view changes `ascribe.nav` as you drag in it; and the editor's quick fixes change the file the problem is in.

Outside the project, Ascribe keeps the repositories `ascribe sources` fetches, and the agents' hooks keep a check server's files, in a folder of your user's cache folder. Nothing there needs committing or keeping.
