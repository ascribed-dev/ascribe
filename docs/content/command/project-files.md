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
| `.ascribe/vale/` {available="ascribe 0.3.0"} | `ascribe check` and the editor, when they run Vale: a preset's configuration, and the vocabulary of the project's own words ([Prose, through Vale](../checks/vale.md#the-projects-own-words)). | No |
| `.ascribe/cache/` {available="ascribe 0.3.0"} | [`ascribe report links`](../review/diff.md#cli-report), which keeps the answers it reuses in `.ascribe/cache/links/` ([Answers it reuses](../checks/report.md#answers-it-reuses)). | No |
| `.ascribe/history/` {available="ascribe 0.3.0"} | `ascribe report links`, which keeps in `.ascribe/history/links.json` the last day each address answered, the day it started failing, and whether it was reported ([Links that answered before](../checks/report.md#links-that-answered-before)). | No |
| `.ascribe/dev.json` | `astro dev` with `@ascribed/astro`, for the editor's site preview. It's removed when `astro dev` stops ([In `astro dev`](../astro/reference.md#in-astro-dev)). | No |
| `.ascribe/memos/` {available=next} | You do: [personal memos](../writing/memo-files.md#personal-memos), memo files kept on your machine only. Ascribe reads them, and writes one only when you ask: the editor's **Ascribe: Add a personal memo** creates one, and a rename (`ascribe rename`, `F2`, or moving a file in the editor) rewrites the `about` of each that names what it renames. | No |
| `ascribe.lock` | [`ascribe sources fetch`](../examples/sources.md#cli-sources-fetch) and [`ascribe sources update`](../examples/sources.md#cli-sources-update): the commit each source in another repository is pinned to ([Drift](../examples/index.md)). | Yes |
| `sources/` | `ascribe sources fetch` and `ascribe sources update`: copies of the files your snippets use from another repository, in `sources/<name>/`. | Yes |
| `ascribe.permalinks` {available="ascribe 0.3.0"} | [`ascribe permalinks sync`](../examples/sources.md#cli-permalinks-sync) and [`ascribe permalinks retire`](../examples/sources.md#cli-permalinks-retire): the list of the docs' permalinks a product checks its links against ([The permalinks list](../contracts/permalinks-list.md)). | Yes |
| `ascribe.nav` {available=next} | You do: the order of the project's sections and pages, and their labels, beside `ascribe.toml` ([Navigation](../navigation/index.md)). Ascribe reads it; with none, a project has the default order. | Yes |
| `memos/` {available=next} | You do: [memo files](../writing/memo-files.md), which say why a page is written the way it is. Ascribe reads them, and writes one only when you ask: the editor's **Ascribe: Add a memo** creates one, and a rename (`ascribe rename`, `F2`, or moving a file in the editor) rewrites the `about` of each that names what it renames. They're in no output. | Yes |
| `.vale.ini` and `.vale/` {available="ascribe 0.3.0"} | [`ascribe vale eject`](../examples/sources.md#cli-vale-eject), once. From then on they're yours, and Ascribe doesn't write them again. | Yes |
| `templates/` {available="ascribe 0.3.1"} | [`ascribe templates eject`](../examples/sources.md#cli-templates-eject), once for each template: the template, its guide, and Ascribe's notes on the type, in `templates/` or the folder `--to` names, and `templates.lock` beside them, which records what each was copied from. From then on the templates are yours, and Ascribe doesn't write them again ([Page templates](../templates/index.md)). | Yes |
| `AGENTS.md` {available="ascribe 0.3.0"} | [`ascribe agents sync`](../agents/commands.md#cli-agents-sync), between its markers, beside `ascribe.toml`, and at the repository's root when that's another folder. The rest of the file is yours ([Agent instructions](../agents/index.md#agent-instructions)). | Yes |
| `CLAUDE.md` {available="ascribe 0.3.0"} | `ascribe agents sync`, an import of `AGENTS.md` between its markers, in each `CLAUDE.md` there is; it creates one only with `--target claude`. | Yes |
| `.agents/skills/ascribe/` and `.claude/skills/ascribe/` {available="ascribe 0.3.0"} | `ascribe agents sync`, at the repository's root: the skill, the second with `--target claude` ([The skill](../agents/index.md#the-skill)). | Yes |
| `.claude/rules/ascribe*.md` and `.github/instructions/ascribe*.instructions.md` {available="ascribe 0.3.0"} | `ascribe agents sync`, at the repository's root, with `--target claude-rules` and `--target copilot` ([Other agents' files](../agents/index.md#other-agents-files)). | Yes |
| `.claude/settings.json`, `.mcp.json`, `.codex/hooks.json`, and `.github/hooks/ascribe.json` {available="ascribe 0.3.0"} | `ascribe agents sync --with-hook`, at the repository's root: the hooks' entries and the MCP server, beside your own ([Hooks](../agents/index.md#hooks)). | Yes |
| `.github/workflows/copilot-setup-steps.yml` and `.github/agents/ascribe-docs.md` {available="ascribe 0.3.0"} | `ascribe agents sync --cloud`, at the repository's root: steps between markers in the workflow, and with `--agent`, the custom agent ([GitHub Copilot](../agents/index.md#github-copilot)). | Yes |
| The files you name with `--into` and `--pages` {available="ascribe 0.3.0"} | [`ascribe agents index`](../agents/commands.md#cli-agents-index): the index of the docs between its markers in the `--into` file, such as your product's `AGENTS.md`, and the pages' Markdown versions in the `--pages` folder ([Index your docs for your product's agents](../agents/index.md#index-your-docs-for-your-products-agents)). | Yes |
| `src/content.config.ts`, `src/pages/[...slug].astro`, `src/layouts/Docs.astro`, and `src/nav.ts` | `npx @ascribed/astro init`, once, in the Astro root. From then on they're yours ([Set up a site](../astro/index.md#set-up-a-site)). | Yes |
| `.astro/integrations/_ascribed_astro/` | `@ascribed/astro`, in the Astro root: a copy of the collection's schema, inside Astro's own folder, which Astro's starters leave out of git. | No |

## What deleting them costs

Everything under `.ascribe/` is written again when it's needed: deleting it costs the next build, check, or report time, and nothing else, apart from `.ascribe/history/` and `.ascribe/memos/`. Without `.ascribe/history/`, the next `ascribe report links` reports every failing link, as a first run does, including one that would have been held as unconfirmed: one noisier report, and never a missed problem. Your personal memos are yours alone, so deleting `.ascribe/memos/` loses them.

A file you commit is a record, not a cache. Without `ascribe.lock` and `sources/`, `ascribe check` reports each snippet from another repository until `ascribe sources fetch` writes both again. Without `ascribe.permalinks`, a product's `ascribe permalinks verify` has no list to check against, and `ascribe check` in the docs can't warn when one is removed.

## Files of yours that Ascribe changes

A few commands change files you wrote, in place, and only when you run them: `ascribe permalinks retire` and `ascribe vale eject` change `ascribe.toml`, keeping its comments, and `ascribe templates eject` adds to its end; [`ascribe fmt`](../checks/check.md#cli-fmt), [`ascribe rename`](questions.md#cli-rename), and `ascribe check --fix` change your pages, and `ascribe rename` changes `ascribe.toml` too, keeping its comments; and the editor's quick fixes change the file the problem is in.

Outside the project, Ascribe keeps the repositories `ascribe sources` fetches, and the agents' hooks keep a check server's files, in a folder of your user's cache folder. Nothing there needs committing or keeping.
