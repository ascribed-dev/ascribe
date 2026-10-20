---
type: how-to
title: Set up the MCP server
description: "Give an agent with no shell Ascribe's answers as tools, through `ascribe mcp`."
available: ascribe 0.3.0
---

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

In VS Code, the extension offers the server itself: see [VS Code](../editor/agents.md#agents-in-vs-code).
