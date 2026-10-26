---
type: how-to
title: Agents
description: Working on an Ascribe project with an AI coding agent, such as Claude Code, GitHub Copilot, or Cursor, and handing it the problems Ascribe finds.
permalink: agents-guide
---

An [agent](../glossary.md#agent) is the AI coding tool you work with: Claude Code, GitHub Copilot, Cursor, Codex, and others. Ascribe gives your agent what it knows about a project, in a form an agent reads well, and checks what the agent writes. Its tools answer questions and return edits; your agent makes them, with your approvals.

Nothing here is set up until you run a command for it. The quickest start, in a project's folder:

```shell
ascribe agents sync
```

That writes `AGENTS.md` and the skill, which most agents read on their own. Then add what your agent can use:

- [Work with an agent](loop.md): the loop of editing a page and checking it, and the commands that answer what an agent would otherwise guess.
- [Write the agent files](instructions.md): your project's rules in `AGENTS.md`, and the skill that teaches the loop.
- [Prompt your agent](prompts.md): **Prompt agent** on a problem, a review comment, or a changed page.
- [Set up the MCP server](mcp.md): the same answers as tools, for an agent without a shell.
- [Check an agent's edits with hooks](hooks.md): a check it can't forget, after each edit and before it finishes.
- [Claude Code](claude-code.md) and [GitHub Copilot](copilot.md): what each needs beyond the rest. In VS Code, the extension sets up most of it: see [Agents in the editor](../editor/agents.md).
- [Index your docs for your product's agents](product-index.md): for the agents of the people who use your product, not the ones that write your docs.

[What's sent where](whats-sent.md) says what leaves your machine, and what to watch for when your agent reads someone else's text.

This guide writes commands as `ascribe`. With `@ascribed/cli` installed in your project, as [Install](../install.md#npm) says, run them as `npx ascribe` (`pnpm exec ascribe` with pnpm); your agent does the same.
