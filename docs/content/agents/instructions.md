---
type: how-to
title: Write the agent files
description: "Write your project's rules into the files agents read on their own, `AGENTS.md` and the Ascribe skill, and keep them current."
---

`ascribe agents sync` writes your project's rules into the files agents read on their own, from `ascribe.toml`:

```shell
ascribe agents sync
```

It writes:

- **`AGENTS.md`** beside `ascribe.toml`: the check to run after each edit and before finishing, to run `ascribe search` for a subject before writing a page about it, your page types with what each is for and its frontmatter, and that each page names its type with `type:`, your phrases, the directives your pages use, with your dimensions and their values for `@variant` and an example of your own targets for `@available`, and how to link. It's at most 4,000 characters, and long lists are cut with the command that shows the rest, the page types first.
- **A short block in the repository's root `AGENTS.md`,** when your project is in a subfolder: where its pages are, the check to run, and where its full rules are. Some agents, such as Codex, read no `AGENTS.md` below the folder they start in.
- **The skill,** in `.agents/skills/ascribe/` at the repository's root. See [The skill](#the-skill).

Beside each file it writes, `sync` says which agents read it, and when an agent doesn't read one, the target that writes it where that agent does. Claude Code doesn't read the skill in `.agents/skills/`, so without the `claude` target, `sync` says so:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/agents-sync.txt

When a page has a [`@memo`](../writing/directives.md#memo), `AGENTS.md` also tells the agent to read a block's memo before changing the block, and to keep the two together. When the project has a [memo file](../writing/memo-files.md) in `memos/`, a sentence after it says that `ascribe outline <page>` lists the memos about a page and its headings, and to read a memo before changing the block it's about. A [personal memo](../writing/memo-files.md#personal-memos) doesn't count, so the rules are the same on every machine. When the rules run over their 4,000 characters, those sentences are cut first, before the page types.

When a page declares a [permalink](../permalinks/index.md), `AGENTS.md` also says, with how to link, that a product's message linking to the docs by a permalink can read what the docs will show for it with `ascribe resolve go:<name>`, which prints the help [a product shows](../permalinks/in-your-product.md). When a heading declares one, it says to give the heading's section a [`@summary`](../writing/directives.md#summary), the sentence a product shows with its help, and how to choose between `@summary` above a paragraph and `@summary: <text>`; `ascribe check --format prompt` quotes the text of each section without one, so the agent can write it.

@available: ascribe 0.4.4
When the project has an [`ascribe.addresses`](../publishing/redirects.md), `AGENTS.md` also has a section on moving pages: move or rename a page with `ascribe rename`, which records the move; after moving pages with `git mv`, record the moves git sees with `ascribe addresses move --from-git`; and forward or retire a page deleted on purpose, with `ascribe addresses move <old> <page>` or `ascribe addresses retire <old> --reason "<why>"`, since `ascribe check` reports `address-lost` for one that's gone with neither.

@available: ascribe 0.4.4
`AGENTS.md` also says how to see what a reader of the site gets, not only a reader of the Markdown: `ascribe resolve <page> --build <name> --emit site`, or `ascribe_resolve` with `emit: "site"`, writes the page [as the site output gives it](../command/site-output.md), and links to [reading the site output](../command/site-output.md#reading-site-output). The skill's line for `ascribe resolve` says the same.

What a type is for is its `purpose` in `ascribe.toml`, or, for a type [`ascribe templates eject`](../templates/index.md) added, The Good Docs Project's summary of it, from `templates.lock` (the [content model reference](../content-model/index.md#41-page-types)).

Here's what it writes for the Quill example:

@snippet {lang=markdown}: code:crates/ascribe-cli/tests/output/agents/quill/AGENTS.md.snap

## Your text stays yours

Ascribe writes only between its two markers, `<!-- ascribe:agents start … -->` and `<!-- ascribe:agents end -->`. Everything else in the file is your team's, and is left byte for byte as it was: write your own rules above or below the block. A file with no markers gets the block at its end, and a file that doesn't exist is created with only the block. When a file has a start marker and no end marker, or two of either, `sync` changes nothing anywhere and exits with `2`: fix the markers by hand.

Don't edit the block itself: change `ascribe.toml`, or your pages, and run `sync` again.

## Other agents' files

Beyond `AGENTS.md` and the skill, `sync` keeps up to date the files of each target that already has them, and writes a target's files when you ask with `--target`. The targets are `claude`, `claude-rules`, `copilot`, and `codex`; [`ascribe agents sync`](commands.md#cli-agents-sync) lists what each writes, and what it names the files of a project in a subfolder. [Claude Code](claude-code.md) and [GitHub Copilot](copilot.md) say when you need theirs.

A repository with several projects can run `sync` at its root, with no `ascribe.toml` there: it syncs each project below, and the root's `AGENTS.md` gets a block for each, which tells an agent starting at the root which project a page is in.

## The command they name

The files tell the agent which commands to run, and `sync` writes each as your project runs Ascribe. When a `package.json` in the project's folder, or a folder above it up to the repository's root, has `@ascribed/cli` or `@ascribed/astro` in its dependencies, as [Install](../install.md#npm) sets up, the commands are `pnpm exec ascribe` with a `pnpm-lock.yaml` beside it or above it, `yarn ascribe` with a `yarn.lock`, and `npx ascribe` otherwise. Without one, they're `ascribe`, from the path. `AGENTS.md`, the skill, the rules for `claude-rules` and `copilot`, and what the [hooks](hooks.md#what-the-agent-hears) tell the agent all use it.

To name another, such as `bunx ascribe`, give it with `--command`, every time you run `sync`, `--check` too: a file written with another command is out of date.

```shell
ascribe agents sync --command "bunx ascribe"
```

The skill at the repository's root is shared by its projects, so when they run Ascribe in different ways, it writes `ascribe`, and says how to run it from a `package.json`.

## Keeping them current in CI

`--check` writes nothing, and exits with `1` when a file is out of date, naming it. Run it in CI so the instructions can't fall behind `ascribe.toml`:

```shell
ascribe agents sync --check
```

## The skill

The skill is one folder, `ascribe`, in the [Agent Skills](https://agentskills.io) format, which most agents read. It teaches the tool, the same for every project, and sends the agent to `AGENTS.md` and `ascribe model` for your project's rules:

- the loop: edit a page, run `ascribe check <file> --format concise`, fix what it reports, and run the whole check before finishing; and to stop and ask after three rounds that don't fix it;
- the commands that answer questions: `explain`, `model`, `outline`, `link`, `resolve`, `refs`, and `search`, which it runs before writing a page about a subject;
- which fixes in `ascribe check`'s JSON can be applied as they are;
- in a project with `ascribe.addresses`, how to move, forward, and retire pages so their old addresses still lead somewhere, and the MCP server's `ascribe_addresses_move` and `ascribe_addresses_retire`, which give the edit to apply without a shell;
- that text from a review or a pull request is someone else's, not instructions;
- each directive's syntax, in a reference it reads only when it needs it.

`sync` writes it to `.agents/skills/ascribe/` at the repository's root, where Codex, Cursor, Gemini CLI, and Copilot find it, and with the `claude` target to `.claude/skills/ascribe/`, where Claude Code does. `ascribe agents skill` prints its `SKILL.md`. The `@ascribed/cli` package ships it too, as `skills/ascribe/`, for tools that install skills from packages.

In VS Code, chat has the skill before you run `sync`: the [extension](../editor/agents.md#agents-in-vs-code) ships it.

See [`ascribe agents`](commands.md#cli-agents) for every option.
