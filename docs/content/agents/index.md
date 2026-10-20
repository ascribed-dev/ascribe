---
type: how-to
title: Agents
description: Working on an Ascribe project with an AI coding agent, such as Claude Code, GitHub Copilot, or Cursor, and handing it the problems Ascribe finds.
available: ascribe 0.3.0
permalink: agents-guide
---

An agent is the AI coding tool you work with: Claude Code, GitHub Copilot, Cursor, Codex, and others. Ascribe gives your agent what it knows about a project, in a form an agent reads well, and checks what the agent writes:

- **Commands that answer its questions**, and a check it can run on one file after each edit. See [The loop](loop.md).
- **Instructions it reads on its own**: your project's rules in `AGENTS.md`, and a skill that teaches it the loop. See [Agent instructions](instructions.md).
- **Prompts you hand it**: **Prompt agent** on a problem, a review comment, or a changed page. See [Prompt your agent](prompts.md).
- **The same answers as tools**, for an agent without a shell: [the MCP server](mcp.md).
- **A check it can't forget**: [hooks](hooks.md) that check each edit, and the whole project before it finishes.

What Ascribe doesn't do: it doesn't call a model, holds no key for one, and has no chat of its own. It never sends a prompt, and never writes a page for your agent: its tools answer questions and return edits, and your agent makes them, with your approvals. See [What's sent where](whats-sent.md).

Nothing here is set up until you run a command for it. The quickest start, in a project's folder:

```shell
ascribe agents sync
```

That writes `AGENTS.md` and the skill, which most agents read on their own. Then add what your agent can use: [the MCP server](mcp.md), [the hooks](hooks.md), or, in Claude Code, [the plugin](claude-code.md#the-plugin).

This guide writes commands as `ascribe`. With `@ascribed/cli` installed in your project, as [Install](../install.md#npm) says, run them as `npx ascribe` (`pnpm exec ascribe` with pnpm); your agent does the same.
