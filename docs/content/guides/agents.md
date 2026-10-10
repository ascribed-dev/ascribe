---
title: Agents
description: Working on an Ascribe project with an AI coding agent, such as Claude Code, GitHub Copilot, or Cursor, and handing it the problems Ascribe finds.
available: next
---

An agent is the AI coding tool you work with: Claude Code, GitHub Copilot, Cursor, Codex, and others. Ascribe gives your agent what it knows about a project, in a form an agent reads well, and checks what the agent writes:

- **Commands that answer its questions**, and a check it can run on one file after each edit. See [The loop](#the-loop).
- **Instructions it reads on its own**: your project's rules in `AGENTS.md`, and a skill that teaches it the loop. See [Agent instructions](#agent-instructions).
- **Prompts you hand it**: **Prompt agent** on a problem, a review comment, or a changed page. See [Prompt your agent](#prompt-your-agent).
- **The same answers as tools**, for an agent without a shell: [the MCP server](#the-mcp-server).
- **A check it can't forget**: [hooks](#hooks) that check each edit, and the whole project before it finishes.

What Ascribe doesn't do: it doesn't call a model, holds no key for one, and has no chat of its own. It never sends a prompt, and never writes a page for your agent: its tools answer questions and return edits, and your agent makes them, with your approvals. See [What's sent where](#whats-sent-where).

Nothing here is set up until you run a command for it. The quickest start, in a project's folder:

```shell
ascribe agents sync
```

That writes `AGENTS.md` and the skill, which most agents read on their own. Then add what your agent can use: [the MCP server](#the-mcp-server), [the hooks](#hooks), or, in Claude Code, [the plugin](#the-plugin).

This guide writes commands as `ascribe`. With `@ascribed/cli` installed in your project, as [Getting started](../getting-started.md#install-the-command) does, run them as `npx ascribe` (`pnpm exec ascribe` with pnpm); your agent does the same.

## The loop

An agent working alone runs one loop: edit a page, check it, fix what's reported, and check the whole project before it finishes. The check takes a file, and has a form made for agents:

```shell
ascribe check guides/install.md --format concise
```

It writes one line per problem, `file:line: [code] message`, with what to change in the message, and at most 50 lines, then the command that shows the rest. With `--editor-build`, it checks only the editor's build, which is quick enough to run after every edit on a project of thousands of pages; a plain `ascribe check` before finishing checks every build. `--stdin --path <file>` checks text that isn't saved yet. On a project with hundreds of problems, `ascribe check --summary` counts them by code and by file, so an agent can work through one rule, or one file, at a time. See [Checking some files](../reference/cli.md#checking-some-files).

`ascribe check` exits with `0` when there are no errors, `1` when there are, and `2` when it couldn't check at all. `--deny-warnings` makes warnings count too.

Other commands answer what an agent would otherwise guess at. Each changes nothing, each takes `--format json`, and each long list is cut with the command that gives the rest:

| Command | What it answers |
|---|---|
| [`ascribe explain <code>`](../reference/cli.md#ascribe-explain) | What a diagnostic means, how to fix it, and a page with the problem and without it |
| [`ascribe model`](../reference/cli.md#ascribe-model) | What the content model allows: page types and their frontmatter, dimensions, phrases, features, glossary terms, widgets, and builds |
| [`ascribe outline <page>`](../reference/cli.md#ascribe-outline) | A page's title and type, and the headings a link can name, with their ids |
| [`ascribe link <target> --from <page>`](../reference/cli.md#ascribe-link) | Whether a link works from a page, and what to write when it doesn't |
| [`ascribe render <page> --build <name>`](../reference/cli.md#ascribe-render) | The page as one build's readers see it, as plain Markdown |
| [`ascribe refs <target>`](../reference/cli.md#ascribe-refs) | Every place that uses a page, a heading, a phrase, or another entry of the content model, such as the links to fix after renaming a heading |

`ascribe check --format json` gives each problem's `fixes` too: edits that would fix it, each marked `safe` to apply as given, or `unsafe`, to read first.

## Agent instructions

`ascribe agents sync` writes your project's rules into the files agents read on their own, from `ascribe.toml`:

```shell
ascribe agents sync
```

It writes:

- **`AGENTS.md`** beside `ascribe.toml`: the check to run after each edit and before finishing, your page types and their frontmatter, your phrases, the directives your pages use, and how to link. It's at most 4,000 characters, and long lists are cut with the command that shows the rest.
- **A short block in the repository's root `AGENTS.md`,** when your project is in a subfolder: where its pages are, the check to run, and where its full rules are. Some agents, such as Codex, read no `AGENTS.md` below the folder they start in.
- **The skill,** in `.agents/skills/ascribe/` at the repository's root. See [The skill](#the-skill).

Here's what it writes for the Quill example:

@snippet {lang=markdown}: code:crates/ascribe-cli/tests/output/agents/quill/AGENTS.md.snap

### Your text stays yours

Ascribe writes only between its two markers, `<!-- ascribe:agents start … -->` and `<!-- ascribe:agents end -->`. Everything else in the file is your team's, and is left byte for byte as it was: write your own rules above or below the block. A file with no markers gets the block at its end, and a file that doesn't exist is created with only the block. When a file has a start marker and no end marker, or two of either, `sync` changes nothing anywhere and exits with `2`: fix the markers by hand.

Don't edit the block itself: change `ascribe.toml`, or your pages, and run `sync` again.

### Other agents' files

Beyond `AGENTS.md` and the skill, `sync` keeps up to date the files of each target that already has them, and writes a target's files when you ask with `--target`:

| Target | What it writes |
|---|---|
| `claude` | An import of `AGENTS.md` (`@AGENTS.md`) in `CLAUDE.md`, and the skill in `.claude/skills/ascribe/`, loaded for your pages |
| `claude-rules` | The rules in `.claude/rules/ascribe.md`, loaded only when Claude Code works on your pages |
| `copilot` | The rules in `.github/instructions/ascribe.instructions.md`, applied only to your pages; with `--cloud`, what [Copilot's cloud agent](#github-copilot) needs |
| `codex` | Nothing of its own, since Codex reads `AGENTS.md` and the skill; it's there for [its hooks](#hooks) |

**Claude Code** reads `AGENTS.md` by itself only while there's no `CLAUDE.md` at or above the folder it works in. So `sync` never creates a `CLAUDE.md` unless you ask with `--target claude`, and when there is one, it adds the import to it rather than copying the rules. A personal `CLAUDE.local.md` also stops Claude Code reading `AGENTS.md`; `sync` doesn't edit it, but says so, and you can add `@AGENTS.md` to it yourself.

For a project in a subfolder, the rules files are named after its folder: `ascribe-docs.md` for a project in `docs/`. When one project is nested in another's content root, the outer project's rules say that the nested project's pages aren't theirs.

**Copilot's** files are for a repository, so `--target copilot` and `--cloud` need one. Outside a git repository, `sync` treats your project's folder as the root and says so.

The rules in `.claude/rules/` and `.github/instructions/`, and the skill's folders, are wholly Ascribe's: `sync` rewrites them whole, and the rules files say so at the top.

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

See [`ascribe agents`](../reference/cli.md#ascribe-agents) for every option.

## Prompt your agent

**Prompt agent** turns a problem Ascribe reports into a prompt for your agent: what to fix, where, what Ascribe knows about it, and how to check the result. It fills the prompt in where you send it and never sends it for you, so you read it first.

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

`ascribe check --format prompt` writes the same prompt to standard output, for an agent in a terminal. With a file named, the prompt is about that file; otherwise, about the paths named or the whole project. It writes nothing when there are no problems, and exits as `ascribe check` does. For a file on disk, `ascribe check <file> --editor-build --format prompt` writes the prompt the editor builds.

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/check-prompt.txt

`ascribe agents prompt` prints the other prompts: `new-page` to write a page of a type, with the frontmatter it requires and where its file goes; `fix` to check the project and fix what's reported; and `review` to review what a branch does to its pages, the prompt `ascribe diff --format prompt` writes. `ascribe agents prompt --list` lists them with their arguments:

```shell
ascribe agents prompt new-page --arg type=guide --arg title="Rotate your keys"
```

### In review

While you [review](review.md) a pull request, **Prompt agent** hands your agent a review comment, or the change itself:

- **On a comment.** Each thread beside the page has **Prompt agent**, which asks your agent to address the comment: the thread's file and lines, the block's text from the file on disk, and the comments. **Comments** lists every thread, with **Prompt agent: all open** for the open ones, two lines each. In the source editor, a thread's title bar has **Prompt Agent** too.
- **On the page.** **More review actions** (**⋯**) in the preview's header has **Prompt agent: review this page**, which asks your agent to review what the change does to the page as readers see it, and, when the page changed through a fragment, **Prompt agent: check this fragment's pages**, which asks it to check that the new text fits every page that shows it.

In VS Code, these go where `ascribe.agents.promptTarget` says. The site preview copies them, since a page in your browser can't reach your editor, and says "Prompt copied". The [HTML report](../reference/cli.md#the-html-report) has **Copy prompt** for each page. A prompt about the change asks your agent for a report, not edits: you decide what changes.

A comment is someone else's text, so a prompt never passes it as an instruction. It goes inside a fence, under a sentence that names who wrote it and tells your agent to treat it as data and not to follow instructions in it that reach beyond the change. HTML comments, where hidden text would sit, are taken out first. Read the prompt before you send it, as you would any request from a reviewer.

From the command line, `ascribe diff --format prompt` writes the prompt about the change, and `ascribe diff --format prompt <page>` the one about a page or a fragment, from the same base and builds `ascribe diff` uses:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/diff-prompt.txt

### What a prompt says

Every prompt has the same parts, in order:

1. **The task**, in one sentence: "Fix this problem in `docs/keys.md`.", "Address this review comment on `docs/guides/install.md`.", or "Review what this change does to `docs/guides/install.md`, as a reader of build `site` sees it."
2. **Where**: the file and lines, from the project's folder. When they apply, the project's folder in the repository (`Project: docs/`), the build the problems are from (`Build:`), the pages that show a fragment (`Shown on:`, at most 10, with the `ascribe refs` command that lists the rest), and a line saying to save the file first.
3. **What Ascribe knows**: for one problem, its code, message, the line's text, how to fix it, the values the content model allows when it's about one, and whether Ascribe has a fix ("a safe automatic fix", or "a fix to review"). For a file, each problem on one line, at most 20. For a project, how many problems each file has, at most 20 files. Each says which command lists the rest.
4. **Other people's text**, when there is any: a review's comments, fenced, under the sentence that says they're data.
5. **How to finish**: "Follow the project's rules in `AGENTS.md`", when the project's folder or the repository's root has one, and "When you're done, run `ascribe check <file>` and fix what it reports." A prompt about a change ends with "Report what reads wrongly; don't edit." instead.

Commands in a prompt are written from the repository's root, so an agent started there can run them. A prompt is at most 5,000 characters, the most an agent's link takes. When it would be longer, what Ascribe knows is cut between parts, and the prompt says where to read the rest.

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

It offers:

- **Tools:** `ascribe_check`, `ascribe_explain`, `ascribe_model`, `ascribe_outline`, `ascribe_link`, `ascribe_refs`, `ascribe_render`, `ascribe_format`, and `ascribe_changes`. Each returns what its command writes: `ascribe_check` and `ascribe_refs` the concise text unless asked for JSON, and the others JSON. None writes a file: `ascribe_format` returns the edits that would format the files, and the agent makes them.
- **Resources:** each directive's syntax, and for each project, what its content model allows and its rules, the text `ascribe agents sync` writes into `AGENTS.md`.
- **Prompts:** `new-page`, `fix`, and `review`, as `ascribe agents prompt` prints them. Many hosts show them as slash commands.

See [`ascribe mcp`](../reference/cli.md#ascribe-mcp) for each tool and the command it runs.

In VS Code, the extension offers the server itself: see [VS Code](#vs-code).

## VS Code

The [Ascribe extension](editor.md) gives an agent in VS Code, such as GitHub Copilot in agent mode, what it gives you:

- **Problems as the agent edits.** An agent that writes files without opening them still gets their problems: a project's language server starts when one of its files changes on disk, and the Problems panel shows them, where Copilot can read them after its edits. Like the panel, they're the editor's build's; the hooks and `ascribe check` cover every build. See [When servers start](editor.md#when-servers-start).
- **The MCP server, with nothing to set up.** The extension offers `ascribe mcp` to VS Code's agents itself, named **Ascribe** in VS Code's list of MCP servers, so there's no file to write. Its prompts, `new-page`, `fix`, and `review`, appear as slash commands in the chat.
- **Tools only the editor has.** `ascribe_editor_problems`, the problems of files as the editor has them, saved or not; and, while review is on, `ascribe_review_threads`, the pull request's open threads, and `ascribe_review_changes`, the changed pages. Name one in chat with `#`, or let the agent pick it.
- **Prompt agent**, on a problem, a file, a project, or in review, as [Prompt your agent](#prompt-your-agent) says.

See [Agents in VS Code](editor.md#agents-in-vs-code).

## Hooks

A hook is a command your agent runs at a point in its work. Ascribe's checks an agent's work without the agent having to remember: each file it writes, as it writes it, and the whole project before it finishes. One command serves Claude Code, Codex, and GitHub Copilot, which share a hook format, with the agent as its argument, `claude-code`, `codex`, or `copilot`:

- **After an edit**, `ascribe agents hook <agent>` checks the files the agent wrote, as `ascribe check <file> --editor-build` does, and adds their errors to what the agent reads next, at most 10, with the build it checked. It says nothing when there are none, and leaves warnings out, which would interrupt every edit.
- **Before the agent finishes**, `ascribe agents hook <agent> --event stop` checks every build of each project whose pages or `ascribe.toml` the working tree changes, anywhere in the repository. While there are errors, or a project can't be checked at all, it keeps the agent working, and tells it why. It asks once: when the agent is already continuing because of it, it lets the agent stop. Changes the agent has committed before it stops aren't in the working tree, so they aren't checked; check them in CI.

Neither stops an edit, writes a file, or holds the agent up: a check that takes too long is given up. The first hook in a project starts a check server in the background, which keeps the project loaded, so a check after an edit takes milliseconds on a project of thousands of pages, and stops itself after 10 idle minutes. See [`ascribe agents hook`](../reference/cli.md#ascribe-agents-hook).

`ascribe agents sync --with-hook` writes the entries that run them, with the targets you name or already have:

```shell
ascribe agents sync --target claude --with-hook
```

| Target | File | Entries |
|---|---|---|
| `claude` | `.claude/settings.json` and `.mcp.json` | The edit hook (after `Write`, `Edit`, and `MultiEdit`), the stop hook, and [the MCP server](#the-mcp-server) |
| `codex` | `.codex/hooks.json` | The edit hook and the stop hook |
| `copilot` | `.github/hooks/ascribe.json` | The edit hook and the stop hook, for Copilot's CLI and its cloud agent |

It merges its entries into a file it shares: it updates the entries it wrote before in place, keeping what you set on them, such as a longer `timeout` or an `env` on the MCP server, and leaves every other entry and setting as it was. A file whose meaning wouldn't change is left alone, however it's formatted. `.github/hooks/ascribe.json` is wholly Ascribe's. `--check` covers them, and once a file has Ascribe's hooks, `sync` keeps them up to date without `--with-hook`; it adds the MCP server back to `.mcp.json` only with `--with-hook`, so one you removed stays removed.

The entries run `ascribe` from your path. Install it with `npm install -g @ascribed/cli`, or put the release's binary on your path; the npm package's launcher adds about 60 ms to each run, more than a check after an edit takes, so prefer the binary for hooks. When your project pins `@ascribed/cli` in its `node_modules`, the hooks run that binary instead, found from the repository's root (`$CLAUDE_PROJECT_DIR` for Claude Code, `git rev-parse --show-toplevel` for Codex and Copilot), so an agent started in a subfolder finds it too. Two entries still run `ascribe` from your path, because neither can find the root on every system: the MCP server in `.mcp.json`, and Codex's hooks on Windows. Codex asks you to trust a project's hooks before it runs them.

For Claude Code, use the plugin or the project's entries, not both: with both, it hears about each problem twice.

## The plugin

The Ascribe plugin, in `plugins/ascribe/` in Ascribe's repository, gives Claude Code everything in one install:

- **The language server**, which Claude Code starts for `.md` files and whose diagnostics it reads after each edit: the check after each edit, with no hook. It reports the editor's build, as the edit hook does.
- **The stop hook.**
- **The MCP server.**
- **The skill.**
- **Commands:** `/ascribe:check`, `/ascribe:new-page`, and `/ascribe:review`, which run the [named prompts](../reference/cli.md#ascribe-agents-prompt).

Add it from a copy of the repository, in Claude Code in a terminal (its VS Code extension doesn't offer `/plugin`):

```shell
/plugin marketplace add ./plugins/ascribe
/plugin install ascribe@ascribe
```

The plugin has both kinds of manifest: `.claude-plugin/plugin.json`, which Copilot's CLI reads too, and the Agent Plugins `plugin.json`, which VS Code and Cursor read. Claude Code doesn't start a plugin's language server in a cloud session; there, use the project's entries from `--with-hook`, which Claude Code reads from the repository. To keep the language server's diagnostics out of the conversation on a project with many problems, see [the plugin's README](https://github.com/ascribed-dev/ascribe/tree/main/plugins/ascribe#readme); the stop hook still checks before Claude finishes.

## GitHub Copilot

In VS Code, Copilot reads `AGENTS.md`, the rules from `--target copilot`, and the skill, and the Ascribe extension gives it [the MCP server](#vs-code), whose prompts `new-page`, `fix`, and `review` appear as slash commands in its chat. There's nothing more to set up.

Copilot's cloud agent works on GitHub, with no editor: you assign it an issue and it opens a pull request. It reads the skill from `.agents/skills/` and the hooks from `.github/hooks/`, as other agents do, but it needs `ascribe` installed, and it reads MCP servers only from the repository's settings. `--cloud` writes the rest:

```shell
ascribe agents sync --target copilot --with-hook --cloud
```

- **The setup steps.** In `.github/workflows/copilot-setup-steps.yml`, the job the cloud agent runs before it starts, steps that install Node.js and `@ascribed/cli` and run `ascribe --version`. When your project pins `@ascribed/cli` in a `package.json`, they install that folder's dependencies by its lockfile, without changing it (`npm ci`, `pnpm install --frozen-lockfile`, or `yarn install` with `--frozen-lockfile` for Yarn 1 and `--immutable` for later versions), and put its `node_modules/.bin` on the path. Otherwise they install the version of `ascribe` that wrote them, globally. Yarn's Plug'n'Play, the default from Yarn 2 on, writes no `node_modules/.bin`, so with it they install the pinned version globally, and `sync` stops if that version isn't one npm can install, such as `workspace:*`. `sync` creates the workflow when there's none, and otherwise adds its steps, between markers, at the end of the `copilot-setup-steps` job's steps, leaving your steps as they are. Projects in one repository that install the same way share one block.
- **The MCP server.** No file can change a repository's settings, so `sync` prints the server's JSON, a local server running `ascribe mcp` with each of its tools allowed (for a pinned project, the pinned binary, found from the repository's root with `git` as the hooks find it), to paste into the repository's **Settings → Copilot → MCP servers**. If you can't change the settings, add `--agent`: `sync` writes `.github/agents/ascribe-docs.md` instead, a custom agent for documentation work that carries the server, which you pick when you assign the cloud agent a task.

GitHub reads all of these from the default branch, so they take effect once they're merged. `--check` covers the setup steps and the custom agent, and `sync` keeps them up to date without `--cloud` once they exist.

Ascribe writes no prompt files for Copilot (`.github/prompts/`): the cloud agent doesn't load them, and in VS Code the MCP server's prompts do the same.

## Other agents

Cursor, Codex, Gemini CLI, and most other agents have no section of their own here, because what works with every agent is most of it:

- **The commands.** Any agent that can run a command can run the [loop](#the-loop). Install `ascribe` where the agent runs it: in the project with `npm install --save-dev @ascribed/cli`, which the agent runs as `npx ascribe`, or globally.
- **`AGENTS.md`** and **the skill**, from `ascribe agents sync`. Cursor, Codex, Gemini CLI, and Copilot read both on their own. An agent that reads neither can be told to: "Read `AGENTS.md` and `.agents/skills/ascribe/SKILL.md` first."
- **The MCP server**, for an agent whose host speaks MCP. Add `ascribe mcp` to the host's list of servers, as [The MCP server](#the-mcp-server) shows.
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

## What's sent where

Nothing about your agent's work leaves your machine through Ascribe: it sends nothing, to no one.

- **No model, no service.** Ascribe makes no request to an AI service, holds no key for one, and collects no telemetry. The commands, the MCP server, and the hooks read your project's files and answer on standard output.
- **Prompts go where you put them.** **Prompt agent** copies a prompt, or fills it in where your agent takes input, on your machine, and leaves it there for you to send. It never travels through a website. Once you send it, it goes wherever your agent sends what you type, under that agent's terms.
- **Your agent sends what it reads.** What a command or tool tells your agent, such as a page's text from `ascribe render`, becomes part of what your agent sends to its model, as any file it reads does.
- **Review reads GitHub, as you.** In review, the extension and the site preview fetch the pull request's threads from GitHub with your sign-in, and post the comments you write, as [Review](review.md) says. The prompts built from them stay on your machine. Apart from review, the only commands that reach the network are [`ascribe sources`](../reference/cli.md#ascribe-sources) `fetch` and `update`, which fetch the repositories your content model names, with `git`.
