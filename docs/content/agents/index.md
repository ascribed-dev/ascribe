---
type: how-to
title: Agents
description: Working on an Ascribe project with an AI coding agent, such as Claude Code, GitHub Copilot, or Cursor, and handing it the problems Ascribe finds.
available: ascribe 0.3.0
permalink: agents-guide
---

An agent is the AI coding tool you work with: Claude Code, GitHub Copilot, Cursor, Codex, and others. Ascribe gives your agent what it knows about a project, in a form an agent reads well, and checks what the agent writes:

- **Commands that answer its questions**, and a check it can run on one file after each edit. See [The loop](index.md#the-loop).
- **Instructions it reads on its own**: your project's rules in `AGENTS.md`, and a skill that teaches it the loop. See [Agent instructions](index.md#agent-instructions).
- **Prompts you hand it**: **Prompt agent** on a problem, a review comment, or a changed page. See [Prompt your agent](index.md#prompt-your-agent).
- **The same answers as tools**, for an agent without a shell: [the MCP server](index.md#the-mcp-server).
- **A check it can't forget**: [hooks](index.md#hooks) that check each edit, and the whole project before it finishes.

What Ascribe doesn't do: it doesn't call a model, holds no key for one, and has no chat of its own. It never sends a prompt, and never writes a page for your agent: its tools answer questions and return edits, and your agent makes them, with your approvals. See [What's sent where](index.md#whats-sent-where).

Nothing here is set up until you run a command for it. The quickest start, in a project's folder:

```shell
ascribe agents sync
```

That writes `AGENTS.md` and the skill, which most agents read on their own. Then add what your agent can use: [the MCP server](index.md#the-mcp-server), [the hooks](index.md#hooks), or, in Claude Code, [the plugin](index.md#the-plugin).

This guide writes commands as `ascribe`. With `@ascribed/cli` installed in your project, as [Getting started](../getting-started.md#install-the-command) does, run them as `npx ascribe` (`pnpm exec ascribe` with pnpm); your agent does the same.

## The loop

An agent working alone runs one loop: edit a page, check it, fix what's reported, and check the whole project before it finishes. The check takes a file, and has a form made for agents:

```shell
ascribe check guides/install.md --format concise
```

It writes one line per problem, `file:line: [code] message`, with what to change in the message, and at most 50 lines, then the command that shows the rest. With `--editor-build`, it checks only the editor's build, which is quick enough to run after every edit on a project of thousands of pages; a plain `ascribe check` before finishing checks every build. `--stdin --path <file>` checks text that isn't saved yet. On a project with hundreds of problems, `ascribe check --summary` counts them by code and by file, and `ascribe check --code <code>` shows one code's problems across the project, so an agent can work through one rule, or one file, at a time. See [Checking some files](../checks/check.md#checking-some-files).

`ascribe check` exits with `0` when there are no errors, `1` when there are, and `2` when it couldn't check at all. `--deny-warnings` makes warnings count too. Advice, below a warning, never counts. Each diagnostic in the JSON says what kind of next step it has (`next`): a `fix` Ascribe can apply, a `choose` among things it lists, or a `write` that needs judgment. See [What to do next](../checks/diagnostics.md#what-to-do-next).

Other commands answer what an agent would otherwise guess at. Each changes nothing, each takes `--format json`, and each long list is cut with the command that gives the rest:

| Command | What it answers |
|---|---|
| [`ascribe explain <code>`](../command/questions.md#cli-explain) | What a diagnostic means, how to fix it, and a page with the problem and without it |
| [`ascribe model`](../command/questions.md#cli-model) | What the content model allows: page types and their frontmatter, dimensions, phrases, features, glossary terms, widgets, builds, and sources |
| [`ascribe outline <page>`](../command/questions.md#cli-outline) | A page's title and type, and the headings a link can name, with their ids |
| [`ascribe link <target> --from <page>`](../command/questions.md#cli-link) | Whether a link works from a page, and what to write when it doesn't |
| [`ascribe render <page> --build <name>`](../command/questions.md#cli-render) | The page as one build's readers see it, as plain Markdown |
| [`ascribe refs <target>`](../command/questions.md#cli-refs) | Every place that uses a page, a heading, a phrase, or another entry of the content model, such as the links to fix after renaming a heading |

`ascribe check --format json` gives each problem's `fixes` too: edits that would fix it, each marked `safe` to apply as given, or `unsafe`, to read first.

To rename a phrase, a dimension's value, a heading's id, or a page, an agent runs [`ascribe rename <target> <new>`](../command/questions.md#cli-rename), which changes every use as the editor's rename does, where a search and replace would miss some: the code a `@snippet {phrases=true}` shows, frontmatter fields, and the links on a page that moves.

### Fix it, or say it's intended
@available: ascribe 0.3.0

A problem whose next step is `review` may be fine as it is: a page nothing links to because the site's sidebar does, say. Fixing it is still the first answer, and an agent should fix it unless you tell it the problem is intended. When it is, the answer is an [acknowledgement](../writing/directives.md#intended): the check's name and your reason, written where the check reports the problem, so the next person reads why. Don't acknowledge a problem to make a check pass, and don't let an agent do it on its own: a prompt about a review check says to acknowledge it only when you've said so. The editor's **Mark as intended, and write why** writes a placeholder reason, which is reported until you replace it.

## Agent instructions

`ascribe agents sync` writes your project's rules into the files agents read on their own, from `ascribe.toml`:

```shell
ascribe agents sync
```

It writes:

- **`AGENTS.md`** beside `ascribe.toml`: the check to run after each edit and before finishing, your page types with what each is for and its frontmatter, and that each page names its type with `type:`, your phrases, the directives your pages use, and how to link. It's at most 4,000 characters, and long lists are cut with the command that shows the rest, the page types first.
- **A short block in the repository's root `AGENTS.md`,** when your project is in a subfolder: where its pages are, the check to run, and where its full rules are. Some agents, such as Codex, read no `AGENTS.md` below the folder they start in.
- **The skill,** in `.agents/skills/ascribe/` at the repository's root. See [The skill](index.md#the-skill).

Beside each file it writes, `sync` says which agents read it, and when an agent doesn't read one, the target that writes it where that agent does. Claude Code doesn't read the skill in `.agents/skills/`, so without the `claude` target, `sync` says so:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/agents-sync.txt

@available: next
When a page has a [`@memo`](../writing/directives.md#memo), `AGENTS.md` also tells the agent to read a block's memo before changing the block, and to keep the two together. When the project has a [memo file](../writing/memo-files.md) in `memos/`, a sentence after it says that `ascribe outline <page>` lists the memos about a page and its headings, and to read a memo before changing the block it's about. A [personal memo](../writing/memo-files.md#personal-memos) doesn't count, so the rules are the same on every machine. When the rules run over their 4,000 characters, those sentences are cut first, before the page types.

What a type is for is its `purpose` in `ascribe.toml`, or, for a type [`ascribe templates eject`](../templates/index.md) added, The Good Docs Project's summary of it, from `templates.lock` (the [content model reference](../content-model/index.md#41-page-types)).

Here's what it writes for the Quill example:

@snippet {lang=markdown}: code:crates/ascribe-cli/tests/output/agents/quill/AGENTS.md.snap

### Your text stays yours

Ascribe writes only between its two markers, `<!-- ascribe:agents start … -->` and `<!-- ascribe:agents end -->`. Everything else in the file is your team's, and is left byte for byte as it was: write your own rules above or below the block. A file with no markers gets the block at its end, and a file that doesn't exist is created with only the block. When a file has a start marker and no end marker, or two of either, `sync` changes nothing anywhere and exits with `2`: fix the markers by hand.

Don't edit the block itself: change `ascribe.toml`, or your pages, and run `sync` again.

### Other agents' files

Beyond `AGENTS.md` and the skill, `sync` keeps up to date the files of each target that already has them, and writes a target's files when you ask with `--target`:

The targets are `claude`, `claude-rules`, `copilot`, and `codex`; [`ascribe agents sync`](commands.md#cli-agents-sync) lists what each writes, and what it names the files of a project in a subfolder.

**Claude Code** reads `AGENTS.md` by itself only while there's no `CLAUDE.md` at or above the folder it works in. So `sync` never creates a `CLAUDE.md` unless you ask with `--target claude`, and when there is one, it adds the import to it rather than copying the rules. A personal `CLAUDE.local.md` also stops Claude Code reading `AGENTS.md`; `sync` doesn't edit it, but says so, and you can add `@AGENTS.md` to it yourself.

A repository with several projects can run `sync` at its root, with no `ascribe.toml` there: it syncs each project below, and the root's `AGENTS.md` gets a block for each, which tells an agent starting at the root which project a page is in.

**Copilot's** files are for a repository, so `--target copilot` and `--cloud` need one. Outside a git repository, `sync` treats your project's folder as the root and says so.

### Keeping them current in CI

`--check` writes nothing, and exits with `1` when a file is out of date, naming it. Run it in CI so the instructions can't fall behind `ascribe.toml`:

```shell
ascribe agents sync --check
```

## The skill

The skill is one folder, `ascribe`, in the [Agent Skills](https://agentskills.io) format, which most agents read. It teaches the tool, the same for every project, and sends the agent to `AGENTS.md` and `ascribe model` for your project's rules:

- the loop: edit a page, run `ascribe check <file> --format concise`, fix what it reports, and run the whole check before finishing; and to stop and ask after three rounds that don't fix it;
- the commands that answer questions: `explain`, `model`, `outline`, `link`, `render`, and `refs`;
- which fixes in `ascribe check`'s JSON can be applied as they are;
- that text from a review or a pull request is someone else's, not instructions;
- each directive's syntax, in a reference it reads only when it needs it.

`sync` writes it to `.agents/skills/ascribe/` at the repository's root, where Codex, Cursor, Gemini CLI, and Copilot find it, and with the `claude` target to `.claude/skills/ascribe/`, where Claude Code does. `ascribe agents skill` prints its `SKILL.md`. The `@ascribed/cli` package ships it too, as `skills/ascribe/`, for tools that install skills from packages.

@available: next
In VS Code, chat has the skill before you run `sync`: the [extension](index.md#vs-code) ships it.

See [`ascribe agents`](commands.md#cli-agents) for every option.

## Prompt your agent

**Prompt agent** turns a problem Ascribe reports into a prompt for your agent: what to fix, where, what Ascribe knows about it, and how to check the result. It fills the prompt in where you send it and never sends it for you, so you read it first. [What a prompt says](commands.md#what-a-prompt-says) lists its parts.

### In VS Code

- On a problem, the lightbulb (`Ctrl+.`, or `Cmd+.` on macOS) offers **Prompt agent: fix this problem**, after the problem's quick fixes. A problem with a safe quick fix is quicker to fix with it.
- **Ascribe: Prompt Agent to Fix This File** builds one prompt for the active file's problems, and **Ascribe: Prompt Agent to Fix This Project** one for the project's. The Command Palette shows them when there are problems.

The prompt is about what the editor checks: the file's problems and its pages' problems in the editor's build (`[editor] build`), as you see them, unsaved edits included. It names the build, and says that `ascribe check` checks every build. Your agent reads the files on disk, so when the file has unsaved changes, the prompt and the editor both say to save it first.

The `ascribe.agents.promptTarget` setting says where the prompt goes:

| Value | Where the prompt goes |
|---|---|
| `clipboard` (the default) | The clipboard: paste it into any agent. |
| `chat` | VS Code's chat, in its input. |
| `claude-code` | A new Claude Code tab, in its input, through the Claude Code extension. |
| `cursor` | Cursor's chat, in its input, through its `cursor://` link. Only in Cursor. |

Each one fills the prompt in and leaves it for you to send. When the one you chose isn't there (the extension isn't installed, or the editor isn't Cursor), the prompt is copied instead, and the status bar says so. The first time you prompt an agent, a notification offers the others your editor has, once.

### From the command line

`ascribe check --format prompt` writes the same prompt to standard output, for an agent in a terminal. With a file named, the prompt is about that file; otherwise, about the paths named or the whole project. A file's or a project's prompt counts what it's about as `ascribe check`'s summary does, and asks the agent to fix the errors and warnings; advice, which never fails the check, comes after them as something to look at. It writes nothing when there are no problems, and exits as `ascribe check` does. For a file on disk, `ascribe check <file> --editor-build --format prompt` writes the prompt the editor builds.

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/check-prompt.txt

`ascribe agents prompt` prints the other prompts: `new-page` to write a page of a type; `fix` to check the project and fix what's reported; and `review` to review what a branch does to its pages, the prompt `ascribe diff --format prompt` writes. `ascribe agents prompt --list` lists them with their arguments:

```shell
ascribe agents prompt new-page --arg type=how-to --arg title="Rotate keys"
```

`new-page` says where the page goes: where its `path` says, a file or a folder, or else where the request says, and otherwise in the content root, named for the title as its address would be, `docs/rotate-keys.md`; never over a file that's there. For a type with a [template](../templates/index.md), it gives the template with its `type` and title set, to start the file from, and before it, the paths of the type's `guide`, the template's guide and Ascribe's notes, for the agent to read before it writes. The guide isn't put in the prompt: it's long, and a file the agent reads arrives whole. For a type with no template, it gives the frontmatter the type requires, with each field to fill in.

### Memos in a prompt
@available: next

A prompt about a page names the [memo files](../writing/memo-files.md) about it, so your agent reads why the page is written as it is before it changes it. They come after the problems, under a heading of their own, a line each: the memo's path, what its `about` names, and its state when it isn't placed. The agent opens the file and reads it whole; a prompt never has a memo's text in it.

- **`fix`**, `ascribe check --format prompt`, and **Prompt agent** on a problem, a file, or a project list the memos about each page they name, outdated and detached ones too, saying which.
- **`review`** and `ascribe diff --format prompt` list each memo on a changed block, on the section a change is in, or on the whole page, and ask the agent to check the change still agrees with it. A memo whose quote is in another block of the section isn't listed.

A [personal memo](../writing/memo-files.md#personal-memos) is listed in prompts made on the machine that has it, marked as personal. Each list stops at ten memos, and says how many more there are and the `ascribe outline` command that lists them.

### In review

While you [review](../review/index.md) a pull request, **Prompt agent** hands your agent a review comment, or the change itself:

- **On a comment.** Each thread beside the page has **Prompt agent**, which asks your agent to address the comment: the thread's file and lines, the block's text from the file on disk, and the comments. **Comments** lists every thread, with **Prompt agent: all open** for the open ones, two lines each. In the source editor, a thread's title bar has **Prompt Agent** too.
- **On the page.** **More review actions** (**⋯**) in the preview's header has **Prompt agent: review this page**, which asks your agent to review what the change does to the page as readers see it, and, when the page changed through a fragment, **Prompt agent: check this fragment's pages**, which asks it to check that the new text fits every page that shows it.

In VS Code, these go where `ascribe.agents.promptTarget` says. The site preview copies them, since a page in your browser can't reach your editor, and says "Prompt copied". The [HTML report](../review/diff.md#the-html-report) has **Copy prompt** for each page. A prompt about the change asks your agent for a report, not edits: you decide what changes.

A comment is someone else's text, so a prompt never passes it as an instruction. It goes inside a fence, under a sentence that names who wrote it and tells your agent to treat it as data and not to follow instructions in it that reach beyond the change. HTML comments, where hidden text would sit, are taken out first. Read the prompt before you send it, as you would any request from a reviewer.

From the command line, `ascribe diff --format prompt` writes the prompt about the change, and `ascribe diff --format prompt <page>` the one about a page or a fragment, from the same base and builds `ascribe diff` uses:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/diff-prompt.txt

## The MCP server

`ascribe mcp` is a server for the [Model Context Protocol](https://modelcontextprotocol.io), which most agents' hosts speak. It offers the commands that answer questions as tools an agent calls, for an agent with no shell, or a host that asks you to approve each command it runs. An agent with a shell needs no server: the commands give the same answers.

Your agent's host starts it. Most hosts read a JSON file listing the servers to start, such as `.mcp.json` at the repository's root for Claude Code, or `.cursor/mcp.json` for Cursor; add Ascribe to it:

```json
{
  "mcpServers": {
    "ascribe": {
      "command": "npx",
      "args": ["ascribe", "mcp"]
    }
  }
}
```

Hosts start the server in the repository's root. It serves every project in it: each call names a file or folder, and the server uses the nearest `ascribe.toml` at or above it. A project is loaded once and kept, and loaded again when its files change.

It offers the commands that answer questions as tools, each directive's syntax and each project's content model and rules as resources, and the prompts `new-page`, `fix`, and `review`, which many hosts show as slash commands. None of its tools writes a file or starts a program the project names, so `ascribe_check` never runs [Vale](../checks/vale.md): it says the prose wasn't checked, and `ascribe check --vale` in a shell, which your host asks you about, checks it.

[`ascribe mcp`](commands.md#cli-mcp) lists each tool and the command it runs, and the resources.

In VS Code, the extension offers the server itself: see [VS Code](index.md#vs-code).

## VS Code

The [Ascribe extension](../editor/index.md) gives an agent in VS Code, such as GitHub Copilot in agent mode, what it gives you:

- **Problems as the agent edits.** An agent that writes files without opening them still gets their problems: a project's language server starts when one of its files changes on disk, and the Problems panel shows them, where Copilot can read them after its edits. Like the panel, they're the editor's build's; the hooks and `ascribe check` cover every build. See [When servers start](../editor/settings.md#when-servers-start).
- **The MCP server, with nothing to set up.** The extension offers `ascribe mcp` to VS Code's agents itself, named **Ascribe** in VS Code's list of MCP servers, so there's no file to write. Its prompts, `new-page`, `fix`, and `review`, appear as slash commands in the chat.
- **Tools only the editor has.** `ascribe_editor_problems`, the problems of files as the editor has them, saved or not; and, while review is on, `ascribe_review_threads`, the pull request's open threads, and `ascribe_review_changes`, the changed pages. Name one in chat with `#`, or let the agent pick it.
- **Prompt agent**, on a problem, a file, a project, or in review, as [Prompt your agent](index.md#prompt-your-agent) says.
- **The binary the editor runs.** When `ascribe` on the window's path isn't the binary the extension runs, such as the one bundled with it, the extension puts that binary's folder first on the path of the window's terminals, so `ascribe` in a terminal runs it. A terminal that was open before asks to be relaunched. **Prompt agent**'s prompts then name the binary by its path, so an agent that doesn't work in the window's terminals, such as Claude Code's extension, runs the same one.

@available: next
- **The skill, with nothing to set up.** The extension gives VS Code's chat [the skill](index.md#the-skill) in a project that never ran `ascribe agents sync`. Once a workspace folder has a synced copy, in `.agents/skills/ascribe/`, `.claude/skills/ascribe/`, or `.github/skills/ascribe/`, chat lists that one, the project's, and not the extension's. VS Code reads those folders at the top of each workspace folder, and `sync` writes at the repository's root, so a workspace opened at a project below the root, such as `docs/`, gets the extension's copy even after `sync`.

See [Agents in VS Code](../editor/index.md#agents-in-vs-code).

## Hooks

A hook is a command your agent runs at a point in its work. Ascribe's checks an agent's work without the agent having to remember: each file it writes, as it writes it, and the whole project before it finishes. One command serves Claude Code, Codex, and GitHub Copilot, which share a hook format, with the agent as its argument, `claude-code`, `codex`, or `copilot`:

- **After an edit**, `ascribe agents hook <agent>` checks the files the agent wrote, as `ascribe check <file> --editor-build` does, and adds their errors to what the agent reads next, at most 10, with the build it checked. It adds the advice in them that has a safe fix too, each with what its fix changes and where, such as "Add `new.md` to `guides/`'s pages in ascribe.nav" for a page the [navigation file](../navigation/index.md) doesn't place; the hook makes no fix, so the agent makes the edit itself and sees every change. It says nothing when there's neither, and leaves warnings and other advice out, which would interrupt every edit.
- **Before the agent finishes**, `ascribe agents hook <agent> --event stop` checks every build of each project whose pages or `ascribe.toml` the working tree changes, anywhere in the repository. While there are errors, or a project can't be checked at all, it keeps the agent working, and tells it why. It asks once: when the agent is already continuing because of it, it lets the agent stop. Changes the agent has committed before it stops aren't in the working tree, so they aren't checked; check them in CI.

@available: ascribe 0.3.0
A project that checks its prose with [Vale](../checks/vale.md) has the hooks run it only with `in-check = true` in `[checks.vale]`, as `ascribe check` does: after an edit on the files the agent wrote, and before it finishes on the whole project. Vale gets half of what the hook waits; when it takes longer, its alerts are left out, and the rest of the check reaches the agent as usual. Only Vale's error-level alerts reach the agent, as with every other check; with `max-level = "advice"` or `"warning"`, none do. A hook can't ask you before it runs a program, so it runs Vale only when `command` is a name found on your `PATH`, such as `vale`, never when it's a path to a program in the project: then it tells the agent, with the errors it reports, that the prose wasn't checked and that `ascribe check --vale` checks it.

Neither stops an edit, writes a file, or holds the agent up: a check that takes too long is given up. The first hook in a project starts a check server in the background, which keeps the project loaded, so a check after an edit takes milliseconds on a project of thousands of pages, and stops itself after 10 idle minutes. See [`ascribe agents hook`](commands.md#cli-agents-hook).

`ascribe agents sync --with-hook` writes the entries that run them, with the targets you name or already have:

```shell
ascribe agents sync --target claude --with-hook
```

[`ascribe agents sync`](commands.md#cli-agents-sync) lists the file and the entries for each target, and how it merges them into a file you share.

The entries run `ascribe` from your path. Put the release's binary on your path, with `brew install ascribed-dev/tap/ascribe` or the install script ([Getting started](../getting-started.md#without-node) has both), or install it with `npm install -g @ascribed/cli`; the npm package's launcher adds about 60 ms to each run, more than a check after an edit takes, so prefer the binary for hooks. When your project pins `@ascribed/cli` in its `node_modules`, the hooks run that binary instead, found from the repository's root (`$CLAUDE_PROJECT_DIR` for Claude Code, `git rev-parse --show-toplevel` for Codex and Copilot), so an agent started in a subfolder finds it too. Two entries still run `ascribe` from your path, because neither can find the root on every system: the MCP server in `.mcp.json`, and Codex's hooks on Windows. Codex asks you to trust a project's hooks before it runs them.

For Claude Code, use the plugin or the project's entries, not both: with both, it hears about each problem twice.

## The plugin

The Ascribe plugin, in `plugins/ascribe/` in Ascribe's repository, gives Claude Code everything in one install:

- **The skill**, for every agent: how to write, check, and fix pages.
- **The MCP server**, whose tools are for an agent with no shell; one with a shell does as well with the commands.
- **The stop hook**, and **the language server**, which Claude Code starts for `.md` files and whose diagnostics it reads after each edit: they check for an agent that doesn't check its own work. The language server reports the editor's build, as the edit hook does, and only for edits Claude Code makes with its own tools, not through its shell.
- **Commands:** `/ascribe:check`, `/ascribe:new-page`, and `/ascribe:review`, which run the [named prompts](commands.md#cli-agents-prompt). `/ascribe:new-page` takes what the page is for: the agent chooses the page type from `ascribe model`, then runs the prompt.

Add it from a copy of the repository, in Claude Code in a terminal (its VS Code extension doesn't offer `/plugin`):

```shell
/plugin marketplace add ./plugins/ascribe
/plugin install ascribe@ascribe
```

The plugin has both kinds of manifest: `.claude-plugin/plugin.json`, which Copilot's CLI reads too, and the Agent Plugins `plugin.json`, which VS Code and Cursor read. Claude Code doesn't start a plugin's language server in a cloud session; there, use the project's entries from `--with-hook`, which Claude Code reads from the repository. To keep the language server's diagnostics out of the conversation on a project with many problems, see [the plugin's README](https://github.com/ascribed-dev/ascribe/tree/main/plugins/ascribe#readme); the stop hook still checks before Claude finishes.

## GitHub Copilot

In VS Code, Copilot reads `AGENTS.md`, the rules from `--target copilot`, and the skill, and the Ascribe extension gives it [the MCP server](index.md#vs-code), whose prompts `new-page`, `fix`, and `review` appear as slash commands in its chat. There's nothing more to set up.

Copilot's cloud agent works on GitHub, with no editor: you assign it an issue and it opens a pull request. It reads the skill from `.agents/skills/` and the hooks from `.github/hooks/`, as other agents do, but it needs `ascribe` installed, and it reads MCP servers only from the repository's settings. `--cloud` writes the rest:

```shell
ascribe agents sync --target copilot --with-hook --cloud
```

- **The setup steps.** In `.github/workflows/copilot-setup-steps.yml`, the job the cloud agent runs before it starts, steps that install Node.js and `@ascribed/cli` and run `ascribe --version`. When your project pins `@ascribed/cli` in a `package.json`, they install that folder's dependencies by its lockfile, without changing it (`npm ci`, `pnpm install --frozen-lockfile`, or `yarn install` with `--frozen-lockfile` for Yarn 1 and `--immutable` for later versions), and put its `node_modules/.bin` on the path. Otherwise they install the version of `ascribe` that wrote them, globally. Yarn's Plug'n'Play, the default from Yarn 2 on, writes no `node_modules/.bin`, so with it they install the pinned version globally, and `sync` stops if that version isn't one npm can install, such as `workspace:*`. `sync` creates the workflow when there's none, and otherwise adds its steps, between markers, at the end of the `copilot-setup-steps` job's steps, leaving your steps as they are. Projects in one repository that install the same way share one block.
- **The MCP server.** No file can change a repository's settings, so `sync` prints the server's JSON, a local server running `ascribe mcp` with each of its tools allowed (for a pinned project, the pinned binary, found from the repository's root with `git` as the hooks find it), to paste into the repository's **Settings → Copilot → MCP servers**. If you can't change the settings, add `--agent`: `sync` writes `.github/agents/ascribe-docs.md` instead, a custom agent for documentation work that carries the server, which you pick when you assign the cloud agent a task. When the repository's projects install Ascribe in different ways, so the setup steps have a block for each, the JSON and the custom agent carry a server for each block, under names told apart as the blocks are: `ascribe` for a global install, and `ascribe-docs` for one from `docs/package.json` (`ascribe-pinned` from the root's). Syncing one project prints them all, as one document to paste in place of any `ascribe` server pasted before.

GitHub reads all of these from the default branch, so they take effect once they're merged. `--check` covers the setup steps and the custom agent, and `sync` keeps them up to date without `--cloud` once they exist.

Ascribe writes no prompt files for Copilot (`.github/prompts/`): the cloud agent doesn't load them, and in VS Code the MCP server's prompts do the same.

## Other agents

Cursor, Codex, Gemini CLI, and most other agents have no section of their own here, because what works with every agent is most of it:

- **The commands.** Any agent that can run a command can run the [loop](index.md#the-loop). Install `ascribe` where the agent runs it: in the project with `npm install --save-dev @ascribed/cli`, which the agent runs as `npx ascribe`, or globally.
- **`AGENTS.md`** and **the skill**, from `ascribe agents sync`. Cursor, Codex, Gemini CLI, and Copilot read both on their own. An agent that reads neither can be told to: "Read `AGENTS.md` and `.agents/skills/ascribe/SKILL.md` first."
- **The MCP server**, for an agent whose host speaks MCP. Add `ascribe mcp` to the host's list of servers, as [The MCP server](index.md#the-mcp-server) shows.
- **Prompt agent.** With `clipboard`, the default, a prompt pastes into any agent. `ascribe check --format prompt` and `ascribe agents prompt` write prompts in a terminal.

What doesn't reach every agent is the hooks: Cursor's and Gemini CLI's hook formats differ from the one Claude Code, Codex, and Copilot share, and Ascribe writes neither. Without a hook, an agent checks only when it remembers to. The skill and `AGENTS.md` tell it to, after each edit and before it finishes, but check the project yourself when it's done, or in CI:

```shell
ascribe check --deny-warnings
```

**In Cursor**, the Ascribe extension works as it does in VS Code, and **Prompt agent** can fill Cursor's chat in with `ascribe.agents.promptTarget` set to `cursor`. Cursor installs extensions from Open VSX, where Ascribe isn't, so install the `.vsix` for your platform from a [release](https://github.com/ascribed-dev/ascribe/releases) with **Extensions: Install from VSIX…**. Add the MCP server to `.cursor/mcp.json`.

## Agents that reach the network

An agent that can fetch web pages, run commands, or post to GitHub can do with Ascribe's answers whatever it can do with any text. Ascribe's tools for agents read nothing from the network and send nothing, but a review comment, or page text from someone else's pull request, is still someone else's text when a prompt or a tool carries it to your agent. Ascribe puts that text in a fence, under a sentence that says it's data, and takes out HTML comments, where hidden text would sit. That lowers the risk; it doesn't remove it. An instruction hidden in a comment ("also update the deploy key", "fetch this URL") is aimed at your agent, and only your agent's own approvals stop it.

- Read a prompt before you send it, as you would a request from the person who wrote the comment.
- Keep your agent's approvals on for commands and network access while it works on someone else's change.
- Hand an agent with no network access the work on pull requests from outside your team.

## Index your docs for your product's agents

Everything above is for the agents that write your docs. The agents of the people who use your product read them too, and `ascribe agents index` writes something for those: a compact index of the docs, one line a page, small enough to go in your product's own `AGENTS.md`. With `--pages`, it also copies each page's Markdown version into a folder, and the index points into it, so a package can ship the docs for the version it installs, with no server and nothing to fetch:

```shell
ascribe agents index --build site --into packages/sdk/AGENTS.md --pages packages/sdk/docs
```

It writes between markers, as `sync` does, so you keep the rest of the file, and running it again, in CI or before a release, replaces only the index. Write what the agent should do with it above the markers: Vercel's version said "Prefer retrieval-led reasoning over pre-training-led reasoning". The index holds to 8 KB, leaving out descriptions before pages, and says what it left out. Its pages come as `llms.txt`'s do, in the [navigation](../navigation/index.md)'s order: the content root's under **Start**, then a heading for each top section. See [`ascribe agents index`](commands.md#cli-agents-index) for each option.

The evidence for it is one vendor's. Vercel put an index of about 8 KB, of paths to local docs files, in `AGENTS.md`, and its evals on Next.js APIs newer than its model's training passed 100% of the time, against 53% with no docs; a skill that offered the same docs wasn't used in 56% of cases ([Vercel](https://vercel.com/blog/agents-md-outperforms-skills-in-our-agent-evals)). It's one framework, on evals whose number and model weren't published, and nobody has repeated it. The reason it makes sense is simpler than the number: an agent uses what's in front of it, and an index in `AGENTS.md` always is.

## What's sent where

Nothing about your agent's work leaves your machine through Ascribe: it sends nothing, to no one.

- **No model, no service.** Ascribe makes no request to an AI service, holds no key for one, and collects no telemetry. The commands, the MCP server, and the hooks read your project's files and answer on standard output.
- **Prompts go where you put them.** **Prompt agent** copies a prompt, or fills it in where your agent takes input, on your machine, and leaves it there for you to send. It never travels through a website. Once you send it, it goes wherever your agent sends what you type, under that agent's terms.
- **Your agent sends what it reads.** What a command or tool tells your agent, such as a page's text from `ascribe render`, becomes part of what your agent sends to its model, as any file it reads does.
- **Review reads GitHub, as you.** In review, the extension and the site preview fetch the pull request's threads from GitHub with your sign-in, and post the comments you write, as [Review](../review/index.md) says. The prompts built from them stay on your machine. Apart from review, the only commands that reach the network are [`ascribe sources`](../examples/sources.md#cli-sources) `fetch` and `update`, which fetch the repositories your content model names, with `git`.
