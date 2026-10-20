---
type: how-to
title: Work with an agent
description: "The loop an agent runs on an Ascribe project: edit a page, check it, fix what's reported, and ask the commands that answer what it would otherwise guess."
available: ascribe 0.3.0
---

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

## Fix it, or say it's intended
@available: ascribe 0.3.0

A problem whose next step is `review` may be fine as it is: a page nothing links to because the site's sidebar does, say. Fixing it is still the first answer, and an agent should fix it unless you tell it the problem is intended. When it is, the answer is an [acknowledgement](../writing/directives.md#intended): the check's name and your reason, written where the check reports the problem, so the next person reads why. Don't acknowledge a problem to make a check pass, and don't let an agent do it on its own: a prompt about a review check says to acknowledge it only when you've said so. The editor's **Mark as intended, and write why** writes a placeholder reason, which is reported until you replace it.

## Other agents

Cursor, Codex, Gemini CLI, and most other agents have no section of their own here, because what works with every agent is most of it:

- **The commands.** Any agent that can run a command can run the [loop](loop.md). Install `ascribe` where the agent runs it: in the project with `npm install --save-dev @ascribed/cli`, which the agent runs as `npx ascribe`, or globally.
- **`AGENTS.md`** and **the skill**, from `ascribe agents sync`. Cursor, Codex, Gemini CLI, and Copilot read both on their own. An agent that reads neither can be told to: "Read `AGENTS.md` and `.agents/skills/ascribe/SKILL.md` first."
- **The MCP server**, for an agent whose host speaks MCP. Add `ascribe mcp` to the host's list of servers, as [The MCP server](mcp.md) shows.
- **Prompt agent.** With `clipboard`, the default, a prompt pastes into any agent. `ascribe check --format prompt` and `ascribe agents prompt` write prompts in a terminal.

What doesn't reach every agent is the hooks: Cursor's and Gemini CLI's hook formats differ from the one Claude Code, Codex, and Copilot share, and Ascribe writes neither. Without a hook, an agent checks only when it remembers to. The skill and `AGENTS.md` tell it to, after each edit and before it finishes, but check the project yourself when it's done, or in CI:

```shell
ascribe check --deny-warnings
```

**In Cursor**, the Ascribe extension works as it does in VS Code, and **Prompt agent** can fill Cursor's chat in with `ascribe.agents.promptTarget` set to `cursor`. Cursor installs extensions from Open VSX, where Ascribe isn't, so install the `.vsix` for your platform from a [release](https://github.com/ascribed-dev/ascribe/releases) with **Extensions: Install from VSIX…**. Add the MCP server to `.cursor/mcp.json`.
