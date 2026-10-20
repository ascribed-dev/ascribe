---
type: how-to
title: Set up the MCP server
description: "Give an agent with no shell Ascribe's answers as tools, through `ascribe mcp`."
available: ascribe 0.3.0
---

`ascribe mcp` is a server for the [Model Context Protocol](https://modelcontextprotocol.io), which most agents' hosts speak. It offers the commands that answer questions as tools an agent calls, for an agent with no shell, or a host that asks you to approve each command it runs. An agent with a shell needs no server: the commands give the same answers.

In VS Code, the extension offers the server itself, so there's nothing to set up: see [Agents in the editor](../editor/agents.md#agents-in-vs-code). In Claude Code, [the plugin](claude-code.md) has it, and `ascribe agents sync --target claude --with-hook` writes it into `.mcp.json`.

## Add the server to your agent's host

Your agent's host starts the server, in the repository's root, and one server serves every project in it.

@steps
1. Open the file your host reads its servers from, such as `.mcp.json` at the repository's root for Claude Code, or `.cursor/mcp.json` for Cursor.
2. Add Ascribe to it:

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

3. Start a new session, and check that the host lists the server's tools.

## What it offers

Its tools are the commands that answer questions; its resources are each directive's syntax, and each project's content model and rules; and its prompts are `new-page`, `fix`, and `review`, which many hosts show as slash commands. Every tool is read-only, so `ascribe_check` never runs [Vale](../checks/vale.md): `ascribe check --vale` in a shell, which your host asks you about, checks the prose. [`ascribe mcp`](commands.md#cli-mcp) lists each tool and the command it runs, and the resources.
